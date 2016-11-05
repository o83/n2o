
// Simple Network and Console

extern crate kernel;

use kernel::io::poll::*;
use kernel::io::tcp::*;
use kernel::reactors::server::*;
use kernel::reactors::console::*;

fn main() {
    println!("IO Server started");
    let x = std::thread::spawn(|| net());
    let y = std::thread::spawn(|| console());
    x.join();
}

fn net() {
    let addr = "127.0.0.1:8000".parse::<std::net::SocketAddr>().ok().expect("Parser Error");
    let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut net = Server::new(sock);
    net.run(&mut poll).expect("Failed to run server");
}

fn console() {
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut con = Console::new();
    con.run(&mut poll);
}
