use std::cell::RefCell;
use std::io::{self, Read, Write};
use reactors::io_token::Io;
use abstractions::poll::Async;
use abstractions::tasks::task_rc::TaskRc;

pub struct ReadHalf<T> {
    handle: TaskRc<RefCell<T>>,
}

pub struct WriteHalf<T> {
    handle: TaskRc<RefCell<T>>,
}

pub fn split<T: Io>(t: T) -> (ReadHalf<T>, WriteHalf<T>) {
    let rc = TaskRc::new(RefCell::new(t));
    (ReadHalf { handle: rc.clone() }, WriteHalf { handle: rc })
}

impl<T: Io> ReadHalf<T> {
    pub fn poll_read(&mut self) -> Async<()> {
        self.handle.with(|t| t.borrow_mut().poll_read())
    }
}

impl<T: Io> WriteHalf<T> {
    pub fn poll_write(&mut self) -> Async<()> {
        self.handle.with(|t| t.borrow_mut().poll_write())
    }
}

impl<T: Read> Read for ReadHalf<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.handle.with(|t| t.borrow_mut().read(buf))
    }
}

impl<T: Write> Write for WriteHalf<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.handle.with(|t| t.borrow_mut().write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.handle.with(|t| t.borrow_mut().flush())
    }
}
