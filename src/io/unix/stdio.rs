use std::prelude::v1::*;
use std::io::prelude::*;

use std::cell::{RefCell, BorrowState};
use std::fmt;
use super::lazy::Lazy;
use std::io::{self, BufReader, LineWriter};
use std::sync::{Arc, Mutex, MutexGuard};
use super::sys_stdio;
use super::remutex::{ReentrantMutex, ReentrantMutexGuard};
use std::thread::LocalKeyState;
use io::poll::{self, Poll, Events};
use io::options::PollOpt;
use io::ready::Ready;
use io::token::Token;
use io::event::Evented;
use std::os::unix::io::{RawFd, FromRawFd, IntoRawFd, AsRawFd};
use libc;
use super::fd::FileDesc;

thread_local! {
    static LOCAL_STDOUT: RefCell<Option<Box<Write + Send>>> = {
        RefCell::new(None)
    }
}

struct StdinRaw(sys_stdio::Stdin);

struct StdoutRaw(sys_stdio::Stdout);

struct StderrRaw(sys_stdio::Stderr);

fn stdin_raw() -> io::Result<StdinRaw> {
    sys_stdio::Stdin::new().map(StdinRaw)
}

fn stdout_raw() -> io::Result<StdoutRaw> {
    sys_stdio::Stdout::new().map(StdoutRaw)
}

fn stderr_raw() -> io::Result<StderrRaw> {
    sys_stdio::Stderr::new().map(StderrRaw)
}

impl Read for StdinRaw {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }
}
impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Write for StderrRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

enum Maybe<T> {
    Real(T),
    Fake,
}

impl<W: io::Write> io::Write for Maybe<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            Maybe::Real(ref mut w) => handle_ebadf(w.write(buf), buf.len()),
            Maybe::Fake => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            Maybe::Real(ref mut w) => handle_ebadf(w.flush(), ()),
            Maybe::Fake => Ok(()),
        }
    }
}

impl<R: io::Read> io::Read for Maybe<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Maybe::Real(ref mut r) => handle_ebadf(r.read(buf), 0),
            Maybe::Fake => Ok(0),
        }
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        match *self {
            Maybe::Real(ref mut r) => handle_ebadf(r.read_to_end(buf), 0),
            Maybe::Fake => Ok(0),
        }
    }
}

