// # Utilizes os process, representing stdout, stderr as a futures::Stream.
//
// process.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use std::io::{self, Read};
use std::process::{Command, Stdio, Child};
use kernel::abstractions::poll::{Poll, Async};
use kernel::abstractions::streams::stream::Stream;

pub struct Frame {
    pub data: [u8; 256],
    pub size: usize,
}

pub struct Process {
    pub process: Child,
}

impl Stream for Process {
    type Item = Frame;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Frame>, io::Error> {
        let buf = &mut [0u8; 256];
        match self.process.stdout {
            Some(ref mut o) => {
                match o.read(buf) {
                    Ok(sz) => {
                        match sz {
                            0 => Ok(Async::Ready(None)),
                            _ => {
                                Ok(Async::Ready(Some(Frame {
                                    data: *buf,
                                    size: sz,
                                })))
                            }
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            None => Ok(Async::NotReady),
        }
    }
}

impl Process {
    pub fn spawn<'a, I: Iterator<Item = &'a str>>(mut cmd: I) -> io::Result<Self> {
        match cmd.next() {
            Some(e) => {
                let mut process = Command::new(e);
                for a in cmd {
                    debug!("Arg: {}", &a);
                    process.arg(a as &str);
                }
                let p = process.stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn();
                match p {
                    Ok(p) => Ok(Process { process: p }),
                    Err(e) => Err(e),
                }
            }
            None => Err(io::Error::new(io::ErrorKind::Other, "Empty command!")),
        }
    }
}
