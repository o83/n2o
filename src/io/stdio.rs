
use std;
use std::io::{self, Read, Write};
use io::token::Token;
use io::ready::Ready;
use io::poll::{self, Poll, Events};
use io::options::*;
use io::unix;
use std::sync::atomic::{AtomicUsize, Ordering};
use io::event::Evented;

pub struct Stdin {
    sys: io::Stdin,
    selector_id: SelectorId,
}

impl Stdin {
    pub fn new() -> Self {
        Stdin {
            sys: io::stdin(),
            selector_id: SelectorId::new(),
        }
    }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.sys).read(buf)
    }
}

impl<'a> Read for &'a Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.sys).read(buf)
    }
}

impl Evented for Stdin {
    fn register(&self,
                poll: &Poll,
                token: Token,
                interest: Ready,
                opts: PollOpt)
                -> io::Result<()> {
        try!(self.selector_id.associate_selector(poll));
        self.sys.register(poll, token, interest, opts)
    }

    fn reregister(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        self.sys.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.sys.deregister(poll)
    }
}

#[derive(Debug)]
struct SelectorId {
    id: AtomicUsize,
}

impl SelectorId {
    fn new() -> SelectorId {
        SelectorId { id: AtomicUsize::new(0) }
    }

    fn associate_selector(&self, poll: &Poll) -> io::Result<()> {
        let selector_id = self.id.load(Ordering::SeqCst);

        if selector_id != 0 && selector_id != poll::selector(poll).id() {
            Err(io::Error::new(io::ErrorKind::Other, "socket already registered"))
        } else {
            self.id.store(poll::selector(poll).id(), Ordering::SeqCst);
            Ok(())
        }
    }
}

impl Clone for SelectorId {
    fn clone(&self) -> SelectorId {
        SelectorId { id: AtomicUsize::new(self.id.load(Ordering::SeqCst)) }
    }
}
