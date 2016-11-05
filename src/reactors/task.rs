// #
//
// task.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use abstractions::future::{Poll, Future, Async};
use std::result::Result;

pub struct Task<F> {
    id: u32,
    priority: u32,
    coro: F,
}

impl<F> Future for Task<F>
    where F: FnMut()
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        (&mut self.coro)();
        Ok(Async::NotReady)
    }
}

impl<F> Task<F> {
    pub fn new(id: u32, priority: u32, f: F) -> Task<F> {
        Task {
            id: id,
            priority: priority,
            coro: f,
        }
    }
}
