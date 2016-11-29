
use libc::{c_int, c_void, c_char};
use std::ffi::{CStr, OsStr};
use libc;
use std::{self, fmt};
use core;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Sys(Errno),
    InvalidPath,
}

impl Error {
    pub fn from_errno(errno: Errno) -> Error {
        Error::Sys(errno)
    }

    pub fn last() -> Error {
        Error::Sys(Errno::last())
    }

    pub fn invalid_argument() -> Error {
        Error::Sys(Errno(22)) // EINVAL
    }

    pub fn errno(&self) -> Errno {
        match *self {
            Error::Sys(errno) => errno,
            Error::InvalidPath => Errno(22), // EINVAL
        }
    }
}

impl From<Errno> for Error {
    fn from(errno: Errno) -> Error {
        Error::from_errno(errno)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidPath => "Invalid path",
            &Error::Sys(ref errno) => errno.desc(),
        }
    }
}

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct Errno(pub c_int);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::InvalidPath => write!(f, "Invalid path"),
            &Error::Sys(errno) => write!(f, "{:?}: {}", errno, errno.desc()),
        }
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [0 as c_char; 1024];
        unsafe {
            if strerror_r(self.0, buf.as_mut_ptr(), buf.len() as libc::size_t) < 0 {
                let Errno(fm_err) = errno();
                if fm_err != libc::ERANGE {
                    return write!(fmt,
                                  "OS Error {} (strerror_r returned error {})",
                                  self.0,
                                  fm_err);
                }
            }
        }
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr()) };
        fmt.write_str(&String::from_utf8_lossy(c_str.to_bytes()))
    }
}

pub fn errno() -> Errno {
    unsafe { Errno(*errno_location()) }
}

pub fn set_errno(Errno(errno): Errno) {
    unsafe {
        *errno_location() = errno;
    }
}

extern "C" {
    #[cfg_attr(any(target_os = "macos",
                   target_os = "ios",
                   target_os = "freebsd"),
               link_name = "__error")]
    #[cfg_attr(target_os = "linux",
               link_name = "__errno_location")]
    fn errno_location() -> *mut c_int;

  #[cfg_attr(target_os = "linux", link_name = "__xpg_strerror_r")]
    fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: libc::size_t) -> c_int;

}

pub trait ErrnoSentinel: Sized {
    fn sentinel() -> Self;
}

impl ErrnoSentinel for isize {
    fn sentinel() -> Self {
        -1
    }
}

impl ErrnoSentinel for i32 {
    fn sentinel() -> Self {
        -1
    }
}

impl ErrnoSentinel for i64 {
    fn sentinel() -> Self {
        -1
    }
}

pub fn sys_errno() -> i32 {
    unsafe { (*errno_location()) as i32 }
}

pub fn last() -> Errno {
    Errno(sys_errno())
}

unsafe fn clear() -> () {
    *errno_location() = 0;
}


pub fn int(errno: Errno) -> i32 {
    return errno.0;
}

pub enum ErrConst {
    UnknownErrno = 0,
    EPERM = 1,
    ENOENT = 2,
    ESRCH = 3,
    EINTR = 4,
    EIO = 5,
    ENXIO = 6,
    E2BIG = 7,
    ENOEXEC = 8,
    EBADF = 9,
    ECHILD = 10,
    EDEADLK = 11,
    ENOMEM = 12,
    EACCES = 13,
    EFAULT = 14,
    ENOTBLK = 15,
    EBUSY = 16,
    EEXIST = 17,
    EXDEV = 18,
    ENODEV = 19,
    ENOTDIR = 20,
    EISDIR = 21,
    EINVAL = 22,
    ENFILE = 23,
    EMFILE = 24,
    ENOTTY = 25,
    ETXTBSY = 26,
    EFBIG = 27,
    ENOSPC = 28,
    ESPIPE = 29,
    EROFS = 30,
    EMLINK = 31,
    EPIPE = 32,
    EDOM = 33,
    ERANGE = 34,
    EAGAIN = 35,
    EINPROGRESS = 36,
    EALREADY = 37,
    ENOTSOCK = 38,
    EDESTADDRREQ = 39,
    EMSGSIZE = 40,
    EPROTOTYPE = 41,
    ENOPROTOOPT = 42,
    EPROTONOSUPPORT = 43,
    ESOCKTNOSUPPORT = 44,
    // ENOTSUP = 45,
    EPFNOSUPPORT = 46,
    EAFNOSUPPORT = 47,
    EADDRINUSE = 48,
    EADDRNOTAVAIL = 49,
    ENETDOWN = 50,
    ENETUNREACH = 51,
    ENETRESET = 52,
    ECONNABORTED = 53,
    ECONNRESET = 54,
    ENOBUFS = 55,
    EISCONN = 56,
    ENOTCONN = 57,
    ESHUTDOWN = 58,
    ETOOMANYREFS = 59,
    ETIMEDOUT = 60,
    ECONNREFUSED = 61,
    ELOOP = 62,
    ENAMETOOLONG = 63,
    EHOSTDOWN = 64,
    EHOSTUNREACH = 65,
    ENOTEMPTY = 66,
    ENOLCK = 77,
    ENOSYS = 78,
    ENOMSG = 83,
    EIDRM = 82,
    EMAXIM = 10000000,
}

