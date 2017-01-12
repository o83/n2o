use std::rc::Rc;
use reactors::core::Core;
use reactors::scheduler::Scheduler;
use streams::intercore::ctx::Ctx;
use streams::intercore::api::{Message, Spawn};
use reactors::hub::Hub;
use std::mem;
use handle::{self, Handle};
use reactors::job::Job;
use std::sync::{Arc, Mutex, Once, ONCE_INIT};
use core::ops::DerefMut;
use std::cell::UnsafeCell;
// TODO: next uses will be removed when Interpreter
// could create IO's dynamically.
use reactors::console::Console;
use reactors::ws::WsServer;
use std::net::SocketAddr;
use reactors::selector::{Select, Selector, Async};
use queues::publisher::{Publisher, Subscriber};
use std::ffi::CString;

pub struct Host<'a> {
    schedulers: Vec<Scheduler<'a, Job<'a>>>,
    junk: Handle<Hub<'a>>,
    rings: Vec<Rc<Ctx>>,
    cores: Vec<Core>,
}

impl<'a> Host<'a> {
    pub fn new() -> Self {
        let mut ctxs = Vec::new();
        ctxs.push(Rc::new(Ctx::new()));
        Host {
            schedulers: Vec::new(),
            junk: handle::new(Hub::new(ctxs.last().unwrap().clone())),
            rings: ctxs,
            cores: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        let mut o = Selector::Rx(Console::new());
        let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
        let mut w = Selector::Ws(WsServer::new(&addr));
        let mut p = Publisher::with_mirror(CString::new("/test").unwrap(), 8);
        let mut s = Selector::Sb(p.subscribe());
        self.junk.add_selected(o);
        self.junk.add_selected(w);
        self.junk.add_intercore(s);
        match p.next_n(3) {
            Some(vs) => {
                vs[0] = Message::Halt;
                vs[1] = Message::Unknown;
                vs[2] = Message::Spawn(Spawn { id: 13, id2: 42 });
                p.commit();
            }
            None => {}
        }
        self.junk.boil();
    }
}

#[derive(Clone)]
pub struct HostSingleton {
    pub inner: Arc<UnsafeCell<Host<'static>>>,
}

impl HostSingleton {
    pub fn borrow_mut(&mut self) -> &mut Host<'static> {
        unsafe { &mut *self.inner.get() }
    }
}

pub fn host() -> HostSingleton {
    static mut SINGLETON: *const HostSingleton = 0 as *const HostSingleton;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| {
            let singleton = HostSingleton { inner: Arc::new(UnsafeCell::new(Host::new())) };
            SINGLETON = mem::transmute(box singleton);
        });

        (*SINGLETON).clone()
    }
}
