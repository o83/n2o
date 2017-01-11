use std::io::{self, Write};
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::collections::HashMap;
use io::event::Evented;
use std::cell::UnsafeCell;
use reactors::core::Core;
use reactors::ws::WsServer;
use reactors::console::Console;
use core::borrow::BorrowMut;
use handle;
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

pub enum Pool<'a> {
    Raw(&'a [u8]),
    Msg(&'a [Message]),
}

pub trait Select<'a>: Write {
    fn init(&mut self, c: &mut Core, s: Slot);
    fn select(&'a mut self, c: &'a mut Core, t: Token) -> Async<Pool<'a>>;
    fn finalize(&mut self);
    fn with<F, R>(&'a mut self, mut f: F) -> R
        where F: FnMut(&'a mut Self) -> R
    {
        f(self)
    }
}

pub enum Selector {
    Ws(WsServer),
    Rx(Console),
    Sb(Subscriber<u8>),
}

#[macro_export]
macro_rules! with(
    ($x:expr,$e:expr) => ({
        match *$x {
            Selector::Ws(ref mut w) => w.with($e),
            Selector::Rx(ref mut c) => c.with($e),
            Selector::Sb(ref mut s) => s.with($e), 
        }
    })
);

impl Write for Selector {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        with!(self, |x| x.write(buf));
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
