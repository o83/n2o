use std::io;
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::collections::HashMap;
use io::event::Evented;
use std::cell::UnsafeCell;
use io::ws::WsServer;
use io::console::Console;

const POLL_SIZE: usize = 1024;
const EVENTS_CAPACITY: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 16;

pub trait Boil<'a> {
    fn initial(&'a mut self, p: &'a Poll, t: usize, a: usize);
    fn select(&mut self, e: &Events) -> usize;
    fn finalize(&mut self);
}

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
        where E: Evented + Sized
    {
        self.poll.register(e, Token(0), Ready::readable(), PollOpt::edge());
        self.tokens += 1;
        Token(self.tokens)
    }

    pub fn spawn(&mut self, s: Box<Boil<'a>>) {
        self.boils.push(s);
        // self.boils.last_mut().unwrap().initial(&self.poll, ti, POLL_SIZE);
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
    t: Token,
}

impl<'a> IntoIterator for Core<'a> {
    type Item = Token;
    type IntoIter = CoreIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CoreIterator {
            c: self,
            t: Token(0),
        }
    }
}

impl<'a> Iterator for CoreIterator<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        // while self.running {
        //     self.poll.poll(&mut self.events, None).unwrap();
        //     for s in self.boils.iter_mut() {
        //         if s.select(&self.events) == 0 {
        //             self.running = false;
        //             break;
        //         }
        //     }
        // }
        // self.finalize();
        Some(Token(0))
    }
}