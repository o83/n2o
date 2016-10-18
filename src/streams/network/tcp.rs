use std::fmt;
use std::io::{self, Read, Write};
use std::mem;
use std::net::{self, SocketAddr, Shutdown};

use abstractions::streams::stream::Stream;
use abstractions::futures::oneshot::oneshot;
use abstractions::futures::future::Future;
use abstractions::futures::failed::failed;
use abstractions::futures::done::done;
use abstractions::poll::{Poll, Async};
use mio;

use reactors::io_token::{Io, IoFuture, IoStream};
use reactors::sched::Handle;
use reactors::poll_evented::PollEvented;

pub struct TcpListener {
    io: PollEvented<mio::tcp::TcpListener>,
}

pub struct Incoming {
    inner: IoStream<(TcpStream, SocketAddr)>,
}

impl TcpListener {
    pub fn bind(addr: &SocketAddr, handle: &Handle) -> io::Result<TcpListener> {
        let l = try!(mio::tcp::TcpListener::bind(addr));
        TcpListener::new(l, handle)
    }

    pub fn from_listener(listener: net::TcpListener,
                         addr: &SocketAddr,
                         handle: &Handle)
                         -> io::Result<TcpListener> {
        let l = try!(mio::tcp::TcpListener::from_listener(listener, addr));
        TcpListener::new(l, handle)
    }

    fn new(listener: mio::tcp::TcpListener, handle: &Handle) -> io::Result<TcpListener> {
        let io = try!(PollEvented::new(listener, handle));
        Ok(TcpListener { io: io })
    }

    pub fn poll_read(&self) -> Async<()> {
        self.io.poll_read()
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().local_addr()
    }

    pub fn incoming(self) -> Incoming {
        struct MyIncoming {
            inner: TcpListener,
        }

        impl Stream for MyIncoming {
            type Item = (mio::tcp::TcpStream, SocketAddr);
            type Error = io::Error;

            fn poll(&mut self) -> Poll<Option<Self::Item>, io::Error> {
                if let Async::NotReady = self.inner.io.poll_read() {
                    return Ok(Async::NotReady);
                }
                match self.inner.io.get_ref().accept() {
                    Ok(pair) => Ok(Async::Ready(Some(pair))),
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        self.inner.io.need_read();
                        Ok(Async::NotReady)
                    }
                    Err(e) => Err(e),
                }
            }
        }

        let remote = self.io.remote().clone();
        let stream = MyIncoming { inner: self };
        Incoming {
            inner: stream.and_then(move |(tcp, addr)| {
                    let (tx, rx) = oneshot();
                    remote.spawn(move |handle| {
                        let res = PollEvented::new(tcp, handle)
                            .map(move |io| (TcpStream { io: io }, addr));
                        tx.complete(res);
                        Ok(())
                    });
                    rx.then(|r| r.expect("shouldn't be canceled"))
                })
                .boxed(),
        }
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.io.get_ref().set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.io.get_ref().ttl()
    }

    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        self.io.get_ref().set_only_v6(only_v6)
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        self.io.get_ref().only_v6()
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.io.get_ref().fmt(f)
    }
}

impl Stream for Incoming {
    type Item = (TcpStream, SocketAddr);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, io::Error> {
        self.inner.poll()
    }
}

pub struct TcpStream {
    io: PollEvented<mio::tcp::TcpStream>,
}

pub struct TcpStreamNew {
    inner: IoFuture<TcpStream>,
}

enum TcpStreamConnect {
    Waiting(TcpStream),
    Empty,
}

impl TcpStream {
    pub fn connect(addr: &SocketAddr, handle: &Handle) -> TcpStreamNew {
        let future = match mio::tcp::TcpStream::connect(addr) {
            Ok(tcp) => TcpStream::new(tcp, handle),
            Err(e) => failed(e).boxed(),
        };
        TcpStreamNew { inner: future }
    }

