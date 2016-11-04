
//  Rediness Queue

use io::poll::*;
use io::registration::*;
use io::options::PollOpt;
use io::token::Token;
use io::ready::Ready;
use io::event::Event;
use io::unix;
use std::{fmt, io, mem, ptr, usize};
use std::cell::{UnsafeCell, Cell};
use std::isize;
use std::marker;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::time::Duration;

#[derive(Clone)]
pub struct ReadinessQueue {
    inner: Arc<UnsafeCell<ReadinessQueueInner>>,
}

pub struct ReadinessQueueInner {
    pub head_all_nodes: Option<Box<ReadinessNode>>,
    pub head_readiness: AtomicPtr<ReadinessNode>,
    pub sleep_token: Box<ReadinessNode>,
}

pub struct ReadyList {
    head: ReadyRef,
}

pub struct ReadyRef {
    ptr: *mut ReadinessNode,
}

pub struct ReadinessNode {
    pub next_all_nodes: Option<Box<ReadinessNode>>,
    pub prev_all_nodes: ReadyRef,
    pub registration_data: UnsafeCell<RegistrationData>,
    pub next_readiness: ReadyRef,
    pub events: AtomicUsize,
    pub queued: AtomicUsize,
    pub ref_count: AtomicUsize,
}

impl ReadinessQueue {
    pub fn new() -> io::Result<ReadinessQueue> {
        let sleep_token =
            Box::new(ReadinessNode::new(Token(0), Ready::none(), PollOpt::empty(), 0));

        Ok(ReadinessQueue {
            inner: Arc::new(UnsafeCell::new(ReadinessQueueInner {
                head_all_nodes: None,
                head_readiness: AtomicPtr::new(ptr::null_mut()),
                sleep_token: sleep_token,
            })),
        })
    }

    pub fn poll(&self, dst: &mut unix::Events) {
        let ready = self.take_ready();

        for node in ready {
            let mut events;
            let opts;

            {
                let node_ref = node.as_ref().unwrap();
                opts = node_ref.poll_opts();
                let mut queued = node_ref.queued.load(Ordering::Acquire);
                events = node_ref.poll_events();

                loop {
                    if ::io::event::is_drop(events) {
                        break;
                    } else if opts.is_edge() || ::io::event::is_empty(events) {
                        let next = node_ref.queued.compare_and_swap(queued, 0, Ordering::Acquire);
                        events = node_ref.poll_events();
                        if queued == next {
                            break;
                        }
                        queued = next;
                    } else {
                        let needs_wakeup = self.prepend_readiness_node(node.clone());
                        debug_assert!(!needs_wakeup, "something funky is going on");
                        break;
                    }
                }
            }

            if ::io::event::is_drop(events) {
                let _ = self.unlink_node(node);
            } else if !events.is_none() {
                let node_ref = node.as_ref().unwrap();
                println!("readiness event {:?} {:?}", events, node_ref.token());
                dst.push_event(Event::new(events, node_ref.token()));
                if opts.is_oneshot() {
                    node_ref.registration_data_mut().disable();
                }
            }
        }
    }

