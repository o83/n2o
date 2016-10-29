
// Try to keep dependency list as minimal as possible

use std::collections::VecDeque;
use abstractions::poll::Async;
use std::io::Result;
use std::time::Instant;

// System Message with Custom Binary representation

pub struct Message {
    body: Vec<u64>,
}

pub trait Parser {}

impl Parser for Message {}

// For using with Tasks and Timers to select active subset

pub trait Discipline {
    fn select(&mut self, u64) -> Result<Async<u64>>;
}

// List of Queues

pub struct QueueManagerContext<Message> {
    queues: Vec<QueueContext<Message>>,
    size: u64,
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
    fn push(&mut self, Message) -> Result<Async<u64>>;
    fn pop(&mut self) -> Result<Async<Message>>;
}

// Queue Linking/Topology

pub struct Multicast {
    pipes: Vec<u64>,
}

// Publish

pub trait Pub<Message> {
    fn register(&mut self, QueueContext<Message>);
    fn publish(&mut self);
}

impl<Message> Pub<Message> for Multicast {
    fn register(&mut self, a: QueueContext<Message>) {}
    fn publish(&mut self) {}
}

// Subscribe

pub trait Sub<Message> {
    fn register<F>(&mut self, F) where F: Fn(Message) -> ();
    fn subscribe(&mut self, u64);
}

impl<Message> Sub<Message> for Multicast {
    fn register<F>(&mut self, fun: F) where F: Fn(Message) -> () {}
    fn subscribe(&mut self, a: u64) {}
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

pub struct Task<Protocol, State> {
    state: Vec<u8>,
    prio: u64,
    lambda: Fn(Protocol, State) -> State,
}

pub trait Process<Protocol, State> {
    fn send(&mut self, Protocol, State) -> Result<Async<State>>;
}

// Timer Reactor Context

pub struct TimersContext {
    cursor: u64,
    timers: Queue<Timer>,
}

// Timer API

pub trait Timers: Discipline {
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

pub struct NetworkContext {
    cursor: u64,
    sockets: Queue<Socket>,
}

pub struct NetworkPoll {}

// X-SOCK native backend

pub trait Network {
    fn open(u8);
    fn close();
    fn client(i8, NetworkContext, Socket);
    fn server(i8, NetworkContext, Socket);
    fn create(NetworkPoll);
    fn destroy(NetworkPoll);
    fn add(NetworkPoll, Socket, cookie: u64);
    fn remove(NetworkPoll, Socket);
    fn poll(NetworkPoll, Instant, i8);
    fn send(Socket, VecDeque<u64>, u64, sent: u64);
    fn recv(Socket, VecDeque<u64>, u64, received: u64);
}

// Socket Context

pub struct Socket {
    buffer: VecDeque<u64>,
}
