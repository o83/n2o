
use std::net;
use std::io::{self, ErrorKind, Error};
use network::transport::{Acceptor, Pipe, Transport, Destination};
use network::tcp::pipe::{AsyncPipe, TcpPipeStub};
use network::tcp::acceptor::TcpAcceptor;
use mio::tcp::{TcpListener, TcpStream};
use core::str::FromStr;

pub struct Tcp;

impl Tcp {
    fn connect(&self, addr: &net::SocketAddr, dest: &Destination) -> io::Result<Box<Pipe>> {
        let stream = try!(TcpStream::connect(addr));
        try!(stream.set_nodelay(dest.tcp_no_delay));
        let stub = TcpPipeStub::new(stream, dest.recv_max_size);
        let pipe = box AsyncPipe::new(stub, dest.pids);
        Ok(pipe)
    }
    fn bind(&self, addr: &net::SocketAddr, dest: &Destination) -> io::Result<Box<Acceptor>> {
        let listener = try!(TcpListener::bind(addr));
        let acceptor = box TcpAcceptor::new(listener, dest);
        Ok(acceptor)
    }
}

impl Transport for Tcp {
    fn connect(&self, dest: &Destination) -> io::Result<Box<Pipe>> {
        match net::SocketAddr::from_str(dest.addr) {
            Ok(addr) => self.connect(&addr, dest),
            Err(_) => Err(Error::new(ErrorKind::Other, "Address Error")),
        }
    }

    fn bind(&self, dest: &Destination) -> io::Result<Box<Acceptor>> {
        match net::SocketAddr::from_str(dest.addr) {
            Ok(addr) => self.bind(&addr, dest),
            Err(_) => Err(Error::new(ErrorKind::Other, "Address Error")),
        }
    }
}
