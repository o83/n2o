use std::io::{self, Write};
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use io::event::Evented;
use reactors::selector::{Slot, Selector};
use std::time::Duration;
use std::str::from_utf8;
use handle;

const EVENTS_CAPACITY: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 16;
const BUFFER_CAPACITY: usize = 1024;

#[derive(Debug)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}

pub struct IO {
    tokens: usize,
    events: Events,
    poll: Poll,
    selectors: Vec<Selector>,
    slots: Vec<Slot>,
    running: bool,
    buf: [u8; BUFFER_CAPACITY],
    polled: usize,
}

impl<'a> IO {
    pub fn new() -> Self {
        IO {
            tokens: 0,
            poll: Poll::new().unwrap(),
            events: Events::with_capacity(EVENTS_CAPACITY),
            selectors: Vec::with_capacity(SUBSCRIBERS_CAPACITY),
            slots: Vec::with_capacity(SUBSCRIBERS_CAPACITY),
            running: true,
            buf: [0u8; BUFFER_CAPACITY],
            polled: 0,
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

    pub fn spawn(&mut self, s: Selector) -> Slot {
        let (s1, s2) = handle::split(self);
        s1.selectors.push(s);
        let slot = Slot(s2.selectors.len() - 1);
        s1.selectors.last_mut().expect("Can't retrieve a selector.").unpack().init(s2, slot);
        slot
    }

    pub fn write(&mut self, s: Slot, buf: &[u8]) -> io::Result<()> {
        self.selectors.get_mut(s.0).expect("Can't retrieve a selector.").unpack().write(buf);
        Ok(())
    }

    pub fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        for s in &mut self.selectors {
            s.write(buf);
        }
        Ok(())
    }

    #[inline]
    fn poll_if_need(&mut self) {
        if self.polled == 0 {
            self.poll.poll(&mut self.events, Some(Duration::from_millis(100))).expect("No events in poll.");
            self.polled = self.events.len();
        }
    }

    pub fn cmd(&mut self, buf: &'a [u8]) -> Option<&'a str> {
        if buf.len() == 0 {
            return None;
        }
        if buf.len() == 1 && buf[0] == 0x0A {
            self.write_all(&[0u8; 0]);
            return None;
        }
        match from_utf8(buf) {
            Ok(x) => Some(x),
            Err(x) => None,
        }
    }

    pub fn poll(&'a mut self) -> Async<(Slot, &'a [u8])> {
        self.poll_if_need();
        match self.polled {
            0 => Async::NotReady,
            id => {
                self.polled -= 1;
                let e = self.events.get(self.polled).expect("Can't retrieve an event.");
                let (s1, s2) = handle::split(self);
                let buf = &mut s1.buf;
                let slot = s1.slots.get(e.token().0).expect("Can't retrieve a slot.");
                let sel = s1.selectors.get_mut(slot.0).expect("Can't retrieve a selector.");
                match sel.unpack().select(s2, e.token(), buf) {
                    0 => Async::NotReady,
                    x => Async::Ready((Slot(slot.0), &buf[..x])),
                }
            }
        }
    }

    #[inline]
    fn finalize(&mut self) {
        for s in self.selectors.iter_mut() {
            s.unpack().finalize();
        }
    }
}