
// Pretty simple mio-based terminal by Anton

use std::io::{self, ErrorKind, Error, Read};
use io::token::Token;
use io::ready::Ready;
use io::poll::*;
use io::options::*;
use io::stdio;
use io::event::Evented;

pub struct Console {
    stdin: stdio::Stdin,
    token: Token,
}

impl Console {
    pub fn new() -> Self {
        Console {
            stdin: stdio::Stdin::new(),
            token: Token(0),
        }
    }
}

impl Evented for Console {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        try!(poll.register(&self.stdin, token, interest, opts));
        Ok(())
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        try!(poll.reregister(&self.stdin, token, interest, opts));
        Ok(())
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        Ok(())
    }
}
