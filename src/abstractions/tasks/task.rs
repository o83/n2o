
use std::prelude::v1::*;
use std::cell::Cell;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicBool, AtomicUsize, ATOMIC_USIZE_INIT};
use std::thread;

use abstractions::futures::future::BoxFuture;
use abstractions::futures::future::Future;
use abstractions::streams::stream::Stream;
use abstractions::poll::{Async, Poll};
use abstractions::tasks::unpark_mutex::UnparkMutex;
use abstractions::tasks::data;

thread_local!(static CURRENT_TASK: Cell<(*const Task, *const data::LocalMap)> = {
    Cell::new((0 as *const _, 0 as *const _))
});

pub fn fresh_task_id() -> usize {

    static NEXT_ID: AtomicUsize = ATOMIC_USIZE_INIT;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    assert!(id < usize::max_value() / 2,
            "too many previous tasks have been allocated");
    id
}

pub fn set<F, R>(task: &Task, data: &data::LocalMap, f: F) -> R
    where F: FnOnce() -> R
{
    struct Reset((*const Task, *const data::LocalMap));
    impl Drop for Reset {
        fn drop(&mut self) {
            CURRENT_TASK.with(|c| c.set(self.0));
        }
    }

    CURRENT_TASK.with(|c| {
        let _reset = Reset(c.get());
        c.set((task as *const _, data as *const _));
        f()
    })
}

pub fn with<F: FnOnce(&Task, &data::LocalMap) -> R, R>(f: F) -> R {
    let (task, data) = CURRENT_TASK.with(|c| c.get());
    assert!(!task.is_null(), "no Task is currently running");
    debug_assert!(!data.is_null());
    unsafe { f(&*task, &*data) }
}

#[derive(Clone)]
pub struct Task {
    pub id: usize,
    unpark: Arc<Unpark>,
    events: Events,
}

fn _assert_kinds() {
    fn _assert_send<T: Send>() {}
    _assert_send::<Task>();
}

pub fn park() -> Task {
    with(|task, _| task.clone())
}

impl Task {
    pub fn unpark(&self) {
        self.events.trigger();
        self.unpark.unpark();
    }

    pub fn is_current(&self) -> bool {
        with(|current, _| current.id == self.id)
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .finish()
    }
}

pub struct Spawn<T> {
    obj: T,
    id: usize,
    data: data::LocalMap,
}

pub fn spawn<T>(obj: T) -> Spawn<T> {
    Spawn {
        obj: obj,
        id: fresh_task_id(),
        data: data::local_map(),
    }
}

impl<F: Future> Spawn<F> {
    pub fn poll_future(&mut self, unpark: Arc<Unpark>) -> Poll<F::Item, F::Error> {
        self.enter(unpark, |f| f.poll())
    }


    pub fn wait_future(&mut self) -> Result<F::Item, F::Error> {
        let unpark = Arc::new(ThreadUnpark::new(thread::current()));
        loop {
            match try!(self.poll_future(unpark.clone())) {
                Async::NotReady => unpark.park(),
                Async::Ready(e) => return Ok(e),
            }
        }
    }

    pub fn execute(self, exec: Arc<Executor>)
        where F: Future<Item = (), Error = ()> + Send + 'static
    {
        exec.clone().execute(Run {
            spawn: Spawn {
                id: self.id,
                data: self.data,
                obj: self.obj.boxed(),
            },
            inner: Arc::new(Inner {
                exec: exec,
                mutex: UnparkMutex::new(),
            }),
        })
    }
}

impl<S: Stream> Spawn<S> {
    pub fn poll_stream(&mut self, unpark: Arc<Unpark>) -> Poll<Option<S::Item>, S::Error> {
        self.enter(unpark, |stream| stream.poll())
    }

    pub fn wait_stream(&mut self) -> Option<Result<S::Item, S::Error>> {
        let unpark = Arc::new(ThreadUnpark::new(thread::current()));
        loop {
            match self.poll_stream(unpark.clone()) {
                Ok(Async::NotReady) => unpark.park(),
                Ok(Async::Ready(Some(e))) => return Some(Ok(e)),
                Ok(Async::Ready(None)) => return None,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

impl<T> Spawn<T> {
    fn enter<F, R>(&mut self, unpark: Arc<Unpark>, f: F) -> R
        where F: FnOnce(&mut T) -> R
    {
        let task = Task {
            id: self.id,
            unpark: unpark,
            events: Events::new(),
        };
        let obj = &mut self.obj;
        set(&task, &self.data, || f(obj))
    }
}

pub trait Unpark: Send + Sync + 'static {
    fn unpark(&self);
}

pub trait Executor: Send + Sync + 'static {
    fn execute(&self, r: Run);
}

struct ThreadUnpark {
    thread: thread::Thread,
    ready: AtomicBool,
}

impl ThreadUnpark {
    fn new(thread: thread::Thread) -> ThreadUnpark {
        ThreadUnpark {
            thread: thread,
            ready: AtomicBool::new(false),
        }
    }

    fn park(&self) {
        if !self.ready.swap(false, Ordering::SeqCst) {
            thread::park();
        }
    }
}

impl Unpark for ThreadUnpark {
    fn unpark(&self) {
        self.ready.store(true, Ordering::SeqCst);
        self.thread.unpark()
    }
}

pub struct Run {
    spawn: Spawn<BoxFuture<(), ()>>,
    inner: Arc<Inner>,
}

struct Inner {
    mutex: UnparkMutex<Run>,
    exec: Arc<Executor>,
}

impl Run {
    pub fn run(self) {
        let Run { mut spawn, inner } = self;

        unsafe {
            inner.mutex.start_poll();

            loop {
                match spawn.poll_future(inner.clone()) {
                    Ok(Async::NotReady) => {}
                    Ok(Async::Ready(())) |
                    Err(()) => return inner.mutex.complete(),
                }
                let run = Run {
                    spawn: spawn,
                    inner: inner.clone(),
                };
                match inner.mutex.wait(run) {
                    Ok(()) => return,            // we've waited
                    Err(r) => spawn = r.spawn,   // someone's notified us
                }
            }
        }
    }
}

impl Unpark for Inner {
    fn unpark(&self) {
        match self.mutex.notify() {
            Ok(run) => self.exec.execute(run),
            Err(()) => {}
        }
    }
}

pub fn with_unpark_event<F, R>(event: UnparkEvent, f: F) -> R
    where F: FnOnce() -> R
{
    with(|task, data| {
        let new_task = Task {
            id: task.id,
            unpark: task.unpark.clone(),
            events: task.events.with_event(event),
        };
        set(&new_task, data, f)
    })
}

#[derive(Clone)]

pub struct UnparkEvent {
    set: Arc<EventSet>,
    item: usize,
}

impl UnparkEvent {
    pub fn new(set: Arc<EventSet>, id: usize) -> UnparkEvent {
        UnparkEvent {
            set: set,
            item: id,
        }
    }
}

pub trait EventSet: Send + Sync + 'static {
    fn insert(&self, id: usize);
}

#[derive(Clone)]
struct Events {
    set: Vec<UnparkEvent>,
}

impl Events {
    fn new() -> Events {
        Events { set: Vec::new() }
    }

    fn trigger(&self) {
        for event in self.set.iter() {
            event.set.insert(event.item)
        }
    }

    fn with_event(&self, event: UnparkEvent) -> Events {
        let mut set = self.set.clone();
        set.push(event);
        Events { set: set }
    }
}
