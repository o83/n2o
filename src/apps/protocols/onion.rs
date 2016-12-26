extern crate kernel;
use std::net::SocketAddr;
use kernel::io::ws::*;
use kernel::io::console::Console;
use kernel::io::reception::{self, Select, Selected, Handle};

fn main() {
    let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
    let addr2 = "0.0.0.0:9002".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut hdl = reception::handle();
    hdl.insert(Box::new(Console::new()));
    hdl.insert(Box::new(WsServer::new(&addr)));
    hdl.insert(Box::new(WsServer::new(&addr2)));
    hdl.poll();
}