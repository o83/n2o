// Wrapper for userspace io event signaling.

use io::unix;
use io::poll::{self, Poll};
use io::event::Evented;
use io::options::PollOpt;
use io::ready::Ready;
use io::token::Token;
use std::io;

pub struct Notify {
    inner: unix::Notify,
}

impl Notify {
    pub fn new() -> Self {
        Notify { inner: unix::Notify::new() }
    }

    pub fn send(&self) {
        self.inner.send();
    }

    pub fn wait(&self) {
        self.inner.wait();
    }
}

impl Evented for Notify {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        poll::selector(poll).register(self.inner.fd, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        poll::selector(poll).reregister(self.inner.fd, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll::selector(poll).deregister(self.inner.fd)
    }
}
