extern crate mio;
extern crate slab;
extern crate core;

mod server;
mod connection;
use std::net::SocketAddr;
use mio::*;
use mio::tcp::*;
use server::*;

fn main() {
    let addr = "127.0.0.1:8000".parse::<SocketAddr>()
        .ok().expect("Failed to parse host:port string");
    let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut server = Server::new(sock);
    server.run(&mut poll).expect("Failed to run server");
}
