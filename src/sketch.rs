
use std::collections::VecDeque;
use abstractions::poll::Async;
use core::ops::FnMut;

// Message

pub struct Message { }

// For using with Tasks and Timers

pub trait Discipline {
    fn select(&mut self, u64) -> Async<Message>;
}

// Queue Buffer

pub struct QueueContext<Message> {
    length: u64,
    cursor: u64,
    buffer: VecDeque<Message>,
}

// Queue API

pub trait Queue<Message>: Discipline {
    fn push(&mut self, Message) -> ();
    fn pop(&mut self) -> Message;
}

// Task Scheduler/Reactor

pub struct ReactorContext {
    cursor: u64,
    tasks: Queue<Message>,
}

pub trait Reactor: Discipline {
    fn add(&mut self, Task) -> u64;
    fn remove(&mut self, u64);
    fn reschedule(&mut self);
}

// Task Context

pub struct Task {
    prio: u64,
    lambda: FnMut(),
}

// Timer Reactor Context

pub struct ClockContext {
    clock: u64,
    timers: VecDeque<Message>,
}

// Timer API

pub trait Clock: Discipline {
    fn add(&mut self, Timer) -> u64;
    fn remove(&mut self, u64);
    fn reschedule(&mut self);
}

// Single Timer Context

pub struct Timer {
    interval: u64,
    task_id: u64,
}

// Network Subsystems

pub struct Network {
    cursor: u64,
    buffers: VecDeque<Socket>,
}

// Socket Context

pub struct Socket {
    buffer: VecDeque<Message>,
}