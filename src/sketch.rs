
use std::collections::VecDeque;
use abstractions::poll::Async;
use core::ops::{FnOnce, FnMut};
use std::io::{Error, Result};

// System Message with Custom Binary representation

pub struct Message { }

// For using with Tasks and Timers to select active subset

pub trait Discipline {
    fn select(&mut self, u64) -> Result<Async<u64>>;
}

// Queues Manager

pub trait QueueManager: Discipline {
    fn create(&mut self, Queue<Message>) -> u64;
    fn destroy(&mut self, u64);
}

// Queue Buffer

pub struct QueueContext<Message> {
    length: u64,
    cursor: u64,
    buffer: VecDeque<Message>,
}

// Queue API

pub trait Queue<Message> {
    fn push(&mut self, Message) -> Queue<Message>;
    fn pop(&mut self) -> Message;
}

// Task Scheduler/Reactor

pub struct ReactorContext<Task> {
    cursor: u64,
    tasks: Queue<Task>,
}

pub trait Reactor<Task>: Discipline {
    fn spawn(&mut self, Task) -> u64;
    fn kill(&mut self, u64);
    fn reschedule(&mut self);
}

// Task Context

pub struct Task<Protocol> {
    prio: u64,
    lambda: FnMut(Protocol),
}

pub trait Process<Protocol, State> {
    fn send(&mut self, Protocol, State) -> State;
}

// Timer Reactor Context

pub struct ClockContext {
    clock: u64,
    timers: Queue<Timer>,
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
    buffers: Queue<Socket>,
}

// Socket Context

pub struct Socket {
    buffer: Queue<Message>,
}
