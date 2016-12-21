
extern crate kernel;
use std::net::SocketAddr;
use kernel::io::ws::*;

fn main() {
    let addr = "127.0.0.1:8000".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut ws = WsServer::new(&addr);
    ws.listen();
}
