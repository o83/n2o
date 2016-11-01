use io::poll::*;
use io::readiness::*;
use io::options::PollOpt;
use io::token::Token;
use io::ready::Ready;
use io::event::Event;
use io::evented::Evented;
use io::unix;
use std::{fmt, io, mem, ptr, usize};
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::isize;

const MAX_REFCOUNT: usize = (isize::MAX) as usize;
const NODE_QUEUED_FLAG: usize = 1;

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

#[derive(Clone)]
pub struct SetReadiness {
    inner: RegistrationInner,
}

pub struct Registration {
    inner: RegistrationInner,
}

pub struct RegistrationInner {
    queue: ReadinessQueue,
    node: ReadyRef,
}

pub struct RegistrationData {
    pub token: Token,
    pub interest: Ready,
    pub opts: PollOpt,
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


impl RegistrationInner {
    pub fn new(poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> RegistrationInner {
        let queue = poll.readiness_queue.clone();
        let node = queue.new_readiness_node(token, interest, opts, 1);

        RegistrationInner {
            node: node,
            queue: queue,
        }
    }

    pub fn update(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        try!(self.registration_data_mut(&poll.readiness_queue)).update(token, interest, opts);
        if !::io::event::is_empty(self.readiness()) {
            let needs_wakeup = self.queue_for_processing();
            debug_assert!(!needs_wakeup, "something funky is going on");
        }

        Ok(())
    }

    pub fn readiness(&self) -> Ready {
        ::io::event::from_usize(self.node().events.load(Ordering::Relaxed))
    }

    pub fn set_readiness(&self, ready: Ready) -> io::Result<()> {
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

    pub fn queue_for_processing(&self) -> bool {
        let prev = self.node().queued.compare_and_swap(0, NODE_QUEUED_FLAG, Ordering::AcqRel);
        if prev == 0 {
            self.queue.prepend_readiness_node(self.node.clone())
        } else {
            false
        }
    }

    pub fn node(&self) -> &ReadinessNode {
        self.node.as_ref().unwrap()
    }

    pub fn registration_data_mut(&self,
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


impl RegistrationData {
    pub fn new(token: Token, interest: Ready, opts: PollOpt) -> RegistrationData {
        RegistrationData {
            token: token,
            interest: interest,
            opts: opts,
        }
    }

    pub fn update(&mut self, token: Token, interest: Ready, opts: PollOpt) {
        self.token = token;
        self.interest = interest;
        self.opts = opts;
    }

    pub fn disable(&mut self) {
        self.interest = Ready::none();
        self.opts = PollOpt::empty();
    }
}
