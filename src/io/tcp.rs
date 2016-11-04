
//  TCP stream

use std;
use std::io::{self, Read, Write};
use std::net::{self, SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr};
use net2::TcpBuilder;
use io::token::Token;
use io::ready::Ready;
use io::poll::{self, Poll, Events};
use io::options::*;
use io::tcp;
use io::unix;
use std::sync::atomic::{AtomicUsize, Ordering};
use io::event::Evented;

#[derive(Debug)]
pub struct TcpStream {
    sys: unix::tcp::TcpStream,
    selector_id: SelectorId,
}

pub use std::net::Shutdown;

impl TcpStream {
    pub fn connect(addr: &SocketAddr) -> io::Result<TcpStream> {
        let sock = try!(match *addr {
            SocketAddr::V4(..) => TcpBuilder::new_v4(),
            SocketAddr::V6(..) => TcpBuilder::new_v6(),
        });
        // Required on Windows for a future `connect_overlapped` operation to be
        // executed successfully.
        if cfg!(windows) {
            try!(sock.bind(&inaddr_any(addr)));
        }
        TcpStream::connect_stream(try!(sock.to_tcp_stream()), addr)
    }

    pub fn connect_stream(stream: std::net::TcpStream,
                          addr: &SocketAddr) -> io::Result<TcpStream> {
        Ok(TcpStream {
            sys: try!(unix::tcp::TcpStream::connect(stream, addr)),
            selector_id: SelectorId::new(),
        })
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.sys.peer_addr()
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.sys.local_addr()
    }

    pub fn try_clone(&self) -> io::Result<TcpStream> {
        self.sys.try_clone().map(|s| {
            TcpStream {
                sys: s,
                selector_id: self.selector_id.clone(),
            }
        })
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.sys.shutdown(how)
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.sys.set_nodelay(nodelay)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.sys.nodelay()
    }

    pub fn set_keepalive_ms(&self, keepalive: Option<u32>) -> io::Result<()> {
        self.sys.set_keepalive_ms(keepalive)
    }

    pub fn keepalive_ms(&self) -> io::Result<Option<u32>> {
        self.sys.keepalive_ms()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.sys.set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.sys.ttl()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.sys.take_error()
    }
}

fn inaddr_any(other: &SocketAddr) -> SocketAddr {
    match *other {
        SocketAddr::V4(..) => {
            let any = Ipv4Addr::new(0, 0, 0, 0);
            let addr = SocketAddrV4::new(any, 0);
            SocketAddr::V4(addr)
        }
        SocketAddr::V6(..) => {
            let any = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0);
            let addr = SocketAddrV6::new(any, 0, 0, 0);
            SocketAddr::V6(addr)
        }
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.sys).read(buf)
    }
}

impl<'a> Read for &'a TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.sys).read(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.sys).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.sys).flush()
    }
}

impl<'a> Write for &'a TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.sys).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.sys).flush()
    }
}

impl Evented for TcpStream {
    fn register(&self, poll: &Poll, token: Token,
                interest: Ready, opts: PollOpt) -> io::Result<()> {
        try!(self.selector_id.associate_selector(poll));
        self.sys.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token,
                  interest: Ready, opts: PollOpt) -> io::Result<()> {
        self.sys.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.sys.deregister(poll)
    }
}

#[derive(Debug)]
pub struct TcpListener {
    sys: unix::tcp::TcpListener,
    selector_id: SelectorId,
}

impl TcpListener {
    pub fn bind(addr: &SocketAddr) -> io::Result<TcpListener> {
        let sock = try!(match *addr {
            SocketAddr::V4(..) => TcpBuilder::new_v4(),
            SocketAddr::V6(..) => TcpBuilder::new_v6(),
        });

        if cfg!(unix) {
            try!(sock.reuse_address(true));
        }

        try!(sock.bind(addr));

        let listener = try!(sock.listen(1024));
        Ok(TcpListener {
            sys: try!(unix::tcp::TcpListener::new(listener, addr)),
            selector_id: SelectorId::new(),
        })
    }

    pub fn from_listener(listener: net::TcpListener, addr: &SocketAddr)
                         -> io::Result<TcpListener> {
        unix::tcp::TcpListener::new(listener, addr).map(|s| {
            TcpListener {
                sys: s,
                selector_id: SelectorId::new(),
            }
        })
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        self.sys.accept().map(|(s, a)| {
            let stream = TcpStream {
                sys: s,
                selector_id: SelectorId::new(),
            };

            (stream, a)
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.sys.local_addr()
    }

    pub fn try_clone(&self) -> io::Result<TcpListener> {
        self.sys.try_clone().map(|s| {
            TcpListener {
                sys: s,
                selector_id: self.selector_id.clone(),
            }
        })
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.sys.set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.sys.ttl()
    }

    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        self.sys.set_only_v6(only_v6)
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        self.sys.only_v6()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.sys.take_error()
    }
}

impl Evented for TcpListener {
    fn register(&self, poll: &Poll, token: Token,
                interest: Ready, opts: PollOpt) -> io::Result<()> {
        try!(self.selector_id.associate_selector(poll));
        self.sys.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token,
                  interest: Ready, opts: PollOpt) -> io::Result<()> {
        self.sys.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.sys.deregister(poll)
    }
}

#[cfg(unix)]
use std::os::unix::io::{IntoRawFd, AsRawFd, FromRawFd, RawFd};

#[cfg(unix)]
impl IntoRawFd for TcpStream {
    fn into_raw_fd(self) -> RawFd {
        self.sys.into_raw_fd()
    }
}

#[cfg(unix)]
impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.sys.as_raw_fd()
    }
}

#[cfg(unix)]
impl FromRawFd for TcpStream {
    unsafe fn from_raw_fd(fd: RawFd) -> TcpStream {
        TcpStream {
            sys: FromRawFd::from_raw_fd(fd),
            selector_id: SelectorId::new(),
        }
    }
}

#[cfg(unix)]
impl IntoRawFd for TcpListener {
    fn into_raw_fd(self) -> RawFd {
        self.sys.into_raw_fd()
    }
}

#[cfg(unix)]
impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.sys.as_raw_fd()
    }
}

#[cfg(unix)]
impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: RawFd) -> TcpListener {
        TcpListener {
            sys: FromRawFd::from_raw_fd(fd),
            selector_id: SelectorId::new(),
        }
    }
}


#[derive(Debug)]
struct SelectorId {
    id: AtomicUsize,
}

impl SelectorId {
    fn new() -> SelectorId {
        SelectorId {
            id: AtomicUsize::new(0),
        }
    }

    fn associate_selector(&self, poll: &Poll) -> io::Result<()> {
        let selector_id = self.id.load(Ordering::SeqCst);

        if selector_id != 0 && selector_id != poll::selector(poll).id() {
            Err(io::Error::new(io::ErrorKind::Other, "socket already registered"))
        } else {
            self.id.store(poll::selector(poll).id(), Ordering::SeqCst);
            Ok(())
        }
    }
}

impl Clone for SelectorId {
    fn clone(&self) -> SelectorId {
        SelectorId {
            id: AtomicUsize::new(self.id.load(Ordering::SeqCst)),
        }
    }
}
