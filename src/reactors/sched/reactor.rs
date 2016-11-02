// #
//
// reactor.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use std::cmp::Ordering;
use super::task::Task;

pub struct Reactor<'a> {
    tasks: Vec<Task<'a>>,
}

impl<'a> Reactor<'a> {
    pub fn new() -> Self {
        Reactor { tasks: Vec::new() }
    }

    pub fn drop() {
        // drop task from list
    }

    pub fn spawn(&'a mut self, t: Task<'a>) {
        self.tasks.push(t);
    }

    fn reorder(&mut self, slice: &mut [i32]) {
        if slice.len() <= 1 {
            return;
        }
        let partition_idx = self.partition(slice);
        self.reorder(&mut slice[..partition_idx]);
        self.reorder(&mut slice[partition_idx + 1..]);
    }

    fn partition(&mut self, slice: &mut [i32]) -> usize {
        let mut partition_idx = 0;
        for i in 1..slice.len() {
            match slice[i].cmp(&slice[partition_idx]) {
                Ordering::Less => {
                    slice.swap(i, partition_idx + 1);
                    slice.swap(partition_idx, partition_idx + 1);
                    partition_idx += 1;
                }
                _ => (),
            }
        }
        return partition_idx;
    }

    pub fn run() {

        loop {
            // process tasks by their priorities:

        }
    }
}

#[cfg(tests)]
mod test {
    #[test]
    fn task_test() {
        let t = Task::new(0, 1, || 12);
        Ok(())
    }
}
