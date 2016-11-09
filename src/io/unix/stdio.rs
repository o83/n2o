use std::prelude::v1::*;
use std::io::prelude::*;

use std;
use std::fmt;
use std::io::{self, BufReader, LineWriter};
use io::poll::{self, Poll, Events};
use io::options::PollOpt;
use io::ready::Ready;
use io::token::Token;
use io::event::Evented;
use std::os::unix::io::{RawFd, FromRawFd, IntoRawFd, AsRawFd};
use libc;
use std::os::raw;

pub struct EventedFd<'a>(pub &'a RawFd);

pub struct Stdin {
    inner: std::io::Stdin,
}

impl Stdin {
    pub fn new() -> Self {
        Stdin { inner: std::io::stdin() }
    }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<'a> Evented for EventedFd<'a> {
    fn register(&self,
                poll: &Poll,
                token: Token,
                interest: Ready,
                opts: PollOpt)
                -> io::Result<()> {
        poll::selector(poll).register(*self.0, token, interest, opts)
    }

    fn reregister(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        poll::selector(poll).reregister(*self.0, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll::selector(poll).deregister(*self.0)
    }
}

impl Evented for Stdin {
    fn register(&self,
                poll: &Poll,
                token: Token,
                interest: Ready,
                opts: PollOpt)
                -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

impl AsRawFd for Stdin {
    fn as_raw_fd(&self) -> RawFd {
        0 as raw::c_int
    }
}
