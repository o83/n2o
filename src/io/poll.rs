
use io::options::PollOpt;
use io::token::Token;
use io::ready::Ready;
use io::event::Event;
use io::evented::Evented;

use std::{fmt, io, mem, ptr, usize};
use std::cell::{UnsafeCell, Cell};
use std::isize;
use std::marker;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::time::Duration;

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

pub struct Poll {
    _marker: marker::PhantomData<Cell<()>>,
    selector: ::io::posix::epoll::Selector,
    readiness_queue: ReadinessQueue,
}

pub struct Registration {
    inner: RegistrationInner,
}

#[derive(Clone)]
pub struct SetReadiness {
    inner: RegistrationInner,
}

struct RegistrationInner {
    queue: ReadinessQueue,
    node: ReadyRef,
}

#[derive(Clone)]
struct ReadinessQueue {
    inner: Arc<UnsafeCell<ReadinessQueueInner>>,
}

struct ReadinessQueueInner {
    //    awakener: sys::Awakener,
    head_all_nodes: Option<Box<ReadinessNode>>,
    head_readiness: AtomicPtr<ReadinessNode>,
    sleep_token: Box<ReadinessNode>,
}

struct ReadyList {
    head: ReadyRef,
}

struct ReadyRef {
    ptr: *mut ReadinessNode,
}

struct ReadinessNode {
    next_all_nodes: Option<Box<ReadinessNode>>,
    prev_all_nodes: ReadyRef,
    registration_data: UnsafeCell<RegistrationData>,
    next_readiness: ReadyRef,
    events: AtomicUsize,
    queued: AtomicUsize,
    ref_count: AtomicUsize,
}

struct RegistrationData {
    token: Token,
    interest: Ready,
    opts: PollOpt,
}

type Tick = usize;

const NODE_QUEUED_FLAG: usize = 1;

const AWAKEN: Token = Token(usize::MAX);

impl Poll {
    pub fn new() -> io::Result<Poll> {
        let poll = Poll {
            selector: try!(::io::posix::epoll::Selector::new()),
            readiness_queue: try!(ReadinessQueue::new()),
            _marker: marker::PhantomData,
        };
        //        try!(poll.readiness_queue
        //            .inner()
        //            .awakener
        //            .register(&poll, AWAKEN, Ready::readable(), PollOpt::edge()));
        Ok(poll)
    }

    pub fn register<E: ?Sized>(&self,
                               io: &E,
                               token: Token,
                               interest: Ready,
                               opts: PollOpt)
                               -> io::Result<()>
        where E: Evented
    {
        try!(validate_args(token, interest));
        println!("registering with poller");
        try!(io.register(self, token, interest, opts));
        Ok(())
    }

    pub fn reregister<E: ?Sized>(&self,
                                 io: &E,
                                 token: Token,
                                 interest: Ready,
                                 opts: PollOpt)
                                 -> io::Result<()>
        where E: Evented
    {
        try!(validate_args(token, interest));
        println!("registering with poller");
        try!(io.reregister(self, token, interest, opts));
        Ok(())
    }

    pub fn deregister<E: ?Sized>(&self, io: &E) -> io::Result<()>
        where E: Evented
    {
        println!("deregistering IO with poller");
        try!(io.deregister(self));
        Ok(())
    }

    pub fn poll(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<usize> {
        let timeout = if !self.readiness_queue.is_empty() {
            println!("custom readiness queue has pending events");
            Some(Duration::from_millis(0))
        } else if !self.readiness_queue.prepare_for_sleep() {
            Some(Duration::from_millis(0))
        } else {
            timeout
        };
        let awoken = try!(self.selector.select(&mut events.inner, AWAKEN, timeout));
        //        if awoken {
        //            self.readiness_queue.inner().awakener.cleanup();
        //        }
        self.readiness_queue.poll(&mut events.inner);
        Ok(events.len())
    }
}

fn validate_args(token: Token, interest: Ready) -> io::Result<()> {
    if token == AWAKEN {
        return Err(io::Error::new(io::ErrorKind::Other, "invalid token"));
    }

    if !interest.is_readable() && !interest.is_writable() {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  "interest must include readable or writable"));
    }

    Ok(())
}

impl fmt::Debug for Poll {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Poll")
    }
}

pub struct Events {
    inner: ::io::posix::epoll::Events,
}

pub struct EventsIter<'a> {
    inner: &'a Events,
    pos: usize,
}

impl Events {
    pub fn with_capacity(capacity: usize) -> Events {
        Events { inner: ::io::posix::epoll::Events::with_capacity(capacity) }
    }

    pub fn get(&self, idx: usize) -> Option<::io::event::Event> {
        self.inner.get(idx)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> EventsIter {
        EventsIter {
            inner: self,
            pos: 0,
        }
    }
}

impl<'a> IntoIterator for &'a Events {
    type Item = Event;
    type IntoIter = EventsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> Iterator for EventsIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        let ret = self.inner.get(self.pos);
        self.pos += 1;
        ret
    }
}

