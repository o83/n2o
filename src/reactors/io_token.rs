use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::io::{self, Read, Write};
use abstractions::futures::future::Future;
use abstractions::futures::future::BoxFuture;
use abstractions::poll::{Async, Poll};
use abstractions::streams::stream::BoxStream;
use abstractions::tasks::task;
use mio;
use reactors::sched::{Message, Remote, Handle, Direction};
use reactors::split::{self, ReadHalf, WriteHalf};

pub type IoFuture<T> = BoxFuture<T, io::Error>;
pub type IoStream<T> = BoxStream<T, io::Error>;

pub struct IoToken {
    token: usize,
    readiness: Arc<AtomicUsize>,
}

impl IoToken {
    pub fn new(source: &mio::Evented, handle: &Handle) -> io::Result<IoToken> {
        match handle.inner.upgrade() {
            Some(inner) => {
                let (ready, token) = try!(inner.borrow_mut().add_source(source));
                Ok(IoToken {
                    token: token,
                    readiness: ready,
                })
            }
            None => Err(io::Error::new(io::ErrorKind::Other, "event loop gone")),
        }
    }

    pub fn take_readiness(&self) -> usize {
        self.readiness.swap(0, Ordering::SeqCst)
    }

    pub fn schedule_read(&self, handle: &Remote) {
        handle.send(Message::Schedule(self.token, task::park(), Direction::Read));
    }

    pub fn schedule_write(&self, handle: &Remote) {
        handle.send(Message::Schedule(self.token, task::park(), Direction::Write));
    }

    pub fn drop_source(&self, handle: &Remote) {
        handle.send(Message::DropSource(self.token));
    }
}


#[macro_export]
macro_rules! try_nb {
    ($e:expr) => (match $e {
        Ok(t) => t,
        Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => {
            return Ok(::abstractions::poll::Async::NotReady)
        }
        Err(e) => return Err(e.into()),
    })
}


pub trait Io: io::Read + io::Write {
    fn poll_read(&mut self) -> Async<()> {
        Async::Ready(())
    }
    fn poll_write(&mut self) -> Async<()> {
        Async::Ready(())
    }
    fn split(self) -> (ReadHalf<Self>, WriteHalf<Self>)
        where Self: Sized
    {
        split::split(self)
    }
}

pub trait FramedIo {
    type In;
    type Out;
    fn poll_read(&mut self) -> Async<()>;
    fn read(&mut self) -> Poll<Self::Out, io::Error>;
    fn poll_write(&mut self) -> Async<()>;
    fn write(&mut self, req: Self::In) -> Poll<(), io::Error>;
    fn flush(&mut self) -> Poll<(), io::Error>;
}

pub struct Copy<R, W> {
    reader: R,
    read_done: bool,
    writer: W,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: Box<[u8]>,
}

pub fn copy<R, W>(reader: R, writer: W) -> Copy<R, W>
    where R: Read,
          W: Write,
{
    Copy {
        reader: reader,
        read_done: false,
        writer: writer,
        amt: 0,
        pos: 0,
        cap: 0,
        buf: Box::new([0; 2048]),
    }
}

impl<R, W> Future for Copy<R, W>
    where R: Read,
          W: Write,
{
    type Item = u64;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<u64, io::Error> {
        loop {
            // If our buffer is empty, then we need to read some data to
            // continue.
            if self.pos == self.cap && !self.read_done {
                let n = try_nb!(self.reader.read(&mut self.buf));
                if n == 0 {
                    self.read_done = true;
                } else {
                    self.pos = 0;
                    self.cap = n;
                }
            }

            // If our buffer has some data, let's write it out!
            while self.pos < self.cap {
                let i = try_nb!(self.writer.write(&self.buf[self.pos..self.cap]));
                self.pos += i;
                self.amt += i as u64;
            }

            // If we've written al the data and we've seen EOF, flush out the
            // data and finish the transfer.
            // done with the entire transfer.
            if self.pos == self.cap && self.read_done {
                try_nb!(self.writer.flush());
                return Ok(self.amt.into())
            }
        }
    }
}
