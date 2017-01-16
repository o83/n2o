use std::rc::Rc;
use streams::intercore::ctx::Ctx;
use streams::intercore::api::{Message, Spawn};
use reactors::boot::Boot;
use std::mem;
use handle::{self, Handle};
use std::sync::{Arc, Once, ONCE_INIT};
use std::cell::UnsafeCell;
use reactors::core::Core;
use reactors::console::Console;
use reactors::ws::WsServer;
use std::net::SocketAddr;
use reactors::selector::Selector;
use queues::publisher::Publisher;
use std::ffi::CString;
use std::env;

struct Args<'a> {
    raw: Vec<String>,
    cores: Option<usize>,
    init: Option<&'a str>,
}

fn args<'a>() -> Args<'a> {
    let a: Vec<String> = env::args().collect();
    Args {
        raw: a,
        cores: Some(5),
        init: None,
    }
}

pub struct Host<'a> {
    args: Args<'a>,
    boot: Handle<Boot<'a>>,
    cores: Vec<Core<'a>>,
}

impl<'a> Host<'a> {
    pub fn new() -> Self {
        let mut ctxs = Vec::new();
        ctxs.push(Rc::new(Ctx::new()));
        Host {
            args: args(),
            cores: Vec::new(),
            boot: handle::new(Boot::new(ctxs.last().expect("There are no ctx's in store.").clone())),
        }
    }

    fn connect_cores(&mut self) {
        for i in 1..self.args.cores.expect("Please, specify number of cores.") {
            println!("init core_{:?}", i);
            let core = Core::new(i);
            core.connect_with(&self.boot.borrow().core);
            for c in &self.cores {
                c.connect_with(&core);
            }
            self.cores.push(core);
        }
    }
    pub fn run(&mut self) {
        self.connect_cores();
        let mut o = Selector::Rx(Console::new());
        let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
        let mut w = Selector::Ws(WsServer::new(&addr));
        let mut p = Publisher::with_mirror(CString::new("/test").unwrap(), 8);
        let mut s = Selector::Sb(p.subscribe());
        self.boot.add_selected(o);
        self.boot.add_selected(w);
        self.boot.add_selected(s);
        match p.next_n(3) {
            Some(vs) => {
                vs[0] = Message::Halt;
                vs[1] = Message::Unknown;
                vs[2] = Message::Spawn(Spawn { id: 13, id2: 42 });
                p.commit();
            }
            None => {}
        }
        self.boot.init();
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
