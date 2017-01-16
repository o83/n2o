use std::io::{self, Write};
use io::token::Token;
use reactors::system::IO;
use reactors::ws::WsServer;
use reactors::console::Console;
use std::fmt::Arguments;
use queues::publisher::Subscriber;
use streams::intercore::api::Message;

const EVENTS_CAPACITY: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 16;
const READ_BUF_SIZE: usize = 2048;

#[derive(Debug)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}

pub struct RingLock<'a> {
    pub buf: &'a [Message],
    pub sub: &'a Subscriber<Message>,
}

impl<'a> Drop for RingLock<'a> {
    fn drop(&mut self) {
        self.sub.commit();
    }
}

pub enum Pool<'a> {
    Raw(&'a [u8]),
    Msg(RingLock<'a>),
}

pub trait Select<'a>: Write {
    fn init(&mut self, io: &mut IO, s: Slot);
    fn select(&'a mut self, io: &'a mut IO, t: Token) -> Async<Pool<'a>>;
    fn finalize(&mut self);
}

pub enum Selector {
    Ws(WsServer),
    Rx(Console),
    Sb(Subscriber<Message>),
}

impl Selector {
    pub fn unpack<'a>(&'a mut self) -> &'a mut Select<'a> {
        match *self {
            Selector::Ws(ref mut w) => w,
            Selector::Rx(ref mut c) => c,
            Selector::Sb(ref mut s) => s,
        }
    }

    pub fn map<'a, F, R>(&'a mut self, mut f: F) -> R
        where F: FnMut(&'a mut Select<'a>) -> R
    {
        f(self.unpack())
    }
}

impl Write for Selector {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.unpack().write(buf);
        Ok(1)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        Ok(())
    }
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub struct Slot(pub usize);