pub fn desc(errno: Errno) -> &'static str {
    let m: ErrConst = unsafe { core::mem::transmute::<i32, ErrConst>(errno.0) };
    match m {
        ErrConst::UnknownErrno => "Unknown errno",
        ErrConst::EPERM => "Operation not permitted",
        ErrConst::ENOENT => "No such file or directory",
        ErrConst::ESRCH => "No such process",
        ErrConst::EINTR => "Interrupted system call",
        ErrConst::EIO => "I/O error",
        ErrConst::ENXIO => "No such device or address",
        ErrConst::E2BIG => "Argument list too long",
        ErrConst::ENOEXEC => "Exec format error",
        ErrConst::EBADF => "Bad file number",
        ErrConst::ECHILD => "No child processes",
        ErrConst::EAGAIN => "Try again",
        ErrConst::ENOMEM => "Out of memory",
        ErrConst::EACCES => "Permission denied",
        ErrConst::EFAULT => "Bad address",
        ErrConst::ENOTBLK => "Block device required",
        ErrConst::EBUSY => "Device or resource busy",
        ErrConst::EEXIST => "File exists",
        ErrConst::EXDEV => "Cross-device link",
        ErrConst::ENODEV => "No such device",
        ErrConst::ENOTDIR => "Not a directory",
        ErrConst::EISDIR => "Is a directory",
        ErrConst::EINVAL => "Invalid argument",
        ErrConst::ENFILE => "File table overflow",
        ErrConst::EMFILE => "Too many open files",
        ErrConst::ENOTTY => "Not a typewriter",
        ErrConst::ETXTBSY => "Text file busy",
        ErrConst::EFBIG => "File too large",
        ErrConst::ENOSPC => "No space left on device",
        ErrConst::ESPIPE => "Illegal seek",
        ErrConst::EROFS => "Read-only file system",
        ErrConst::EMLINK => "Too many links",
        ErrConst::EPIPE => "Broken pipe",
        ErrConst::EDOM => "Math argument out of domain of func",
        ErrConst::ERANGE => "Math result not representable",
        ErrConst::EDEADLK => "Resource deadlock would occur",
        ErrConst::ENAMETOOLONG => "File name too long",
        ErrConst::ENOLCK => "No record locks available",
        ErrConst::ENOSYS => "Function not implemented",
        ErrConst::ENOTEMPTY => "Directory not empty",
        ErrConst::ELOOP => "Too many symbolic links encountered",
        ErrConst::ENOMSG => "No message of desired type",
        ErrConst::EIDRM => "Identifier removed",
        ErrConst::EINPROGRESS => "Operation now in progress",
        ErrConst::EALREADY => "Operation already in progress",
        ErrConst::ENOTSOCK => "Socket operation on non-socket",
        ErrConst::EDESTADDRREQ => "Destination address required",
        ErrConst::EMSGSIZE => "Message too long",
        ErrConst::EPROTOTYPE => "Protocol wrong type for socket",
        ErrConst::ENOPROTOOPT => "Protocol not available",
        ErrConst::EPROTONOSUPPORT => "Protocol not supported",
        ErrConst::ESOCKTNOSUPPORT => "Socket type not supported",
        ErrConst::EPFNOSUPPORT => "Protocol family not supported",
        ErrConst::EAFNOSUPPORT => "Address family not supported by protocol",
        ErrConst::EADDRINUSE => "Address already in use",
        ErrConst::EADDRNOTAVAIL => "Cannot assign requested address",
        ErrConst::ENETDOWN => "Network is down",
        ErrConst::ENETUNREACH => "Network is unreachable",
        ErrConst::ENETRESET => "Network dropped connection because of reset",
        ErrConst::ECONNABORTED => "Software caused connection abort",
        ErrConst::ECONNRESET => "Connection reset by peer",
        ErrConst::ENOBUFS => "No buffer space available",
        ErrConst::EISCONN => "Transport endpoint is already connected",
        ErrConst::ENOTCONN => "Transport endpoint is not connected",
        ErrConst::ESHUTDOWN => "Cannot send after transport endpoint shutdown",
        ErrConst::ETOOMANYREFS => "Too many references: cannot splice",
        ErrConst::ETIMEDOUT => "Connection timed out",
        ErrConst::ECONNREFUSED => "Connection refused",
        ErrConst::EHOSTDOWN => "Host is down",
        ErrConst::EHOSTUNREACH => "No route to host",
        ErrConst::EMAXIM => "",
    }

}

impl Errno {
    pub fn last() -> Self {
        last()
    }

    pub fn desc(self) -> &'static str {
        desc(self)
    }

    pub unsafe fn clear() -> () {
        clear()
    }

    pub fn result<S: ErrnoSentinel + PartialEq<S>>(value: S) -> Result<S> {
        if value == S::sentinel() {
            Err(Error::Sys(Self::last()))
        } else {
            Ok(value)
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
