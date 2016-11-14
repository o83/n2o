// #
//
// sched.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//
extern crate kernel;

use kernel::reactors::reactor::Reactor;
use kernel::reactors::streams::stream::{Async, Poll, Stream};
use kernel::reactors::streams::into_stream::IntoStream;
use kernel::reactors::streams::done::{self, Done};

struct TestStream {
    id: u32,
}

impl Stream for TestStream {
    type Item = u32;
    type Error = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.id += 1;
        if self.id == 11 {
            return Ok(None);
        }
        match self.id % 2 {
            0 => Ok(Some(Async::Ready(self.id))),
            _ => Ok(Some(Async::NotReady)),
        }
    }
}

fn test_map() {
    println!("===> Testing map combinator...");
    let s = TestStream { id: 0 }
        .map(|v| {
            println!("Stream produced: {:?}", &v);
            v
        })
        .map(|v| println!("Map2 {:?}", v));
    let mut r = Reactor::new();
    r.spawn(s);
    r.run();
}

fn test_then() {
    println!("===> Testing then combinator...");
    let s = TestStream { id: 0 }
        .then(|v| {
            println!("Then combinator received: {:?}", &v);
            let r: Result<u32, u32> = Ok(1);
            r
        })
        .then(|v| {
            println!("Another Then combinator received: {:?}", &v);
            let r: Result<u32, u32> = Ok(0);
            r
        });
    let mut r = Reactor::new();
    r.spawn(s);
    r.run();
}

fn main() {
    test_map();
    test_then();
}