pub fn selector(poll: &Poll) -> &::io::posix::epoll::Selector {
    &poll.selector
}

impl Registration {
    pub fn new(poll: &Poll,
               token: Token,
               interest: Ready,
               opts: PollOpt)
               -> (Registration, SetReadiness) {
        let inner = RegistrationInner::new(poll, token, interest, opts);
        let registration = Registration { inner: inner.clone() };
        let set_readiness = SetReadiness { inner: inner.clone() };

        (registration, set_readiness)
    }

    pub fn update(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        self.inner.update(poll, token, interest, opts)
    }

    pub fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.inner.update(poll, Token(0), Ready::none(), PollOpt::empty())
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        let inner = &self.inner;
        inner.registration_data_mut(&inner.queue).unwrap().disable();
    }
}

impl fmt::Debug for Registration {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Registration")
            .finish()
    }
}

unsafe impl Send for Registration {}

impl SetReadiness {
    pub fn readiness(&self) -> Ready {
        self.inner.readiness()
    }

    pub fn set_readiness(&self, ready: Ready) -> io::Result<()> {
        self.inner.set_readiness(ready)
    }
}

unsafe impl Send for SetReadiness {}
unsafe impl Sync for SetReadiness {}

impl RegistrationInner {
    fn new(poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> RegistrationInner {
        let queue = poll.readiness_queue.clone();
        let node = queue.new_readiness_node(token, interest, opts, 1);

        RegistrationInner {
            node: node,
            queue: queue,
        }
    }

    fn update(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        try!(self.registration_data_mut(&poll.readiness_queue)).update(token, interest, opts);
        if !::io::event::is_empty(self.readiness()) {
            let needs_wakeup = self.queue_for_processing();
            debug_assert!(!needs_wakeup, "something funky is going on");
        }

        Ok(())
    }

    fn readiness(&self) -> Ready {
        ::io::event::from_usize(self.node().events.load(Ordering::Relaxed))
    }

    fn set_readiness(&self, ready: Ready) -> io::Result<()> {
        self.node().events.store(::io::event::as_usize(ready), Ordering::Relaxed);
        println!("readiness event {:?} {:?}", ready, self.node().token());
        if ::io::event::is_empty(ready) {
            return Ok(());
        }
        if self.queue_for_processing() {
            try!(self.queue.wakeup());
        }
        Ok(())
    }

    fn queue_for_processing(&self) -> bool {
        let prev = self.node().queued.compare_and_swap(0, NODE_QUEUED_FLAG, Ordering::AcqRel);
        if prev == 0 {
            self.queue.prepend_readiness_node(self.node.clone())
        } else {
            false
        }
    }

    fn node(&self) -> &ReadinessNode {
        self.node.as_ref().unwrap()
    }

    fn registration_data_mut(&self,
                             readiness_queue: &ReadinessQueue)
                             -> io::Result<&mut RegistrationData> {
        if !self.queue.identical(readiness_queue) {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      "registration registered with another instance of Poll"));
        }

        Ok(self.node().registration_data_mut())
    }
}

impl Clone for RegistrationInner {
    fn clone(&self) -> RegistrationInner {
        let old_size = self.node().ref_count.fetch_add(1, Ordering::Relaxed);
        if old_size & !MAX_REFCOUNT != 0 {
            panic!("too many outstanding refs");
        }
        RegistrationInner {
            queue: self.queue.clone(),
            node: self.node.clone(),
        }
    }
}

impl Drop for RegistrationInner {
    fn drop(&mut self) {
        let old_size = self.node().ref_count.fetch_sub(1, Ordering::Release);
        if old_size != 1 {
            return;
        }
        let _ = self.set_readiness(::io::event::drop());
    }
}

impl ReadinessQueue {
    fn new() -> io::Result<ReadinessQueue> {
        let sleep_token =
            Box::new(ReadinessNode::new(Token(0), Ready::none(), PollOpt::empty(), 0));

        Ok(ReadinessQueue {
            inner: Arc::new(UnsafeCell::new(ReadinessQueueInner {
                //                awakener: try!(sys::Awakener::new()),
                head_all_nodes: None,
                head_readiness: AtomicPtr::new(ptr::null_mut()),
                sleep_token: sleep_token,
            })),
        })
    }

