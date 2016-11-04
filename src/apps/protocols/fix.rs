#[macro_use]
extern crate kernel;
extern crate core;
extern crate rand;

use kernel::abstractions::session_types::*;
use std::{thread, time, string};
use core::mem::transmute;
use rand::random;

type ACCN = u64;
type SYNC = u64;
type ICMP = u64;
type DENY = u64;

type Client = <Fix as HasDual>::Dual;
type Fix = Rec<Offer<Ping, Login>>;
type Ping = Recv<ICMP, Choose<Choose<SyncData, Pong>, Var<Z>>>;
type Login = Recv<ACCN, Choose<Choose<Reject, Logon>, Var<Z>>>;

type Logon = Send<ACCN, Var<Z>>;
type Reject = Send<DENY, Var<Z>>;
type SyncData = Send<SYNC, Var<Z>>;
type Pong = Send<ICMP, Var<Z>>;

fn main() {
    let (server, client) = session_channel();
    let x = thread::spawn(|| fix(server));
    let y = thread::spawn(|| cli(client));
    x.join();
}

fn cli(c: Chan<(), Client>) {
    let mut c = c.enter();
    let icmp: ICMP = 200;
    let accn: ACCN = 100;
    let sync: SYNC = 300;
    let deny: DENY = 400;
    for i in 0..10 {
        println!("iter {}", i);
        if i % 2 == 0 && i > 2 {
            match c.sel1().send(icmp + i).offer() {
                Branch::Left(z) => {
                    match z.offer() {
                        Branch::Left(x) => {
                            let (i, n) = x.recv();
                            println!("SYNC {}", n);
                            c = i.zero();
                        }
                        Branch::Right(x) => {
                            let (i, n) = x.recv();
                            println!("PONG {}", n);
                            c = i.zero();
                        }
                    }
                }
                Branch::Right(z) => {
                    c = z.zero();
                }
            };
        } else {
            match c.sel2().send(accn + i).offer() {
                Branch::Left(z) => {
                    match z.offer() {
                        Branch::Left(x) => {
                            let (i, n) = x.recv();
                            println!("REJECT {}", n);
                            c = i.zero();
                        }
                        Branch::Right(x) => {
                            let (i, n) = x.recv();
                            println!("LOGON {}", n);
                            c = i.zero();
                        }
                    }
                }
                Branch::Right(z) => {
                    c = z.zero();
                }
            };
        }
    }
}

fn fix(c: Chan<(), Fix>) {
    let mut c = c.enter();
    println!("server");
    let accn: ACCN = 100;
    let icmp: ICMP = 200;
    let sync: SYNC = 300;
    let deny: DENY = 400;
    loop {
        c = offer!{
            c,
            PING => {
                let (c, ping) = c.recv();
                println!("ping received: {}", ping);
                if ping < icmp {
                    c.sel2().zero()
                } else if ping == icmp {
                    c.sel1().sel1().send(sync).zero()
                } else {
                    c.sel1().sel2().send(icmp).zero()
                }
            },
            LOGIN => {
                let (c, account) = c.recv();
                println!("login received: {}", account);
                if account < accn {
                    thread::sleep_ms(100);
                    c.sel2().zero()
                } else if account == accn {
                    c.sel1().sel1().send(deny).zero()
                } else {
                    c.sel1().sel2().send(accn).zero()
                }
            }
        }
    }
}
