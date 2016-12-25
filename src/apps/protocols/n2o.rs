extern crate kernel;
use std::net::SocketAddr;
use kernel::io::ws::*;
use kernel::io::reception::Select;
use std::cell::UnsafeCell;

fn main() {
    let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
    let addr2 = "0.0.0.0:9002".parse::<SocketAddr>().ok().expect("Parser Error");
    let mut s = UnsafeCell::new(Select::new());
    let s1 = unsafe { &mut *s.get() };
    let s2 = unsafe { &mut *s.get() };
    let s3 = unsafe { &mut *s.get() };
    s1.insert(WsServer::new(&addr));
    s2.insert(WsServer::new(&addr2));
    s3.poll();
}