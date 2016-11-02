// #
//
// sched.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//
extern crate kernel;

use kernel::reactors::sched::reactor::Reactor;
use kernel::reactors::sched::task::Task;

#[test]
fn task_test() {
    let t = Task::new(0, 1);
    let mut r = Reactor::new();
    r.spawn(t);
    r.run();
}
