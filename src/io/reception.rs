use std::io;
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::collections::HashMap;
use io::event::Evented;
use std::cell::UnsafeCell;

const POLL_SIZE: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 256;

pub trait Selected<'a>
    where Self: Sized
{
    fn initial(&'a mut self, p: &'a Poll, t: usize, a: usize);
    fn select(&mut self, e: &Events);
}

pub struct Select<S> {
    events: Events,
    poll: Poll,
    selectors: Vec<S>,
}

impl<'a, S> Select<S>
    where S: Selected<'a> + 'a
{
    pub fn new() -> Self {
        Select {
            poll: Poll::new().unwrap(),
            events: Events::with_capacity(1024),
            selectors: Vec::with_capacity(SUBSCRIBERS_CAPACITY),
        }
    }

    pub fn insert(&'a mut self, s: S) {
        let ti = self.selectors.len() * POLL_SIZE;
        self.selectors.push(s);
        self.selectors.last_mut().unwrap().initial(&self.poll, ti, POLL_SIZE);
    }

    pub fn poll(&mut self) {
        loop {
            self.poll.poll(&mut self.events, None).unwrap();
            for s in self.selectors.iter_mut() {
                s.select(&self.events);
            }
        }
    }
}