use std::io;
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::collections::HashMap;
use io::event::Evented;

pub struct Reception<'a, E: 'a, F> {
    poll: Poll,
    events: Events,
    subscribers: Vec<(&'a mut E, F)>,
}

impl<'a, E, F> Reception<'a, E, F>
    where E: Sized + Evented,
          F: FnMut(&E)
{
    pub fn register(&'a mut self, e: &'a mut E, p: PollOpt, f: F) -> io::Result<usize> {
        let id = self.subscribers.len();
        try!(self.poll.register(e, Token(id), Ready::readable(), p));
        self.subscribers.push((e, f));
        Ok(id)
    }

    #[inline]
    pub fn split(&mut self) -> (&mut Self, &mut Self) {
        let f: *mut Reception<'a, E, F> = self;
        let uf: &mut Reception<'a, E, F> = unsafe { &mut *f };
        let us: &mut Reception<'a, E, F> = unsafe { &mut *f };
        (uf, us)
    }

    pub fn select(&mut self) {
        loop {
            self.poll.poll(&mut self.events, None).unwrap();
            let (s1, s2) = self.split();
            for event in s1.events.iter() {
                let m = s2.subscribers.get_mut(event.token().0).unwrap();
                m.1(m.0);
            }
        }
    }
}