fn handle_ebadf<T>(r: io::Result<T>, default: T) -> io::Result<T> {
    #[cfg(windows)]
    const ERR: i32 = ::sys::c::ERROR_INVALID_HANDLE as i32;
    #[cfg(not(windows))]
    const ERR: i32 = ::libc::EBADF as i32;

    match r {
        Err(ref e) if e.raw_os_error() == Some(ERR) => Ok(default),
        r => r,
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
pub struct Stdin {
    inner: Arc<Mutex<BufReader<Maybe<StdinRaw>>>>,
}

#[stable(feature = "rust1", since = "1.0.0")]
pub struct StdinLock<'a> {
    inner: MutexGuard<'a, BufReader<Maybe<StdinRaw>>>,
}

#[stable(feature = "rust1", since = "1.0.0")]
pub fn stdin() -> Stdin {
    static INSTANCE: Lazy<Mutex<BufReader<Maybe<StdinRaw>>>> = Lazy::new(stdin_init);
    return Stdin { inner: INSTANCE.get().expect("cannot access stdin during shutdown") };

    fn stdin_init() -> Arc<Mutex<BufReader<Maybe<StdinRaw>>>> {
        let stdin = match stdin_raw() {
            Ok(stdin) => Maybe::Real(stdin),
            _ => Maybe::Fake,
        };

        // The default buffer capacity is 64k, but apparently windows
        // doesn't like 64k reads on stdin. See #13304 for details, but the
        // idea is that on windows we use a slightly smaller buffer that's
        // been seen to be acceptable.
        Arc::new(Mutex::new(if cfg!(windows) {
            BufReader::with_capacity(8 * 1024, stdin)
        } else {
            BufReader::new(stdin)
        }))
    }
}

impl Stdin {
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdinLock {
        StdinLock { inner: self.inner.lock().unwrap_or_else(|e| e.into_inner()) }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_line(buf)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Read for StdinLock<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> BufRead for StdinLock<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }
    fn consume(&mut self, n: usize) {
        self.inner.consume(n)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
pub struct Stdout {
    // FIXME: this should be LineWriter or BufWriter depending on the state of
    //        stdout (tty or not). Note that if this is not line buffered it
    //        should also flush-on-panic or some form of flush-on-abort.
    inner: Arc<ReentrantMutex<RefCell<LineWriter<Maybe<StdoutRaw>>>>>,
}

#[stable(feature = "rust1", since = "1.0.0")]
pub struct StdoutLock<'a> {
    inner: ReentrantMutexGuard<'a, RefCell<LineWriter<Maybe<StdoutRaw>>>>,
}

#[stable(feature = "rust1", since = "1.0.0")]
pub fn stdout() -> Stdout {
    static INSTANCE: Lazy<ReentrantMutex<RefCell<LineWriter<Maybe<StdoutRaw>>>>> =
        Lazy::new(stdout_init);
    return Stdout { inner: INSTANCE.get().expect("cannot access stdout during shutdown") };

    fn stdout_init() -> Arc<ReentrantMutex<RefCell<LineWriter<Maybe<StdoutRaw>>>>> {
        let stdout = match stdout_raw() {
            Ok(stdout) => Maybe::Real(stdout),
            _ => Maybe::Fake,
        };
        Arc::new(ReentrantMutex::new(RefCell::new(LineWriter::new(stdout))))
    }
}

impl Stdout {
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdoutLock {
        StdoutLock { inner: self.inner.lock().unwrap_or_else(|e| e.into_inner()) }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lock().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.lock().flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.lock().write_all(buf)
    }
    fn write_fmt(&mut self, args: fmt::Arguments) -> io::Result<()> {
        self.lock().write_fmt(args)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Write for StdoutLock<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().flush()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
pub struct Stderr {
    inner: Arc<ReentrantMutex<RefCell<Maybe<StderrRaw>>>>,
}

#[stable(feature = "rust1", since = "1.0.0")]
pub struct StderrLock<'a> {
    inner: ReentrantMutexGuard<'a, RefCell<Maybe<StderrRaw>>>,
}

#[stable(feature = "rust1", since = "1.0.0")]
pub fn stderr() -> Stderr {
    static INSTANCE: Lazy<ReentrantMutex<RefCell<Maybe<StderrRaw>>>> = Lazy::new(stderr_init);
    return Stderr { inner: INSTANCE.get().expect("cannot access stderr during shutdown") };

    fn stderr_init() -> Arc<ReentrantMutex<RefCell<Maybe<StderrRaw>>>> {
        let stderr = match stderr_raw() {
            Ok(stderr) => Maybe::Real(stderr),
            _ => Maybe::Fake,
        };
        Arc::new(ReentrantMutex::new(RefCell::new(stderr)))
    }
}

impl Stderr {
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StderrLock {
        StderrLock { inner: self.inner.lock().unwrap_or_else(|e| e.into_inner()) }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lock().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.lock().flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.lock().write_all(buf)
    }
    fn write_fmt(&mut self, args: fmt::Arguments) -> io::Result<()> {
        self.lock().write_fmt(args)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Write for StderrLock<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().flush()
    }
}

pub struct EventedFd<'a>(pub &'a RawFd);

impl<'a> Evented for EventedFd<'a> {
    fn register(&self,
                poll: &Poll,
                token: Token,
                interest: Ready,
                opts: PollOpt)
                -> io::Result<()> {
        poll::selector(poll).register(*self.0, token, interest, opts)
    }

    fn reregister(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        poll::selector(poll).reregister(*self.0, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll::selector(poll).deregister(*self.0)
    }
}

impl Evented for Stdin {
    fn register(&self,
                poll: &Poll,
                token: Token,
                interest: Ready,
                opts: PollOpt)
                -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

impl AsRawFd for Stdin {
    fn as_raw_fd(&self) -> RawFd {
        FileDesc::new(libc::STDIN_FILENO);
    }
}
