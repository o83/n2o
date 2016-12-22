
extern crate kernel;
use std::net::SocketAddr;
use kernel::io::ws::*;

fn main() {
    let addr = "127.0.0.1:9001".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut ws = WsServer::new(&addr);
    ws.listen(|(s, m)| {
        println!("Message: {:?}", m);
        s.write_message(&[131, 106]);
    });
}
