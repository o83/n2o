use std::io::{self, Write};
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::collections::HashMap;
use io::event::Evented;
use std::cell::UnsafeCell;
use io::ws::WsServer;
use io::console::Console;
use core::borrow::BorrowMut;
use handle;

const EVENTS_CAPACITY: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 16;
const READ_BUF_SIZE: usize = 2048;

#[derive(Debug)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}

pub trait Select<'a>: Write {
    fn init(&mut self, c: &mut Core<'a>, s: Slot);
    fn select(&mut self, c: &mut Core<'a>, t: Token, buf: &mut [u8]) -> usize;
    fn finalize(&mut self);
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub struct Slot(pub usize);

pub struct Core<'a> {
    tokens: usize,
    events: Events,
    poll: Poll,
    selectors: Vec<Box<Select<'a>>>,
    slots: Vec<Slot>,
    running: bool,
    i: usize,
    buf: Vec<u8>,
}

impl<'a> Core<'a> {
    pub fn new() -> Self {
        Core {
            tokens: 0,
            poll: Poll::new().unwrap(),
            events: Events::with_capacity(EVENTS_CAPACITY),
            selectors: Vec::with_capacity(SUBSCRIBERS_CAPACITY),
            slots: Vec::with_capacity(SUBSCRIBERS_CAPACITY),
            running: true,
            i: 0,
            buf: vec![0u8;READ_BUF_SIZE],
        }
    }

    pub fn register<E>(&mut self, e: &E, s: Slot) -> Token
        where E: Evented
    {
        let t = self.tokens;
        self.poll.register(e, Token(t), Ready::readable(), PollOpt::edge());
        self.slots.push(s);
        self.tokens += 1;
        Token(t)
    }

    pub fn spawn(&mut self, s: Box<Select<'a>>) -> Slot {
        let (s1, s2) = handle::split(self);
        s1.selectors.push(s);
        let slot = Slot(s2.selectors.len() - 1);
        s1.selectors.last_mut().unwrap().init(s2, slot);
        slot
    }

    pub fn write(&mut self, s: Slot, buf: &[u8]) -> io::Result<()> {
        self.selectors.get_mut(s.0).unwrap().write(buf);
        Ok(())
    }

    #[inline]
    fn poll_if_need(&mut self) {
        if self.i == 0 {
            self.poll.poll(&mut self.events, None).unwrap();
            self.i = self.events.len();
        }
    }

    pub fn poll(&mut self) -> Async<(Slot, &[u8])> {
        self.poll_if_need();
        match self.i {
            0 => Async::NotReady,
            id => {
                self.i -= 1;
                let e = self.events.get(self.i).unwrap();
                let (s1, s2) = handle::split(self);
                let slot = s1.slots.get(e.token().0).unwrap();
                let buf = &mut s1.buf;
                let recv = s1.selectors.get_mut(slot.0).unwrap().select(s2, e.token(), buf);
                match recv {
                    0 => Async::NotReady,
                    _ => Async::Ready((Slot(slot.0), &s2.buf[..recv])),
                }
            }
        }
    }

    #[inline]
    fn finalize(&mut self) {
        for s in self.selectors.iter_mut() {
            s.finalize();
        }
    }
}