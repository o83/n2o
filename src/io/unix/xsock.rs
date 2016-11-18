
use io::ready::Ready;
use io::options::PollOpt;
use io::token::Token;
use io::event::Event;
use io::unix::errno::Errno;
use libc::{c_int, c_void};
use libc;
use io::unix::errno;
use std::{self, mem};
use std::io::{self, Result};
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::time::Duration;

pub fn from_nix_error(err: ::io::unix::errno::Error) -> std::io::Error {
    std::io::Error::from_raw_os_error(errno::int(err.errno()))
}

pub mod ffi {
    use libc::{c_int, c_void};
    use ::io::unix::xsock::EpollEvent;
    extern "C" {
        pub fn epoll_create(size: c_int) -> c_int;
        pub fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *const EpollEvent) -> c_int;
        pub fn epoll_wait(epfd: c_int,
                          events: *mut EpollEvent,
                          max_events: c_int,
                          timeout: c_int)
                          -> c_int;
    }
}

pub fn close(fd: RawFd) -> errno::Result<()> {
    let res = unsafe { libc::close(fd) };
    Errno::result(res).map(drop)
}

const NANOS_PER_MILLI: u32 = 1_000_000;
const MILLIS_PER_SEC: u64 = 1_000;

pub fn millis(duration: Duration) -> u64 {
    let millis = (duration.subsec_nanos() + NANOS_PER_MILLI - 1) / NANOS_PER_MILLI;
    duration.as_secs().saturating_mul(MILLIS_PER_SEC).saturating_add(millis as u64)
}

static NEXT_ID: AtomicUsize = ATOMIC_USIZE_INIT;

#[derive(Debug)]
pub struct Selector {
    id: usize,
    epfd: RawFd,
}

