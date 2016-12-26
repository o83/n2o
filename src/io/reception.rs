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

pub trait Selected<'a> {
    fn initial(&'a mut self, p: &'a Poll, t: usize, a: usize);
    fn select(&mut self, e: &Events) -> usize;
    fn finalize(&mut self);
}

pub struct Select<'a> {
    events: Events,
    poll: Poll,
    selectors: Vec<Box<Selected<'a>>>,
    running: bool,
}

impl<'a> Select<'a> {
    pub fn new() -> Self {
        Select {
            poll: Poll::new().unwrap(),
            events: Events::with_capacity(EVENTS_CAPACITY),
            selectors: Vec::with_capacity(SUBSCRIBERS_CAPACITY),
            running: true,
        }
    }

    pub fn poll(&'a mut self) {
        while self.running {
            self.poll.poll(&mut self.events, None).unwrap();
            for s in self.selectors.iter_mut() {
                if s.select(&self.events) == 0 {
                    self.running = false;
                    break;
                }
            }
            println!("RUNNING: {:?}", &self.running);
        }

        self.finalize();
    }

    #[inline]
    fn finalize(&mut self) {
        for s in self.selectors.iter_mut() {
            s.finalize();
        }
    }
}

pub struct Handle<'a>(UnsafeCell<Select<'a>>);

impl<'a> Handle<'a> {
    pub fn insert(&self, s: Box<Selected<'a>>) {
        let mut se = unsafe { &mut *self.0.get() };
        let ti = se.selectors.len() * POLL_SIZE;
        se.selectors.push(s);
        se.selectors.last_mut().unwrap().initial(&se.poll, ti, POLL_SIZE);
    }

    pub fn poll(&self) {
        let mut se = unsafe { &mut *self.0.get() };
        se.poll();
    }
}

pub fn handle<'a>() -> Handle<'a> {
    Handle(UnsafeCell::new(Select::new()))
}