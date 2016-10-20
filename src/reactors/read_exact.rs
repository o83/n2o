// # Snatch from tokio-core.
//
// read_exact.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use std::io::{self, Read};
use std::mem;
use abstractions::poll::Poll;
use abstractions::futures::future::Future;

pub struct ReadExact<A, T> {
    state: State<A, T>,
}

enum State<A, T> {
    Reading { a: A, buf: T, pos: usize },
    Empty,
}

pub fn read_exact<A, T>(a: A, buf: T) -> ReadExact<A, T>
    where A: Read,
          T: AsMut<[u8]>
{
    ReadExact {
        state: State::Reading {
            a: a,
            buf: buf,
            pos: 0,
        },
    }
}

fn eof() -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, "early eof")
}

impl<A, T> Future for ReadExact<A, T>
    where A: Read,
          T: AsMut<[u8]>
{
    type Item = (A, T);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(A, T), io::Error> {
        match self.state {
            State::Reading { ref mut a, ref mut buf, ref mut pos } => {
                let buf = buf.as_mut();
                while *pos < buf.len() {
                    let n = try_nb!(a.read(&mut buf[*pos..]));
                    *pos += n;
                    if n == 0 {
                        return Err(eof());
                    }
                }
            }
            State::Empty => panic!("poll a ReadExact after it's done"),
        }

        match mem::replace(&mut self.state, State::Empty) {
            State::Reading { a, buf, .. } => Ok((a, buf).into()),
            State::Empty => panic!(),
        }
    }
}
