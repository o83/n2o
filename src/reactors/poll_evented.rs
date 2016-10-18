use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use abstractions::poll::Async;
use mio;
use reactors::io_token::Io;
use reactors::sched::{Handle, Remote};
use reactors::io_token::IoToken;

pub struct PollEvented<E> {
    token: IoToken,
    handle: Remote,
    readiness: AtomicUsize,
    io: E,
}

impl<E: mio::Evented> PollEvented<E> {
    pub fn new(io: E, handle: &Handle) -> io::Result<PollEvented<E>> {
        Ok(PollEvented {
            token: try!(IoToken::new(&io, handle)),
            handle: handle.remote().clone(),
            readiness: AtomicUsize::new(0),
            io: io,
        })
    }
}

impl<E> PollEvented<E> {
    pub fn poll_read(&self) -> Async<()> {
        if self.readiness.load(Ordering::SeqCst) & 1 != 0 {
            return Async::Ready(());
        }
        self.readiness.fetch_or(self.token.take_readiness(), Ordering::SeqCst);
        if self.readiness.load(Ordering::SeqCst) & 1 != 0 {
            Async::Ready(())
        } else {
            self.token.schedule_read(&self.handle);
            Async::NotReady
        }
    }

    pub fn poll_write(&self) -> Async<()> {
        if self.readiness.load(Ordering::SeqCst) & 2 != 0 {
            return Async::Ready(());
        }
        self.readiness.fetch_or(self.token.take_readiness(), Ordering::SeqCst);
        if self.readiness.load(Ordering::SeqCst) & 2 != 0 {
            Async::Ready(())
        } else {
            self.token.schedule_write(&self.handle);
            Async::NotReady
        }
    }


    pub fn need_read(&self) {
        self.readiness.fetch_and(!1, Ordering::SeqCst);
        self.token.schedule_read(&self.handle)
    }

    pub fn need_write(&self) {
        self.readiness.fetch_and(!2, Ordering::SeqCst);
        self.token.schedule_write(&self.handle)
    }

    pub fn remote(&self) -> &Remote {
        &self.handle
    }


    pub fn get_ref(&self) -> &E {
        &self.io
    }

    pub fn get_mut(&mut self) -> &mut E {
        &mut self.io
    }
}

impl<E: Read> Read for PollEvented<E> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Async::NotReady = self.poll_read() {
            return Err(mio::would_block());
        }
        let r = self.get_mut().read(buf);
        if is_wouldblock(&r) {
            self.need_read();
        }
        return r;
    }
}

impl<E: Write> Write for PollEvented<E> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Async::NotReady = self.poll_write() {
            return Err(mio::would_block());
        }
        let r = self.get_mut().write(buf);
        if is_wouldblock(&r) {
            self.need_write();
        }
        return r;
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Async::NotReady = self.poll_write() {
            return Err(mio::would_block());
        }
        let r = self.get_mut().flush();
        if is_wouldblock(&r) {
            self.need_write();
        }
        return r;
    }
}

impl<E: Read + Write> Io for PollEvented<E> {
    fn poll_read(&mut self) -> Async<()> {
        <PollEvented<E>>::poll_read(self)
    }

    fn poll_write(&mut self) -> Async<()> {
        <PollEvented<E>>::poll_write(self)
    }
}

impl<'a, E> Read for &'a PollEvented<E>
    where &'a E: Read
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Async::NotReady = self.poll_read() {
            return Err(mio::would_block());
        }
        let r = self.get_ref().read(buf);
        if is_wouldblock(&r) {
            self.need_read();
        }
        return r;
    }
}

impl<'a, E> Write for &'a PollEvented<E>
    where &'a E: Write
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Async::NotReady = self.poll_write() {
            return Err(mio::would_block());
        }
        let r = self.get_ref().write(buf);
        if is_wouldblock(&r) {
            self.need_write();
        }
        return r;
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Async::NotReady = self.poll_write() {
            return Err(mio::would_block());
        }
        let r = self.get_ref().flush();
        if is_wouldblock(&r) {
            self.need_write();
        }
        return r;
    }
}

impl<'a, E> Io for &'a PollEvented<E>
    where &'a E: Read + Write
{
    fn poll_read(&mut self) -> Async<()> {
        <PollEvented<E>>::poll_read(self)
    }

    fn poll_write(&mut self) -> Async<()> {
        <PollEvented<E>>::poll_write(self)
    }
}

fn is_wouldblock<T>(r: &io::Result<T>) -> bool {
    match *r {
        Ok(_) => false,
        Err(ref e) => e.kind() == io::ErrorKind::WouldBlock,
    }
}

impl<E> Drop for PollEvented<E> {
    fn drop(&mut self) {
        self.token.drop_source(&self.handle);
    }
}
