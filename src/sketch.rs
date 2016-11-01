
// THE KERNEL PROTO

// Try to keep dependency list as minimal as possible

use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::collections::VecDeque;
use std::io::Result;
use std::time::{Instant, Duration};
use std::os::unix::io::RawFd;

// Future/Streams compatible protocol

pub enum Async<T> {
    Ready(T),
    NotReady,
}

// System Message with Custom Binary representation

pub struct Message {
    body: Vec<u64>,
}

pub struct State {
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

pub struct Trace<Message, State> {
    trace: Vec<(Message, State)>,
    error: Vec<Message>,
}

pub trait Process<Protocol, State> {
    fn send(&mut self, Protocol, State) -> Result<Async<State>>;
    fn recv(&mut self) -> Result<Async<(Protocol, State)>>;
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

pub struct NetworkPoll(usize);
pub struct PollOpt(usize);
pub struct Ready(usize);
pub struct Token(usize);

// Event loop

pub struct Poll {
    selector: Selector,
    readiness_queue: PollQueue,
}

struct PollQueue {
    all: Option<Box<ReadinessNode>>,
    readiness: AtomicPtr<ReadinessNode>,
    sleep_token: Box<ReadinessNode>,
}

struct ReadinessNode {
    next: Option<Box<ReadinessNode>>,
    prev: ReadyRef,
    events: AtomicUsize,
    queued: AtomicUsize,
}

struct ReadyRef {
    ptr: *mut ReadinessNode,
}

struct RegistrationData {
    token: Token,
    interest: Ready,
    opts: PollOpt,
}

pub struct epoll_event {
    // backend specific info
    events: u32,
    u64: u64,
}

pub struct Events {
    events: Vec<epoll_event>,
}

pub struct Selector {
    id: usize,
    epfd: RawFd,
}

// mio compatible API for Selector

pub trait Network {
    fn create() -> Result<Selector>; // xsock_poll_create
    fn poll(&mut self, evts: &mut Events, timeout: Option<Duration>) -> Result<bool>; // xsock_poll_poll
    fn add(&mut self, fd: RawFd, cookie: u64) -> Result<()>; // xsock_poll_add
    fn remove(&mut self, fd: RawFd) -> Result<()>; // xsock_poll_remove
}

// Socket Context

pub struct Socket {
    buffer: VecDeque<u64>,
    poll: Poll,
}
