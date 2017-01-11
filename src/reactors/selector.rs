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

pub trait Select<'a, T>: Write {
    fn init(&mut self, c: &mut Core, s: Slot);
    fn select(&mut self, c: &mut Core, t: Token, buf: &mut [T]) -> usize;
    fn finalize(&mut self);
}

pub fn with<'a,S,T,F,R>(s: &'a mut S, mut f: F) -> R 
    where S: Select<'a, T>,
          F: FnMut(&mut S) -> R 
    {
        f(s)
    }

pub enum Selector {
    Ws(WsServer),
    Rx(Console),
    Sb(Subscriber<Message>),
}

impl Selector {
    pub fn with<F,R>(&mut self, mut f: F) -> R 
    where F: FnMut(&mut Self) -> R 
    {
        f(self)
    }
}

#[macro_export]
macro_rules! with(
    ($x:expr,$e:expr) => ({
        let (s1,s2) = handle::split($x);
        match *s1 {
            Selector::Ws(ref mut w) => with(w, $e),
            Selector::Rx(ref mut c) => with(c, $e),
            Selector::Sb(ref mut s) => with(s, $e), 
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
