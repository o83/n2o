extern crate kernel;
use kernel::reactors::boot::reactor::{Async, Core};
use kernel::reactors::boot::console::Console;
use kernel::reactors::boot::ws::WsServer;
use std::io::Read;
use std::net::SocketAddr;
use kernel::handle;

fn main() {
    let mut c = Core::new();
    let mut o = Box::new(Console::new());
    let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut w = Box::new(WsServer::new(&addr));
    c.spawn(o);
    c.spawn(w);
    let mut h = handle::new(c);
    loop {
        match h.borrow_mut().poll() {
            Async::Ready((i, s)) => {
                println!("Received: {:?}", String::from_utf8_lossy(s));
                h.borrow_mut().write(i, b"170");
            }
            x => println!("{:?}", x),
        }
    }
}