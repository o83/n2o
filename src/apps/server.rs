extern crate kernel;

use std::net::SocketAddr;
use kernel::io::poll::*;
use kernel::io::tcp::*;
use kernel::io::server::*;

// Simple Network Server

fn main() {
    println!("IO Server started");
    let addr        = "127.0.0.1:8000".parse::<SocketAddr>().ok().expect("Parser Error");
    let sock       = TcpListener::bind(&addr).ok().expect("Failed to bind address");
    let mut poll   = Poll::new().expect("Failed to create Poll");
    let mut server = Server::new(sock);
    server.run(&mut poll).expect("Failed to run server");
}
