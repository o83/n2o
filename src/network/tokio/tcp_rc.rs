// # Allows sharing TcpStream between closures/futures, spawned at runtime.
// # This assumes only one thread. In other case TaskRc<T> should be used.
//
// tcp_rc.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, Read, Write};
use network::tokio::tcp::TcpStream;
use abstractions::poll::Async;

pub struct TcpStreamRc {
    ptr: Rc<RefCell<TcpStream>>,
}

impl TcpStreamRc {
    pub fn new(tcp: TcpStream) -> Self {
        TcpStreamRc { ptr: Rc::new(RefCell::new(tcp)) }
    }
}

impl Clone for TcpStreamRc {
    fn clone(&self) -> TcpStreamRc {
        TcpStreamRc { ptr: self.ptr.clone() }
    }
}

pub fn dup(tcp: TcpStreamRc) -> TcpStreamRc {
    TcpStreamRc { ptr: tcp.ptr.clone() }
}

pub fn split(tcp: TcpStream) -> (TcpStreamRc, TcpStreamRc) {
    let ptr = Rc::new(RefCell::new(tcp));
    (TcpStreamRc { ptr: ptr.clone() }, TcpStreamRc { ptr: ptr })
}

impl TcpStreamRc {
    pub fn poll_read(&mut self) -> Async<()> {
        let ptr = self.ptr.borrow_mut();
        ptr.poll_read()
    }

    pub fn poll_write(&mut self) -> Async<()> {
        let ptr = self.ptr.borrow_mut();
        ptr.poll_write()
    }
}

impl Read for TcpStreamRc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut ptr = self.ptr.borrow_mut();
        ptr.read(buf)
    }
}

impl Write for TcpStreamRc {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut ptr = self.ptr.borrow_mut();
        ptr.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut ptr = self.ptr.borrow_mut();
        ptr.flush()
    }
}
