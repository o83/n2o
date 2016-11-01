use io::poll::Poll;
use io::options::PollOpt;
use io::token::Token;
use io::ready::Ready;

pub use std::io::{Read, Write, Result, Error, ErrorKind};

pub trait Evented {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()>;
    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()>;
    fn deregister(&self, poll: &Poll) -> Result<()>;
}
