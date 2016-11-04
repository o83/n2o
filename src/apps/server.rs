extern crate kernel;

use std::net::SocketAddr;
use kernel::io::poll::*;
use kernel::io::tcp::*;
use kernel::io::server::*;
use kernel::io::console::Console;
use std::thread;

// Simple Network and Console server

fn main() {
    println!("IO Server started");
    let x = thread::spawn(|| {
        let addr = "127.0.0.1:8000".parse::<SocketAddr>().ok().expect("Parser Error");
        let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");
        let mut poll1 = Poll::new().expect("Failed to create Poll");
        let mut net = Server::new(sock);
        net.run(&mut poll1).expect("Failed to run server");
    });
    let mut poll2 = Poll::new().expect("Failed to create Poll");
    let mut con = Console::new();
    con.run(&mut poll2);
}
