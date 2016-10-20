use std::io;
use std::net::{self, SocketAddr, Ipv4Addr, Ipv6Addr};
use std::fmt;

use abstractions::poll::Async;
use mio;

use reactors::sched::Handle;
use reactors::poll_evented::PollEvented;

pub struct UdpSocket {
    io: PollEvented<mio::udp::UdpSocket>,
}

impl UdpSocket {
    pub fn bind(addr: &SocketAddr, handle: &Handle) -> io::Result<UdpSocket> {
        let udp = try!(mio::udp::UdpSocket::bind(addr));
        UdpSocket::new(udp, handle)
    }

    fn new(socket: mio::udp::UdpSocket, handle: &Handle) -> io::Result<UdpSocket> {
        let io = try!(PollEvented::new(socket, handle));
        Ok(UdpSocket { io: io })
    }

    pub fn from_socket(socket: net::UdpSocket, handle: &Handle) -> io::Result<UdpSocket> {
        let udp = try!(mio::udp::UdpSocket::from_socket(socket));
        UdpSocket::new(udp, handle)
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.get_ref().local_addr()
    }


    pub fn poll_read(&self) -> Async<()> {
        self.io.poll_read()
    }

    pub fn poll_write(&self) -> Async<()> {
        self.io.poll_write()
    }

    pub fn send_to(&self, buf: &[u8], target: &SocketAddr) -> io::Result<usize> {
        if let Async::NotReady = self.io.poll_write() {
            return Err(mio::would_block());
        }
        match self.io.get_ref().send_to(buf, target) {
            Ok(Some(n)) => Ok(n),
            Ok(None) => {
                self.io.need_write();
                Err(mio::would_block())
            }
            Err(e) => Err(e),
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        if let Async::NotReady = self.io.poll_read() {
            return Err(mio::would_block());
        }
        match self.io.get_ref().recv_from(buf) {
            Ok(Some(n)) => Ok(n),
            Ok(None) => {
                self.io.need_read();
                Err(mio::would_block())
            }
            Err(e) => Err(e),
        }
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        self.io.get_ref().broadcast()
    }

    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.io.get_ref().set_broadcast(on)
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        self.io.get_ref().multicast_loop_v4()
    }

    pub fn set_multicast_loop_v4(&self, on: bool) -> io::Result<()> {
        self.io.get_ref().set_multicast_loop_v4(on)
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        self.io.get_ref().multicast_ttl_v4()
    }

    pub fn set_multicast_ttl_v4(&self, ttl: u32) -> io::Result<()> {
        self.io.get_ref().set_multicast_ttl_v4(ttl)
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        self.io.get_ref().multicast_loop_v6()
    }

    pub fn set_multicast_loop_v6(&self, on: bool) -> io::Result<()> {
        self.io.get_ref().set_multicast_loop_v6(on)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.io.get_ref().ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.io.get_ref().set_ttl(ttl)
    }

    pub fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.io.get_ref().join_multicast_v4(multiaddr, interface)
    }

    pub fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.io.get_ref().join_multicast_v6(multiaddr, interface)
    }

    pub fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.io.get_ref().leave_multicast_v4(multiaddr, interface)
    }

    pub fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.io.get_ref().leave_multicast_v6(multiaddr, interface)
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.io.get_ref().fmt(f)
    }
}

#[cfg(unix)]
mod sys {
    use std::os::unix::prelude::*;
    use super::UdpSocket;

    impl AsRawFd for UdpSocket {
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
    // use super::UdpSocket;
    //
    // impl AsRawHandle for UdpSocket {
    //     fn as_raw_handle(&self) -> RawHandle {
    //         self.io.get_ref().as_raw_handle()
    //     }
    // }
}
