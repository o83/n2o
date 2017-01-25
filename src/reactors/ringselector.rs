use io::event::Evented;
use io::poll::Poll;
use io::options::PollOpt;
use io::ready::Ready;
use io::token::Token;
use std::io::{self, Read, Write};
use std::slice;
use std::mem;
use reactors::selector::{Select, Slot, Async, Pool};
use reactors::system::IO;
use intercore::message::Message;
use queues::publisher::Subscriber;
use io::notify::Notify;
use std::rc::Rc;

pub struct RingLock<'a, T: 'a> {
    pub buf: &'a [T],
    pub sub: &'a Subscriber<T>,
}

impl<'a, T> RingLock<'a, T> {
    pub fn new(buf: &'a [T], sub: &'a Subscriber<T>) -> Self {
        RingLock {
            buf: buf,
            sub: sub,
        }
    }
}

impl<'a, T> Drop for RingLock<'a, T> {
    fn drop(&mut self) {
        self.sub.commit();
    }
}

pub struct RingSelector<T> {
    inner: Rc<Subscriber<T>>,
    notify: Rc<Notify>,
}

impl<T> RingSelector<T> {
    pub fn new(a: Rc<Subscriber<T>>, notify: Rc<Notify>) -> Self {
        RingSelector {
            inner: a,
            notify: notify,
        }
    }
}


impl<T> Evented for RingSelector<T> {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        self.notify.register(poll, token, interest, opts);
        Ok(())
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        self.notify.reregister(poll, token, interest, opts);
        Ok(())
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.notify.deregister(poll);
        Ok(())
    }
}

impl<'a> Select<'a> for RingSelector<Message> {
    fn init(&mut self, io: &mut IO, s: Slot) {
        io.register(self, s);
    }
    fn select(&'a mut self, io: &mut IO, t: Token) -> Async<Pool<'a>> {
        self.notify.wait();
        match self.inner.recv_all() {
            Some(v) => Async::Ready(Pool::Msg(RingLock::new(v, &self.inner))),
            _ => Async::NotReady,
        }
    }
    fn finalize(&mut self) {}
}

fn copy_block_memory<T>(src: *const T, dst: &mut [u8]) -> usize {
    let src = src as *const u8;
    let size = mem::size_of::<T>();
    unsafe {
        let bytes = slice::from_raw_parts(src, size);
        dst[..size].clone_from_slice(bytes);
    }
    println!("{:?}", size);
    size
}

impl<T> Read for RingSelector<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.notify.wait();
        match self.inner.recv() {
            Some(v) => {
                let sz = copy_block_memory::<T>(v as *const T, buf);
                self.inner.commit();
                Ok(sz)
            }
            _ => Ok(0),
        }
    }
}

impl<T> Write for RingSelector<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        //(&self.inner).write(buf)
        Ok(1)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.commit();
        Ok(())
    }
}