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
    subscribers: Vec<(&'a E, F)>,
}

impl<'a, E, F> Reception<'a, E, F>
    where E: Sized + Evented,
          F: FnMut(&E)
{
    pub fn register(&'a mut self, e: &'a E, p: PollOpt, f: F) -> io::Result<usize> {
        let id = self.subscribers.len();
        try!(self.poll.register(e, Token(id), Ready::readable(), p));
        // self.subscribers.push(e);
        Ok(id)
    }
}