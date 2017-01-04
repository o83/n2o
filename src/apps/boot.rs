extern crate kernel;
use kernel::reactors::boot::reactor::{Async, Core};
use kernel::reactors::boot::console::Console;
use kernel::reactors::boot::ws::WsServer;
use std::io::Read;
use std::net::SocketAddr;

fn main() {
    let mut c = Core::new();
    let mut o = Box::new(Console::new());
    let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut w = Box::new(WsServer::new(&addr));
    c.spawn(o);
    c.spawn(w);
    for a in c {
        match a {
            Async::Ready((i, s)) => {
                println!("Async: {:?}", i);
                // c.write(i, s.as_bytes());
            }
            x => println!("{:?}", x),
        }
    }
}