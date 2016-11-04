// #
//
// fd.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use std::io::{self, ErrorKind, Read};
use libc::{self, c_int, c_void};
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::slice::from_raw_parts_mut;

pub unsafe fn read_to_end_uninitialized(r: &mut Read, buf: &mut Vec<u8>) -> io::Result<usize> {

    let start_len = buf.len();
    buf.reserve(16);

    loop {
        if buf.len() == buf.capacity() {
            buf.reserve(1);
        }

        let buf_slice = from_raw_parts_mut(buf.as_mut_ptr().offset(buf.len() as isize),
                                           buf.capacity() - buf.len());

        match r.read(buf_slice) {
            Ok(0) => {
                return Ok(buf.len() - start_len);
            }
            Ok(n) => {
                let len = buf.len() + n;
                buf.set_len(len);
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => {
                return Err(e);
            }
        }
    }
}

pub trait AsInner<Inner: ?Sized> {
    fn as_inner(&self) -> &Inner;
}

pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

pub fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}


pub struct FileDesc {
    fd: c_int,
}

impl FileDesc {
    pub fn new(fd: c_int) -> FileDesc {
        FileDesc { fd: fd }
    }

    pub fn raw(&self) -> c_int {
        self.fd
    }

    /// Extracts the actual filedescriptor without closing it.
    pub fn into_raw(self) -> c_int {
        let fd = self.fd;
        mem::forget(self);
        fd
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = cvt(unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut c_void, buf.len()) })?;
        Ok(ret as usize)
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        unsafe fn cvt_pread64(fd: c_int,
                              buf: *mut c_void,
                              count: usize,
                              offset: i64)
                              -> io::Result<isize> {
            #[cfg(any(target_os = "linux", target_os = "emscripten"))]
            use libc::pread64;
            #[cfg(not(any(target_os = "linux", target_os = "emscripten")))]
            use libc::pread as pread64;
            cvt(pread64(fd, buf, count, offset))
        }

        unsafe {
            cvt_pread64(self.fd,
                        buf.as_mut_ptr() as *mut c_void,
                        buf.len(),
                        offset as i64)
                .map(|n| n as usize)
        }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = cvt(unsafe { libc::write(self.fd, buf.as_ptr() as *const c_void, buf.len()) })?;
        Ok(ret as usize)
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        #[cfg(target_os = "android")]
        use super::android::cvt_pwrite64;

        #[cfg(not(target_os = "android"))]
        unsafe fn cvt_pwrite64(fd: c_int,
                               buf: *const c_void,
                               count: usize,
                               offset: i64)
                               -> io::Result<isize> {
            #[cfg(any(target_os = "linux", target_os = "emscripten"))]
            use libc::pwrite64;
            #[cfg(not(any(target_os = "linux", target_os = "emscripten")))]
            use libc::pwrite as pwrite64;
            cvt(pwrite64(fd, buf, count, offset))
        }

        unsafe {
            cvt_pwrite64(self.fd,
                         buf.as_ptr() as *const c_void,
                         buf.len(),
                         offset as i64)
                .map(|n| n as usize)
        }
    }

    #[cfg(not(any(target_env = "newlib",
                  target_os = "solaris",
                  target_os = "emscripten",
                  target_os = "haiku")))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            cvt(libc::ioctl(self.fd, libc::FIOCLEX))?;
            Ok(())
        }
    }
    #[cfg(any(target_env = "newlib",
              target_os = "solaris",
              target_os = "emscripten",
              target_os = "haiku"))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            let previous = cvt(libc::fcntl(self.fd, libc::F_GETFD))?;
            cvt(libc::fcntl(self.fd, libc::F_SETFD, previous | libc::FD_CLOEXEC))?;
            Ok(())
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        unsafe {
            let previous = cvt(libc::fcntl(self.fd, libc::F_GETFL))?;
            let new = if nonblocking {
                previous | libc::O_NONBLOCK
            } else {
                previous & !libc::O_NONBLOCK
            };
            cvt(libc::fcntl(self.fd, libc::F_SETFL, new))?;
            Ok(())
        }
    }

    pub fn duplicate(&self) -> io::Result<FileDesc> {
        // We want to atomically duplicate this file descriptor and set the
        // CLOEXEC flag, and currently that's done via F_DUPFD_CLOEXEC. This
        // flag, however, isn't supported on older Linux kernels (earlier than
        // 2.6.24).
        //
        // To detect this and ensure that CLOEXEC is still set, we
        // follow a strategy similar to musl [1] where if passing
        // F_DUPFD_CLOEXEC causes `fcntl` to return EINVAL it means it's not
        // supported (the third parameter, 0, is always valid), so we stop
        // trying that.
        //
        // Also note that Android doesn't have F_DUPFD_CLOEXEC, but get it to
        // resolve so we at least compile this.
        //
        // [1]: http://comments.gmane.org/gmane.linux.lib.musl.general/2963
        #[cfg(any(target_os = "android", target_os = "haiku"))]
        use libc::F_DUPFD as F_DUPFD_CLOEXEC;
        #[cfg(not(any(target_os = "android", target_os="haiku")))]
        use libc::F_DUPFD_CLOEXEC;

        let make_filedesc = |fd| {
            let fd = FileDesc::new(fd);
            fd.set_cloexec()?;
            Ok(fd)
        };
        static TRY_CLOEXEC: AtomicBool = AtomicBool::new(!cfg!(target_os = "android"));
        let fd = self.raw();
        if TRY_CLOEXEC.load(Ordering::Relaxed) {
            match cvt(unsafe { libc::fcntl(fd, F_DUPFD_CLOEXEC, 0) }) {
                // We *still* call the `set_cloexec` method as apparently some
                // linux kernel at some point stopped setting CLOEXEC even
                // though it reported doing so on F_DUPFD_CLOEXEC.
                Ok(fd) => {
                    return Ok(if cfg!(target_os = "linux") {
                        make_filedesc(fd)?
                    } else {
                        FileDesc::new(fd)
                    })
                }
                Err(ref e) if e.raw_os_error() == Some(libc::EINVAL) => {
                    TRY_CLOEXEC.store(false, Ordering::Relaxed);
                }
                Err(e) => return Err(e),
            }
        }
        cvt(unsafe { libc::fcntl(fd, libc::F_DUPFD, 0) }).and_then(make_filedesc)
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        unsafe { read_to_end_uninitialized(self, buf) }
    }
}

impl AsInner<c_int> for FileDesc {
    fn as_inner(&self) -> &c_int {
        &self.fd
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        // Note that errors are ignored when closing a file descriptor. The
        // reason for this is that if an error occurs we don't actually know if
        // the file descriptor was closed or not, and if we retried (for
        // something like EINTR), we might close another valid file descriptor
        // (opened after we closed ours.
        let _ = unsafe { libc::close(self.fd) };
    }
}
