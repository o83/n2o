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

const EVENTS_CAPACITY: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 16;
const READ_BUF_SIZE: usize = 2048;

#[derive(Debug)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}

pub trait Select<'a>: Write {
    fn init(&mut self, c: &mut Core, s: Slot);
    fn select(&mut self, c: &mut Core, t: Token, buf: &mut [u8]) -> usize;
    fn finalize(&mut self);
}

pub enum Selector {
    Ws(WsServer),
    Rx(Console),
    Sb(Subscriber<u64>),
}

impl Selector {
    pub fn unwrap<'a>(&mut self) -> &mut Select<'a> {
        match *self {
            Selector::Ws(ref mut w) => w,
            Selector::Rx(ref mut c) => c,
            Selector::Sb(ref mut s) => s, 
        }
    }
}

impl<'a> Select<'a> for Selector {
    fn init(&mut self, c: &mut Core, s: Slot) {
        self.unwrap().init(c, s);
    }
    fn select(&mut self, c: &mut Core, t: Token, buf: &mut [u8]) -> usize {
        self.unwrap().select(c, t, buf)
    }
    fn finalize(&mut self) {
        self.unwrap().finalize();
    }
}

impl Write for Selector {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.unwrap().write(buf);
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
