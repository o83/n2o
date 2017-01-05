
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
    stdout: io::Stdout,
}

impl Console {
    pub fn new() -> Self {
        Console {
            stdin: stdio::Stdin::new(),
            stdout: io::stdout(),
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
        try!(poll.deregister(&self.stdin));
        Ok(())
    }
}

impl<'a> Select<'a> for Console {
    fn init(&mut self, c: &mut Core, s: Slot) {
        write!(self.stdout, "Starting console...\n>");
        self.stdout.flush();
        c.register(self, s);
    }

    fn select(&mut self, c: &mut Core, t: Token, buf: &mut [u8]) -> usize {
        self.stdin.read(buf).unwrap()
    }

    fn finalize(&mut self) {
        write!(self.stdout, "Bye!");
        self.stdout.flush();
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
        write!(self.stdout, "{}>", String::from_utf8_lossy(buf));
        self.stdout.flush();
        Ok(1)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush();
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        Ok(())
    }
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        Ok(())
    }
}