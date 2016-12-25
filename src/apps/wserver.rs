
extern crate kernel;
use std::net::SocketAddr;
use kernel::io::ws::*;
use kernel::io::reception::Reception;

fn main() {
    let mut r = Reception::new();
    let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut ws = WsServer::new(&mut r, &addr);
    r.select();
}
