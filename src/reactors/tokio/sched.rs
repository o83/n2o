
use std::cell::RefCell;
use std::io::{self, ErrorKind};
use std::mem;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::time::{Instant, Duration};

use abstractions::futures::lazy;
use abstractions::futures::future::Future;
use abstractions::streams::stream::IntoFuture;
use abstractions::poll::Async;
use abstractions::tasks::task;
use abstractions::tasks::task::{Unpark, Task, Spawn};
use mio;
use slab::Slab;

use reactors::tokio::heap::{Heap, Slot};
use reactors::tokio::channel::{Sender, Receiver, channel};


static NEXT_LOOP_ID: AtomicUsize = ATOMIC_USIZE_INIT;
scoped_thread_local!(static CURRENT_LOOP: Core);

const SLAB_CAPACITY: usize = 1024 * 64;


pub struct Core {
    events: mio::Events,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    inner: Rc<RefCell<Inner>>,
    _future_registration: mio::Registration,
    future_readiness: Arc<MySetReadiness>,
}

pub struct Inner {
    id: usize,
    io: mio::Poll,
    io_dispatch: Slab<ScheduledIo>,
    task_dispatch: Slab<ScheduledTask>,
    timer_heap: Heap<(Instant, usize)>,
    timeouts: Slab<(Option<Slot>, TimeoutState)>,
}


#[derive(Clone)]
pub struct Remote {
    id: usize,
    tx: Sender<Message>,
}

#[derive(Clone)]
pub struct Handle {
    remote: Remote,
    pub inner: Weak<RefCell<Inner>>,
}

pub struct ScheduledIo {
    readiness: Arc<AtomicUsize>,
    reader: Option<Task>,
    writer: Option<Task>,
}

pub struct ScheduledTask {
    _registration: mio::Registration,
    spawn: Option<Spawn<Box<Future<Item = (), Error = ()>>>>,
    wake: Arc<MySetReadiness>,
}

pub enum TimeoutState {
    NotFired,
    Fired,
    Waiting(Task),
}

pub enum Direction {
    Read,
    Write,
}

pub enum Message {
    DropSource(usize),
    Schedule(usize, Task, Direction),
    UpdateTimeout(usize, Task),
    ResetTimeout(usize, Instant),
    CancelTimeout(usize),
    Run(Box<FnBox>),
}

const TOKEN_MESSAGES: mio::Token = mio::Token(0);
const TOKEN_FUTURE: mio::Token = mio::Token(1);
const TOKEN_START: usize = 2;

impl Core {
    pub fn new() -> io::Result<Core> {
        let (tx, rx) = channel();
        let io = try!(mio::Poll::new());
        try!(io.register(&rx,
                         TOKEN_MESSAGES,
                         mio::Ready::readable(),
                         mio::PollOpt::edge()));
        let future_pair = mio::Registration::new(&io,
                                                 TOKEN_FUTURE,
                                                 mio::Ready::readable(),
                                                 mio::PollOpt::level());
        Ok(Core {
            events: mio::Events::with_capacity(1024),
            tx: tx,
            rx: rx,
            _future_registration: future_pair.0,
            future_readiness: Arc::new(MySetReadiness(future_pair.1)),

            inner: Rc::new(RefCell::new(Inner {
                id: NEXT_LOOP_ID.fetch_add(1, Ordering::Relaxed),
                io: io,
                io_dispatch: Slab::with_capacity(SLAB_CAPACITY),
                task_dispatch: Slab::with_capacity(SLAB_CAPACITY),
                timeouts: Slab::with_capacity(SLAB_CAPACITY),
                timer_heap: Heap::new(),
            })),
        })
    }


