use std::io;
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::collections::HashMap;
use io::event::Evented;

pub struct Reception {
    poll: Poll,
    events: Events,
    subscribers: Vec<usize>,
}

impl Reception {
    pub fn register<E, F>(&mut self, e: &E, p: PollOpt, f: F) -> io::Result<usize>
        where E: ?Sized+Evented,
              F: FnMut(&E)
    {
        let id = self.subscribers.len();
        try!(self.poll.register(e, Token(id), Ready::readable(), p));
        // self.subscribers.push(e);
        Ok(id)
    }
}