    pub fn wakeup(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn prepare_for_sleep(&self) -> bool {
        ptr::null_mut() ==
        self.inner()
            .head_readiness
            .compare_and_swap(ptr::null_mut(), self.sleep_token(), Ordering::Relaxed)
    }

    pub fn take_ready(&self) -> ReadyList {
        let mut head = self.inner().head_readiness.swap(ptr::null_mut(), Ordering::Acquire);
        if head == self.sleep_token() {
            head = ptr::null_mut();
        }

        ReadyList { head: ReadyRef::new(head) }
    }

    pub fn new_readiness_node(&self,
                              token: Token,
                              interest: Ready,
                              opts: PollOpt,
                              ref_count: usize)
                              -> ReadyRef {
        let mut node = Box::new(ReadinessNode::new(token, interest, opts, ref_count));
        let ret = ReadyRef::new(&mut *node as *mut ReadinessNode);

        node.next_all_nodes = self.inner_mut().head_all_nodes.take();

        let ptr = &*node as *const ReadinessNode as *mut ReadinessNode;

        if let Some(ref mut next) = node.next_all_nodes {
            next.prev_all_nodes = ReadyRef::new(ptr);
        }

        self.inner_mut().head_all_nodes = Some(node);

        ret
    }

    pub fn prepend_readiness_node(&self, mut node: ReadyRef) -> bool {
        let mut curr_head = self.inner().head_readiness.load(Ordering::Relaxed);

        loop {
            let node_next = if curr_head == self.sleep_token() {
                ptr::null_mut()
            } else {
                curr_head
            };

            node.as_mut().unwrap().next_readiness = ReadyRef::new(node_next);
            let next_head = self.inner()
                .head_readiness
                .compare_and_swap(curr_head, node.ptr, Ordering::Release);

            if curr_head == next_head {
                return curr_head == self.sleep_token();
            }

            curr_head = next_head;
        }
    }

    pub fn unlink_node(&self, mut node: ReadyRef) -> Box<ReadinessNode> {
        node.as_mut().unwrap().unlink(&mut self.inner_mut().head_all_nodes)
    }

    pub fn is_empty(&self) -> bool {
        self.inner().head_readiness.load(Ordering::Relaxed).is_null()
    }

    pub fn sleep_token(&self) -> *mut ReadinessNode {
        &*self.inner().sleep_token as *const ReadinessNode as *mut ReadinessNode
    }

    pub fn identical(&self, other: &ReadinessQueue) -> bool {
        self.inner.get() == other.inner.get()
    }

    pub fn inner(&self) -> &ReadinessQueueInner {
        unsafe { mem::transmute(self.inner.get()) }
    }

    pub fn inner_mut(&self) -> &mut ReadinessQueueInner {
        unsafe { mem::transmute(self.inner.get()) }
    }
}

unsafe impl Send for ReadinessQueue {}

impl ReadinessNode {
    pub fn new(token: Token, interest: Ready, opts: PollOpt, ref_count: usize) -> ReadinessNode {
        ReadinessNode {
            next_all_nodes: None,
            prev_all_nodes: ReadyRef::none(),
            registration_data: UnsafeCell::new(RegistrationData::new(token, interest, opts)),
            next_readiness: ReadyRef::none(),
            events: AtomicUsize::new(0),
            queued: AtomicUsize::new(0),
            ref_count: AtomicUsize::new(ref_count),
        }
    }

    pub fn poll_events(&self) -> Ready {
        (self.interest() | ::io::event::drop()) &
        ::io::event::from_usize(self.events.load(Ordering::Relaxed))
    }

    pub fn token(&self) -> Token {
        unsafe { &*self.registration_data.get() }.token
    }

    pub fn interest(&self) -> Ready {
        unsafe { &*self.registration_data.get() }.interest
    }

    pub fn poll_opts(&self) -> PollOpt {
        unsafe { &*self.registration_data.get() }.opts
    }

    pub fn registration_data_mut(&self) -> &mut RegistrationData {
        unsafe { &mut *self.registration_data.get() }
    }

    pub fn unlink(&mut self, head: &mut Option<Box<ReadinessNode>>) -> Box<ReadinessNode> {
        if let Some(ref mut next) = self.next_all_nodes {
            next.prev_all_nodes = self.prev_all_nodes.clone();
        }

        let node;

        match self.prev_all_nodes.take().as_mut() {
            Some(prev) => {
                node = prev.next_all_nodes.take().unwrap();
                prev.next_all_nodes = self.next_all_nodes.take();
            }
            None => {
                node = head.take().unwrap();
                *head = self.next_all_nodes.take();
            }
        }

        node
    }
}


impl Iterator for ReadyList {
    type Item = ReadyRef;

    fn next(&mut self) -> Option<ReadyRef> {
        let mut next = self.head.take();

        if next.is_some() {
            next.as_mut().map(|n| self.head = n.next_readiness.take());
            Some(next)
        } else {
            None
        }
    }
}


impl ReadyRef {
    pub fn new(ptr: *mut ReadinessNode) -> ReadyRef {
        ReadyRef { ptr: ptr }
    }

    pub fn none() -> ReadyRef {
        ReadyRef { ptr: ptr::null_mut() }
    }

    pub fn take(&mut self) -> ReadyRef {
        let ret = ReadyRef { ptr: self.ptr };
        self.ptr = ptr::null_mut();
        ret
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_none(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn as_ref(&self) -> Option<&ReadinessNode> {
        if self.ptr.is_null() {
            return None;
        }

        unsafe { Some(&*self.ptr) }
    }

    pub fn as_mut(&mut self) -> Option<&mut ReadinessNode> {
        if self.ptr.is_null() {
            return None;
        }

        unsafe { Some(&mut *self.ptr) }
    }
}

impl Clone for ReadyRef {
    fn clone(&self) -> ReadyRef {
        ReadyRef::new(self.ptr)
    }
}

impl fmt::Pointer for ReadyRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            Some(r) => fmt::Pointer::fmt(&r, fmt),
            None => fmt::Pointer::fmt(&ptr::null::<ReadinessNode>(), fmt),
        }
    }
}
