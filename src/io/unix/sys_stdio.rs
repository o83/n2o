// #
//
// stdio.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use std::io;
use libc;
use super::fd::FileDesc;

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

impl Stdin {
    pub fn new() -> io::Result<Stdin> {
        Ok(Stdin(()))
    }

    pub fn read(&self, data: &mut [u8]) -> io::Result<usize> {
        let fd = FileDesc::new(libc::STDIN_FILENO);
        let ret = fd.read(data);
        fd.into_raw();
        ret
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let fd = FileDesc::new(libc::STDIN_FILENO);
        let ret = fd.read_to_end(buf);
        fd.into_raw();
        ret
    }
}

impl Stdout {
    pub fn new() -> io::Result<Stdout> {
        Ok(Stdout(()))
    }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        let fd = FileDesc::new(libc::STDOUT_FILENO);
        let ret = fd.write(data);
        fd.into_raw();
        ret
    }
}

impl Stderr {
    pub fn new() -> io::Result<Stderr> {
        Ok(Stderr(()))
    }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        let fd = FileDesc::new(libc::STDERR_FILENO);
        let ret = fd.write(data);
        fd.into_raw();
        ret
    }
}

impl io::Write for Stderr {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        Stderr::write(self, data)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub const EBADF_ERR: i32 = ::libc::EBADF as i32;
pub const STDIN_BUF_SIZE: usize = 8 * 1024;