    fn new(connected_stream: mio::tcp::TcpStream, handle: &Handle) -> IoFuture<TcpStream> {
        let tcp = PollEvented::new(connected_stream, handle);
        done(tcp)
            .and_then(|io| TcpStreamConnect::Waiting(TcpStream { io: io }))
            .boxed()
    }

    pub fn connect_stream(stream: net::TcpStream,
                          addr: &SocketAddr,
                          handle: &Handle)
                          -> IoFuture<TcpStream> {
        match mio::tcp::TcpStream::connect_stream(stream, addr) {
            Ok(tcp) => TcpStream::new(tcp, handle),
            Err(e) => failed(e).boxed(),
        }
    }

    pub fn poll_read(&self) -> Async<()> {
        self.io.poll_read()
    }

    pub fn poll_write(&self) -> Async<()> {
        self.io.poll_write()
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().local_addr()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().peer_addr()
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.io.get_ref().shutdown(how)
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.io.get_ref().set_nodelay(nodelay)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.io.get_ref().nodelay()
    }

    pub fn set_keepalive_ms(&self, keepalive: Option<u32>) -> io::Result<()> {
        self.io.get_ref().set_keepalive_ms(keepalive)
    }

    pub fn keepalive_ms(&self) -> io::Result<Option<u32>> {
        self.io.get_ref().keepalive_ms()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.io.get_ref().set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.io.get_ref().ttl()
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.io.read(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.io.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.io.flush()
    }
}

impl Io for TcpStream {
    fn poll_read(&mut self) -> Async<()> {
        <TcpStream>::poll_read(self)
    }

    fn poll_write(&mut self) -> Async<()> {
        <TcpStream>::poll_write(self)
    }
}

impl<'a> Read for &'a TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.io).read(buf)
    }
}

impl<'a> Write for &'a TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.io).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.io).flush()
    }
}

impl<'a> Io for &'a TcpStream {
    fn poll_read(&mut self) -> Async<()> {
        <TcpStream>::poll_read(self)
    }

    fn poll_write(&mut self) -> Async<()> {
        <TcpStream>::poll_write(self)
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.io.get_ref().fmt(f)
    }
}

impl Future for TcpStreamNew {
    type Item = TcpStream;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<TcpStream, io::Error> {
        self.inner.poll()
    }
}

impl Future for TcpStreamConnect {
    type Item = TcpStream;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<TcpStream, io::Error> {
        {
            let stream = match *self {
                TcpStreamConnect::Waiting(ref s) => s,
                TcpStreamConnect::Empty => panic!("can't poll TCP stream twice"),
            };

            if let Async::NotReady = stream.io.poll_write() {
                return Ok(Async::NotReady);
            }
            if let Some(e) = try!(stream.io.get_ref().take_error()) {
                return Err(e);
            }
        }
        match mem::replace(self, TcpStreamConnect::Empty) {
            TcpStreamConnect::Waiting(stream) => Ok(Async::Ready(stream)),
            TcpStreamConnect::Empty => panic!(),
        }
    }
}

#[cfg(unix)]
mod sys {
    use std::os::unix::prelude::*;
    use super::{TcpStream, TcpListener};

    impl AsRawFd for TcpStream {
        fn as_raw_fd(&self) -> RawFd {
            self.io.get_ref().as_raw_fd()
        }
    }

    impl AsRawFd for TcpListener {
        fn as_raw_fd(&self) -> RawFd {
            self.io.get_ref().as_raw_fd()
        }
    }
}

#[cfg(windows)]
mod sys {
    // TODO: let's land these upstream with mio and then we can add them here.
    //
    // use std::os::windows::prelude::*;
    // use super::{TcpStream, TcpListener};
    //
    // impl AsRawHandle for TcpStream {
    //     fn as_raw_handle(&self) -> RawHandle {
    //         self.io.get_ref().as_raw_handle()
    //     }
    // }
    //
    // impl AsRawHandle for TcpListener {
    //     fn as_raw_handle(&self) -> RawHandle {
    //         self.listener.io().as_raw_handle()
    //     }
    // }
}