    fn poll(&self, dst: &mut ::io::posix::epoll::Events) {
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

    fn wakeup(&self) -> io::Result<()> {
        Ok(())
    }

    fn prepare_for_sleep(&self) -> bool {
        ptr::null_mut() ==
        self.inner()
            .head_readiness
            .compare_and_swap(ptr::null_mut(), self.sleep_token(), Ordering::Relaxed)
    }

    fn take_ready(&self) -> ReadyList {
        let mut head = self.inner().head_readiness.swap(ptr::null_mut(), Ordering::Acquire);
        if head == self.sleep_token() {
            head = ptr::null_mut();
        }

        ReadyList { head: ReadyRef::new(head) }
    }

    fn new_readiness_node(&self,
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

    fn prepend_readiness_node(&self, mut node: ReadyRef) -> bool {
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

    fn unlink_node(&self, mut node: ReadyRef) -> Box<ReadinessNode> {
        node.as_mut().unwrap().unlink(&mut self.inner_mut().head_all_nodes)
    }

    fn is_empty(&self) -> bool {
        self.inner().head_readiness.load(Ordering::Relaxed).is_null()
    }

    fn sleep_token(&self) -> *mut ReadinessNode {
        &*self.inner().sleep_token as *const ReadinessNode as *mut ReadinessNode
    }

    fn identical(&self, other: &ReadinessQueue) -> bool {
        self.inner.get() == other.inner.get()
    }

    fn inner(&self) -> &ReadinessQueueInner {
        unsafe { mem::transmute(self.inner.get()) }
    }

    fn inner_mut(&self) -> &mut ReadinessQueueInner {
        unsafe { mem::transmute(self.inner.get()) }
    }
}

unsafe impl Send for ReadinessQueue {}

impl ReadinessNode {
    fn new(token: Token, interest: Ready, opts: PollOpt, ref_count: usize) -> ReadinessNode {
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

    fn poll_events(&self) -> Ready {
        (self.interest() | ::io::event::drop()) &
        ::io::event::from_usize(self.events.load(Ordering::Relaxed))
    }

    fn token(&self) -> Token {
        unsafe { &*self.registration_data.get() }.token
    }

    fn interest(&self) -> Ready {
        unsafe { &*self.registration_data.get() }.interest
    }

    fn poll_opts(&self) -> PollOpt {
        unsafe { &*self.registration_data.get() }.opts
    }

    fn registration_data_mut(&self) -> &mut RegistrationData {
        unsafe { &mut *self.registration_data.get() }
    }

    fn unlink(&mut self, head: &mut Option<Box<ReadinessNode>>) -> Box<ReadinessNode> {
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

impl RegistrationData {
    fn new(token: Token, interest: Ready, opts: PollOpt) -> RegistrationData {
        RegistrationData {
            token: token,
            interest: interest,
            opts: opts,
        }
    }

    fn update(&mut self, token: Token, interest: Ready, opts: PollOpt) {
        self.token = token;
        self.interest = interest;
        self.opts = opts;
    }

    fn disable(&mut self) {
        self.interest = Ready::none();
        self.opts = PollOpt::empty();
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
    fn new(ptr: *mut ReadinessNode) -> ReadyRef {
        ReadyRef { ptr: ptr }
    }

    fn none() -> ReadyRef {
        ReadyRef { ptr: ptr::null_mut() }
    }

    fn take(&mut self) -> ReadyRef {
        let ret = ReadyRef { ptr: self.ptr };
        self.ptr = ptr::null_mut();
        ret
    }

    fn is_some(&self) -> bool {
        !self.is_none()
    }

    fn is_none(&self) -> bool {
        self.ptr.is_null()
    }

    fn as_ref(&self) -> Option<&ReadinessNode> {
        if self.ptr.is_null() {
            return None;
        }

        unsafe { Some(&*self.ptr) }
    }

    fn as_mut(&mut self) -> Option<&mut ReadinessNode> {
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

#[cfg(test)]
mod test {

    use io::poll::*;
    use io::options::PollOpt;
    use io::token::Token;
    use io::ready::Ready;
    use std::time::Duration;

    fn ensure_send<T: Send>(_: &T) {}
    fn ensure_sync<T: Sync>(_: &T) {}

    #[allow(dead_code)]
    fn ensure_type_bounds(r: &Registration, s: &SetReadiness) {
        ensure_send(r);
        ensure_send(s);
        ensure_sync(s);
    }

    fn readiness_node_count(poll: &Poll) -> usize {
        let mut cur = poll.readiness_queue.inner().head_all_nodes.as_ref();
        let mut cnt = 0;

        while let Some(node) = cur {
            cnt += 1;
            cur = node.next_all_nodes.as_ref();
        }

        cnt
    }

    #[test]
    pub fn test_nodes_do_not_leak() {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(1024);
        let mut registrations = Vec::with_capacity(1_000);

        for _ in 0..3 {
            registrations.push(Registration::new(&mut poll, Token(0), Ready::readable(), PollOpt::edge()));
        }

        drop(registrations);

        // Poll
        let num = poll.poll(&mut events, Some(Duration::from_millis(300))).unwrap();

        assert_eq!(0, num);
        assert_eq!(0, readiness_node_count(&poll));
    }
}