impl Selector {
    pub fn new() -> io::Result<Selector> {
        let epfd = try!(unsafe { epoll_new() });
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed) + 1;
        Ok(Selector {
            id: id,
            epfd: epfd,
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn select(&self,
                  evts: &mut Events,
                  awakener: Token,
                  timeout: Option<Duration>)
                  -> io::Result<bool> {
        use std::{cmp, i32, slice};

        let timeout_ms = timeout.map(|to| cmp::min(millis(to), i32::MAX as u64) as i32)
            .unwrap_or(-1);

        let dst =
            unsafe { slice::from_raw_parts_mut(evts.events.as_mut_ptr(), evts.events.capacity()) };

        let cnt = try!(epoll_wait(self.epfd, dst, timeout_ms as isize).map_err(from_nix_error));

        unsafe {
            evts.events.set_len(cnt);
        }

        for i in 0..cnt {
            if evts.get(i).map(|e| e.token()) == Some(awakener) {
                evts.events.remove(i);
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn register(&self,
                    fd: RawFd,
                    token: Token,
                    interests: Ready,
                    opts: PollOpt)
                    -> io::Result<()> {
        let info = EpollEvent {
            events: ioevent_to_epoll(interests, opts),
            data: usize::from(token) as u64,
        };

        epoll_ctl(self.epfd, EpollOp::EpollCtlAdd, fd, &info).map_err(from_nix_error)
    }

    pub fn reregister(&self,
                      fd: RawFd,
                      token: Token,
                      interests: Ready,
                      opts: PollOpt)
                      -> io::Result<()> {
        let info = EpollEvent {
            events: ioevent_to_epoll(interests, opts),
            data: usize::from(token) as u64,
        };

        epoll_ctl(self.epfd, EpollOp::EpollCtlMod, fd, &info).map_err(from_nix_error)
    }

    pub fn deregister(&self, fd: RawFd) -> io::Result<()> {
        let info = EpollEvent {
            events: EpollEventKind::empty(),
            data: 0,
        };

        epoll_ctl(self.epfd, EpollOp::EpollCtlDel, fd, &info).map_err(from_nix_error)
    }
}

fn ioevent_to_epoll(interest: Ready, opts: PollOpt) -> EpollEventKind {
    let mut kind = EpollEventKind::empty();

    if interest.is_readable() {
        if opts.is_urgent() {
            kind.insert(EPOLLPRI);
        } else {
            kind.insert(EPOLLIN);
        }
    }

    if interest.is_writable() {
        kind.insert(EPOLLOUT);
    }

    if interest.is_hup() {
        kind.insert(EPOLLRDHUP);
    }

    if opts.is_edge() {
        kind.insert(EPOLLET);
    }

    if opts.is_oneshot() {
        kind.insert(EPOLLONESHOT);
    }

    if opts.is_level() {
        kind.remove(EPOLLET);
    }

    kind
}

impl Drop for Selector {
    fn drop(&mut self) {
        let _ = close(self.epfd);
    }
}

pub struct Events {
    events: Vec<EpollEvent>,
}

impl Events {
    pub fn with_capacity(u: usize) -> Events {
        Events { events: Vec::with_capacity(u) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<Event> {
        self.events.get(idx).map(|event| {
            let epoll = event.events;
            let mut kind = Ready::none();

            if epoll.contains(EPOLLIN) | epoll.contains(EPOLLPRI) {
                kind = kind | Ready::readable();
            }

            if epoll.contains(EPOLLOUT) {
                kind = kind | Ready::writable();
            }

            // EPOLLHUP - Usually means a socket error happened
            if epoll.contains(EPOLLERR) {
                kind = kind | Ready::error();
            }

            if epoll.contains(EPOLLRDHUP) | epoll.contains(EPOLLHUP) {
                kind = kind | Ready::hup();
            }

            let token = self.events[idx].data;

            Event::new(kind, Token(token as usize))
        })
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(EpollEvent {
            events: ioevent_to_epoll(event.kind(), PollOpt::empty()),
            data: usize::from(event.token()) as u64,
        });
    }
}


#[inline]
pub fn epoll_new() -> std::io::Result<RawFd> {
    let res = unsafe { ffi::epoll_create(1024) };
    Ok(res)
}

#[inline]
pub fn epoll_ctl(epfd: RawFd, op: EpollOp, fd: RawFd, event: &EpollEvent) -> errno::Result<()> {
    let res = unsafe { ffi::epoll_ctl(epfd, op as c_int, fd, event as *const EpollEvent) };

    Errno::result(res).map(drop)
}

#[inline]
pub fn epoll_wait(epfd: RawFd,
                  events: &mut [EpollEvent],
                  timeout_ms: isize)
                  -> errno::Result<usize> {
    let res = unsafe {
        ffi::epoll_wait(epfd,
                        events.as_mut_ptr(),
                        events.len() as c_int,
                        timeout_ms as c_int)
    };

    Errno::result(res).map(|r| r as usize)
}

bitflags!(
    #[repr(C)]
    flags EpollEventKind: u32 {
        const EPOLLIN = 0x001,
        const EPOLLPRI = 0x002,
        const EPOLLOUT = 0x004,
        const EPOLLRDNORM = 0x040,
        const EPOLLRDBAND = 0x080,
        const EPOLLWRNORM = 0x100,
        const EPOLLWRBAND = 0x200,
        const EPOLLMSG = 0x400,
        const EPOLLERR = 0x008,
        const EPOLLHUP = 0x010,
        const EPOLLRDHUP = 0x2000,
        const EPOLLEXCLUSIVE = 1 << 28,
        const EPOLLWAKEUP = 1 << 29,
        const EPOLLONESHOT = 1 << 30,
        const EPOLLET = 1 << 31
    }
);

#[derive(Clone, Copy)]
#[repr(C)]
pub enum EpollOp {
    EpollCtlAdd = 1,
    EpollCtlDel = 2,
    EpollCtlMod = 3,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct EpollEvent {
    pub events: EpollEventKind,
    pub data: u64,
}
