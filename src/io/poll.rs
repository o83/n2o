
//  Kernel Main I/O Event Stream

use io::options::PollOpt;
use io::token::Token;
use io::ready::Ready;
use io::event::{Event, Evented};
use io::unix;
use io::readiness::*;
use io::registration::*;

use std::{fmt, io, mem, ptr, usize};
use std::cell::{UnsafeCell, Cell};
use std::marker;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::time::Duration;

pub struct Poll {
    pub _marker: marker::PhantomData<Cell<()>>,
    pub selector: unix::Selector,
    pub readiness_queue: ReadinessQueue,
}
type Tick = usize;

const AWAKEN: Token = Token(100);

impl Poll {
    pub fn new() -> io::Result<Poll> {
        let poll = Poll {
            selector: try!(unix::Selector::new()),
            readiness_queue: try!(ReadinessQueue::new()),
            _marker: marker::PhantomData,
        };
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
        trace!("registering with poller");
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
        trace!("registering with poller");
        try!(io.reregister(self, token, interest, opts));
        Ok(())
    }

    pub fn deregister<E: ?Sized>(&self, io: &E) -> io::Result<()>
        where E: Evented
    {
        trace!("deregistering IO with poller");
        try!(io.deregister(self));
        Ok(())
    }

    pub fn poll(&self, events: &mut self::Events, timeout: Option<Duration>) -> io::Result<usize> {
        let timeout = if !self.readiness_queue.is_empty() {
            trace!("custom readiness queue has pending events");
            Some(Duration::from_millis(0))
        } else if !self.readiness_queue.prepare_for_sleep() {
            Some(Duration::from_millis(0))
        } else {
            timeout
        };
        let awoken = try!(self.selector.select(&mut events.inner, AWAKEN, timeout));
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
    inner: unix::Events,
}

pub struct EventsIter<'a> {
    inner: &'a Events,
    pos: usize,
}

impl Events {
    pub fn with_capacity(capacity: usize) -> Events {
        Events { inner: unix::Events::with_capacity(capacity) }
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

pub fn selector(poll: &Poll) -> &unix::Selector {
    &poll.selector
}

#[cfg(test)]
mod test {

    use io::poll::*;
    use io::registration::*;
    use io::readiness::*;
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
