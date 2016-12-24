use std::io;
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::collections::HashMap;
use io::event::Evented;
use std::cell::UnsafeCell;

const EVENTS_CAPACITY: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 256;

pub trait React<'a>
    where Self: Sized
{
    fn react(&mut self, t: Token);
}

pub struct Reception {
    poll: Poll,
    events: Events,
    tokens: usize,
    subscribers: HashMap<Token, Box<FnMut(Token)>>,
}

impl Reception {
    pub fn new() -> Self {
        Reception {
            poll: Poll::new().unwrap(),
            events: Events::with_capacity(EVENTS_CAPACITY),
            tokens: 0,
            subscribers: HashMap::new(),
        }
    }

    pub fn register<'a, E>(&'a mut self, e: &'a E, p: PollOpt, f: Box<FnMut(Token)>) -> io::Result<Token>
        where E: Sized + Evented
    {
        let t = Token(self.tokens);
        try!(self.poll.register(e, t, Ready::readable(), p));
        self.subscribers.insert(t, f);
        self.tokens += 1;
        Ok(t)
    }

    pub fn select(&mut self) {
        let sc = UnsafeCell::new(self);
        loop {
            let s0 = unsafe { &mut *sc.get() };
            s0.poll.poll(&mut s0.events, None).unwrap();
            let s1 = unsafe { &mut *sc.get() };
            for event in s1.events.iter() {
                let t = event.token();
                let s2 = unsafe { &mut *sc.get() };
                let s3 = unsafe { &mut *sc.get() };
                let mut f = s2.subscribers.get_mut(&t).unwrap();
                f(t);
            }
        }
    }

    #[inline]
    pub fn split(&mut self) -> (&mut Self, &mut Self) {
        let f: *mut Reception = self;
        let uf: &mut Reception = unsafe { &mut *f };
        let us: &mut Reception = unsafe { &mut *f };
        (uf, us)
    }
}