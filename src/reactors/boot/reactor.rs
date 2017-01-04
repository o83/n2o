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
use ptr::handle;

const EVENTS_CAPACITY: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 16;
const READ_BUF_SIZE: usize = 256;

#[derive(Debug)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}

pub trait Boil<'a>: Write {
    fn init(&mut self, c: &mut Core<'a>);
    fn select(&mut self, c: &mut Core<'a>, t: Token, buf: &mut Vec<u8>);
    fn finalize(&mut self);
}

#[derive(Debug)]
pub struct BoilId(usize);

pub struct Core<'a> {
    tokens: usize,
    events: Events,
    poll: Poll,
    boils: Vec<Box<Boil<'a>>>,
    running: bool,
}

impl<'a> Core<'a> {
    pub fn new() -> Self {
        Core {
            tokens: 0,
            poll: Poll::new().unwrap(),
            events: Events::with_capacity(EVENTS_CAPACITY),
            boils: Vec::with_capacity(SUBSCRIBERS_CAPACITY),
            running: true,
        }
    }

    pub fn register<E>(&mut self, e: &E) -> Token
        where E: Evented
    {
        self.poll.register(e, Token(self.tokens), Ready::readable(), PollOpt::edge());
        self.tokens += 1;
        Token(self.tokens)
    }

    pub fn spawn(&mut self, s: Box<Boil<'a>>) -> BoilId {
        let (s1, s2) = handle::split(self);
        s1.boils.push(s);
        s1.boils.last_mut().unwrap().init(s2);
        BoilId(s2.boils.len() - 1)
    }

    pub fn write(&mut self, b: BoilId, buf: &[u8]) -> io::Result<()> {
        self.boils.get_mut(b.0).unwrap().write(buf);
        Ok(())
    }

    #[inline]
    fn finalize(&mut self) {
        for s in self.boils.iter_mut() {
            s.finalize();
        }
    }
}

pub struct CoreIterator<'a> {
    c: Core<'a>,
    i: usize,
}

impl<'a> CoreIterator<'a> {
    #[inline]
    fn poll_if_need(&mut self) {
        if self.i == 0 {
            self.c.poll.poll(&mut self.c.events, None).unwrap();
            self.i = self.c.events.len();
        }
    }
}

impl<'a> IntoIterator for Core<'a> {
    type Item = Async<(BoilId, String)>;
    type IntoIter = CoreIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CoreIterator { c: self, i: 0 }
    }
}

impl<'a> Iterator for CoreIterator<'a> {
    type Item = Async<(BoilId, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.poll_if_need();
        match self.i {
            0 => Some(Async::NotReady),
            id => {
                let e = self.c.events.get(id - 1).unwrap();
                let (s1, s2) = handle::split(&mut self.c);
                let mut buf = vec![0u8;READ_BUF_SIZE];
                println!("Event id {:?} token: {:?}", id, e.token());
                s1.boils.get_mut(e.token().0).unwrap().select(s2, e.token(), &mut buf);
                let s = unsafe { String::from_utf8_unchecked(buf) };
                self.i -= 1;
                Some(Async::Ready((BoilId(e.token().0 + 1), s)))
            }
        }
    }
}