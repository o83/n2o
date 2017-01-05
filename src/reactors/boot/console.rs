
// Pretty simple mio-based terminal by Anton

use std::io::{self, ErrorKind, Error, Read, Write};
use io::token::Token;
use io::ready::Ready;
use io::poll::*;
use io::options::*;
use io::stdio;
use io::event::Evented;
use reactors::boot::reactor::{Select, Core, Slot};
use std::fmt::Arguments;

pub struct Console {
    stdin: stdio::Stdin,
}

impl Console {
    pub fn new() -> Self {
        Console { stdin: stdio::Stdin::new() }
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
        try!(poll.deregister(&self.stdin));
        Ok(())
    }
}

impl<'a> Select<'a> for Console {
    fn init(&mut self, c: &mut Core<'a>, s: Slot) {
        println!("Starting console...");
        c.register(self, s);
    }

    fn select(&mut self, c: &mut Core<'a>, t: Token, buf: &mut Vec<u8>) {
        self.stdin.read(buf);
    }

    fn finalize(&mut self) {
        println!("Bye!");
    }
}

impl Read for Console {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.stdin.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.stdin.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.stdin.read_exact(buf)
    }
}

impl Write for Console {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        println!("{}", String::from_utf8_lossy(buf));
        Ok(1)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        Ok(())
    }
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        Ok(())
    }
}
