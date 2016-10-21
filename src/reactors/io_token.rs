use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::io;
use abstractions::futures::future::BoxFuture;
use abstractions::poll::{Async, Poll};
use abstractions::streams::stream::BoxStream;
use abstractions::tasks::task;
use mio;
use reactors::sched::{Message, Remote, Handle, Direction};
use reactors::split;

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

use reactors::split::{ReadHalf, WriteHalf};

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
