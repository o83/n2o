// #
//
// task.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use super::future::{Poll, Future, Async};
use std::result::Result;

pub struct Task<'a> {
    id: u32,
    priority: u32,
    rxs: Vec<usize>,
    txs: Vec<usize>,
    // coro: FnMut(),
    tail: Option<&'a Task<'a>>,
}

impl<'a> Future for Task<'a> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // self.coro();
        println!("POLL called!!!!");
        Ok(Async::NotReady)
    }
}

impl<'a> Task<'a> {
    pub fn new(id: u32, priority: u32 /* f: FnMut() */) -> Task<'a> {
        Task {
            id: id,
            priority: priority,
            rxs: Vec::new(),
            txs: Vec::new(),
            // coro: f,
            tail: None,
        }
    }
}