    pub fn handle(&self) -> Handle {
        Handle {
            remote: self.remote(),
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub fn remote(&self) -> Remote {
        Remote {
            id: self.inner.borrow().id,
            tx: self.tx.clone(),
        }
    }

    pub fn run<F>(&mut self, f: F) -> Result<F::Item, F::Error>
        where F: Future
    {
        let mut task = task::spawn(f);
        let ready = self.future_readiness.clone();

        let mut res = None;
        self._run(&mut || {
            assert!(res.is_none());
            match task.poll_future(ready.clone()) {
                Ok(Async::NotReady) => {}
                Ok(Async::Ready(e)) => res = Some(Ok(e)),
                Err(e) => res = Some(Err(e)),
            }
            res.is_some()
        });
        res.expect("run should not return until future is done")
    }

    fn _run(&mut self, done: &mut FnMut() -> bool) {

        if CURRENT_LOOP.set(self, || done()) {
            return;
        }

        let mut finished = false;
        while !finished {
            let amt;

            let start = Instant::now();
            loop {
                let inner = self.inner.borrow_mut();
                let timeout = inner.timer_heap.peek().map(|t| {
                    if t.0 < start {
                        Duration::new(0, 0)
                    } else {
                        t.0 - start
                    }
                });
                match inner.io.poll(&mut self.events, timeout) {
                    Ok(a) => {
                        amt = a;
                        break;
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                    err @ Err(_) => {
                        err.unwrap();
                    }
                }
            }
            debug!("loop poll - {:?}", start.elapsed());
            debug!("loop time - {:?}", Instant::now());

            let start = Instant::now();
            self.consume_timeouts(start);


            for i in 0..self.events.len() {
                let event = self.events.get(i).unwrap();
                let token = event.token();
                trace!("event {:?} {:?}", event.kind(), event.token());

                if token == TOKEN_MESSAGES {
                    CURRENT_LOOP.set(&self, || self.consume_queue());
                } else if token == TOKEN_FUTURE {
                    self.future_readiness.0.set_readiness(mio::Ready::none()).unwrap();
                    if !finished && CURRENT_LOOP.set(self, || done()) {
                        finished = true;
                    }
                } else {
                    self.dispatch(token, event.kind());
                }
            }

            debug!("loop process - {} events, {:?}", amt, start.elapsed());
        }
    }

    fn dispatch(&mut self, token: mio::Token, ready: mio::Ready) {
        let token = usize::from(token) - TOKEN_START;
        if token % 2 == 0 {
            self.dispatch_io(token / 2, ready)
        } else {
            self.dispatch_task(token / 2)
        }
    }

    fn dispatch_io(&mut self, token: usize, ready: mio::Ready) {
        let mut reader = None;
        let mut writer = None;
        let mut inner = self.inner.borrow_mut();
        if let Some(io) = inner.io_dispatch.get_mut(token) {
            if ready.is_readable() {
                reader = io.reader.take();
                io.readiness.fetch_or(1, Ordering::Relaxed);
            }
            if ready.is_writable() {
                writer = io.writer.take();
                io.readiness.fetch_or(2, Ordering::Relaxed);
            }
        }
        drop(inner);

        if let Some(reader) = reader {
            self.notify_handle(reader);
        }
        if let Some(writer) = writer {
            self.notify_handle(writer);
        }
    }

    fn dispatch_task(&mut self, token: usize) {
        let mut inner = self.inner.borrow_mut();
        let (task, wake) = match inner.task_dispatch.get_mut(token) {
            Some(slot) => (slot.spawn.take(), slot.wake.clone()),
            None => return,
        };
        wake.0.set_readiness(mio::Ready::none()).unwrap();
        let mut task = match task {
            Some(task) => task,
            None => return,
        };
        drop(inner);
        let res = CURRENT_LOOP.set(self, || task.poll_future(wake));
        inner = self.inner.borrow_mut();
        match res {
            Ok(Async::NotReady) => {
                assert!(inner.task_dispatch[token].spawn.is_none());
                inner.task_dispatch[token].spawn = Some(task);
            }
            Ok(Async::Ready(())) |
            Err(()) => {
                inner.task_dispatch.remove(token).unwrap();
            }
        }
    }

    fn consume_timeouts(&mut self, now: Instant) {
        loop {
            let mut inner = self.inner.borrow_mut();
            match inner.timer_heap.peek() {
                Some(head) if head.0 <= now => {}
                Some(_) => break,
                None => break,
            };
            let (_, slab_idx) = inner.timer_heap.pop().unwrap();

            trace!("firing timeout: {}", slab_idx);
            inner.timeouts[slab_idx].0.take().unwrap();
            let handle = inner.timeouts[slab_idx].1.fire();
            drop(inner);
            if let Some(handle) = handle {
                self.notify_handle(handle);
            }
        }
    }

    fn notify_handle(&self, handle: Task) {
        debug!("notifying a task handle");
        CURRENT_LOOP.set(&self, || handle.unpark());
    }

    fn consume_queue(&self) {
        debug!("consuming notification queue");
        while let Some(msg) = self.rx.recv().unwrap() {
            self.notify(msg);
        }
    }

    fn notify(&self, msg: Message) {
        match msg {
            Message::DropSource(tok) => self.inner.borrow_mut().drop_source(tok),
            Message::Schedule(tok, wake, dir) => {
                let task = self.inner.borrow_mut().schedule(tok, wake, dir);
                if let Some(task) = task {
                    self.notify_handle(task);
                }
            }
            Message::UpdateTimeout(t, handle) => {
                let task = self.inner.borrow_mut().update_timeout(t, handle);
                if let Some(task) = task {
                    self.notify_handle(task);
                }
            }
            Message::ResetTimeout(t, at) => {
                self.inner.borrow_mut().reset_timeout(t, at);
            }
            Message::CancelTimeout(t) => self.inner.borrow_mut().cancel_timeout(t),
            Message::Run(r) => r.call_box(self),
        }
    }
}

impl Inner {
    pub fn add_source(&mut self, source: &mio::Evented) -> io::Result<(Arc<AtomicUsize>, usize)> {
        debug!("adding a new I/O source");
        let sched = ScheduledIo {
            readiness: Arc::new(AtomicUsize::new(0)),
            reader: None,
            writer: None,
        };
        if self.io_dispatch.vacant_entry().is_none() {
            let amt = self.io_dispatch.len();
            self.io_dispatch.reserve_exact(amt);
        }
        let entry = self.io_dispatch.vacant_entry().unwrap();
        try!(self.io.register(source,
                              mio::Token(TOKEN_START + entry.index() * 2),
                              mio::Ready::readable() | mio::Ready::writable(),
                              mio::PollOpt::edge()));
        Ok((sched.readiness.clone(), entry.insert(sched).index()))
    }

    fn drop_source(&mut self, token: usize) {
        debug!("dropping I/O source: {}", token);
        self.io_dispatch.remove(token).unwrap();
    }

    fn schedule(&mut self, token: usize, wake: Task, dir: Direction) -> Option<Task> {
        debug!("scheduling direction for: {}", token);
        let sched = self.io_dispatch.get_mut(token).unwrap();
        let (slot, bit) = match dir {
            Direction::Read => (&mut sched.reader, 1),
            Direction::Write => (&mut sched.writer, 2),
        };
        if sched.readiness.load(Ordering::SeqCst) & bit != 0 {
            *slot = None;
            Some(wake)
        } else {
            *slot = Some(wake);
            None
        }
    }

    pub fn add_timeout(&mut self, at: Instant) -> usize {
        if self.timeouts.vacant_entry().is_none() {
            let len = self.timeouts.len();
            self.timeouts.reserve_exact(len);
        }
        let entry = self.timeouts.vacant_entry().unwrap();
        let slot = self.timer_heap.push((at, entry.index()));
        let entry = entry.insert((Some(slot), TimeoutState::NotFired));
        debug!("added a timeout: {}", entry.index());
        return entry.index();
    }

    fn update_timeout(&mut self, token: usize, handle: Task) -> Option<Task> {
        debug!("updating a timeout: {}", token);
        self.timeouts[token].1.block(handle)
    }

    fn reset_timeout(&mut self, token: usize, at: Instant) {
        let pair = &mut self.timeouts[token];
        if let Some(slot) = pair.0.take() {
            self.timer_heap.remove(slot);
        }
        let slot = self.timer_heap.push((at, token));
        *pair = (Some(slot), TimeoutState::NotFired);
        debug!("set a timeout: {}", token);
    }

    fn cancel_timeout(&mut self, token: usize) {
        debug!("cancel a timeout: {}", token);
        let pair = self.timeouts.remove(token);
        if let Some((Some(slot), _state)) = pair {
            self.timer_heap.remove(slot);
        }
    }

    fn spawn(&mut self, future: Box<Future<Item = (), Error = ()>>) {
        if self.task_dispatch.vacant_entry().is_none() {
            let len = self.task_dispatch.len();
            self.task_dispatch.reserve_exact(len);
        }
        let entry = self.task_dispatch.vacant_entry().unwrap();
        let token = TOKEN_START + 2 * entry.index() + 1;
        let pair = mio::Registration::new(&self.io,
                                          mio::Token(token),
                                          mio::Ready::readable(),
                                          mio::PollOpt::level());
        let unpark = Arc::new(MySetReadiness(pair.1));
        let entry = entry.insert(ScheduledTask {
            spawn: Some(task::spawn(future)),
            wake: unpark,
            _registration: pair.0,
        });
        entry.get().wake.clone().unpark();
    }
}

impl Remote {
    pub fn send(&self, msg: Message) {
        self.with_loop(|lp| {
            match lp {
                Some(lp) => {
                    lp.consume_queue();
                    lp.notify(msg);
                }
                None => {
                    match self.tx.send(msg) {
                        Ok(()) => {}
                        Err(e) => panic!("error sending message to event loop: {}", e),
                    }
                }
            }
        })
    }

    fn with_loop<F, R>(&self, f: F) -> R
        where F: FnOnce(Option<&Core>) -> R
    {
        if CURRENT_LOOP.is_set() {
            CURRENT_LOOP.with(|lp| {
                let same = lp.inner.borrow().id == self.id;
                if same { f(Some(lp)) } else { f(None) }
            })
        } else {
            f(None)
        }
    }

    pub fn spawn<F, R>(&self, f: F)
        where F: FnOnce(&Handle) -> R + Send + 'static,
              R: IntoFuture<Item = (), Error = ()>,
              R::Future: 'static
    {
        self.send(Message::Run(Box::new(|lp: &Core| {
            let f = f(&lp.handle());
            lp.inner.borrow_mut().spawn(Box::new(f.into_future()));
        })));
    }
}

impl Handle {
    pub fn remote(&self) -> &Remote {
        &self.remote
    }

    pub fn spawn<F>(&self, f: F)
        where F: Future<Item = (), Error = ()> + 'static
    {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return,
        };
        inner.borrow_mut().spawn(Box::new(f));
    }

    pub fn spawn_fn<F, R>(&self, f: F)
        where F: FnOnce() -> R + 'static,
              R: IntoFuture<Item = (), Error = ()> + 'static
    {
        self.spawn(lazy::new(f))
    }
}

impl TimeoutState {
    fn block(&mut self, handle: Task) -> Option<Task> {
        match *self {
            TimeoutState::Fired => return Some(handle),
            _ => {}
        }
        *self = TimeoutState::Waiting(handle);
        None
    }

    fn fire(&mut self) -> Option<Task> {
        match mem::replace(self, TimeoutState::Fired) {
            TimeoutState::NotFired => None,
            TimeoutState::Fired => panic!("fired twice?"),
            TimeoutState::Waiting(handle) => Some(handle),
        }
    }
}

struct MySetReadiness(mio::SetReadiness);

impl Unpark for MySetReadiness {
    fn unpark(&self) {
        self.0
            .set_readiness(mio::Ready::readable())
            .expect("failed to set readiness");
    }
}

pub trait FnBox: Send + 'static {
    fn call_box(self: Box<Self>, lp: &Core);
}

impl<F: FnOnce(&Core) + Send + 'static> FnBox for F {
    fn call_box(self: Box<Self>, lp: &Core) {
        (*self)(lp)
    }
}
