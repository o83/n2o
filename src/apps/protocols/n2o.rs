extern crate kernel;
use std::net::SocketAddr;
use kernel::io::ws::*;
use kernel::io::reception::Reception;
use std::cell::UnsafeCell;

fn main() {
    let mut r = UnsafeCell::new(Reception::new());
    let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut r1 = unsafe { &mut *r.get() };
    let mut ws = WsServer::new(&mut r1, &addr);
    let r2 = unsafe { &mut *r.get() };
    r2.select();
}