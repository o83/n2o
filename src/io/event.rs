use io::poll::Poll;
use io::options::PollOpt;
use io::token::Token;
use io::ready::Ready;
use std::io::{Read, Write, Result, Error, ErrorKind};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Event {
    kind: Ready,
    token: Token,
}

impl Event {
    pub fn new(kind: Ready, token: Token) -> Event {
        Event {
            kind: kind,
            token: token,
        }
    }

    pub fn kind(&self) -> Ready {
        self.kind
    }

    pub fn token(&self) -> Token {
        self.token
    }
}

pub fn is_empty(events: Ready) -> bool {
    events.0 == 0
}

pub fn is_drop(events: Ready) -> bool {
    events.contains(Ready::drop())
}

pub fn drop() -> Ready {
    Ready::drop()
}

pub fn as_usize(events: Ready) -> usize {
    events.0
}

pub fn from_usize(events: usize) -> Ready {
    Ready(events)
}

#[allow(dead_code)]
pub fn kind_mut(event: &mut Event) -> &mut Ready {
    &mut event.kind
}

pub trait Evented {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()>;
    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()>;
    fn deregister(&self, poll: &Poll) -> Result<()>;
}
