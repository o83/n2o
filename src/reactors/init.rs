use std::rc::Rc;
use streams::intercore::ctx::Ctx;
use streams::intercore::api::{Message, Spawn};
use reactors::junk::Junk;
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

pub struct Host<'a> {
    junk: Handle<Junk<'a>>,
    cores: Vec<Core<'a>>,
}

impl<'a> Host<'a> {
    pub fn new() -> Self {
        let mut ctxs = Vec::new();
        ctxs.push(Rc::new(Ctx::new()));
        Host {
            cores: Vec::new(),
            junk: handle::new(Junk::new(ctxs.last().unwrap().clone())),
        }
    }

    fn connect_cores(&mut self) {
        for i in 1..5 {
            println!("init core_{:?}", i);
            let core = Core::new(i);
            core.connect_with(&self.junk.borrow().core);
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
        self.junk.add_selected(o);
        self.junk.add_selected(w);
        self.junk.add_selected(s);
        match p.next_n(3) {
            Some(vs) => {
                vs[0] = Message::Halt;
                vs[1] = Message::Unknown;
                vs[2] = Message::Spawn(Spawn { id: 13, id2: 42 });
                p.commit();
            }
            None => {}
        }
        self.junk.run();
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
