extern crate kernel;
use kernel::reactors::boot::hub::Hub;
use kernel::handle;
use kernel::reactors::boot::console::Console;
use kernel::reactors::boot::ws::WsServer;
use std::net::SocketAddr;
use kernel::reactors::boot::reactor::{Async, Core, Select, Selector};

fn main() {
    let mut h = handle::new(Hub::new());
    let mut o = Selector::Rx(Console::new());
    let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut w = Selector::Ws(WsServer::new(&addr));
    let input1 = "a:{snd[0;42]; snd[0;44]; rcv 1; rcv 1;a 1};a 1";
    let input2 = "b:{rcv 0; rcv 0; snd[1;41]; snd[1;43]; b 1};b 1";
    // h.borrow_mut().exec(Some(&input1));
    // h.borrow_mut().exec(Some(&input2));
    h.borrow_mut().add_selected(o);
    h.borrow_mut().add_selected(w);
    h.borrow_mut().boil()
}