use std::mem;
use std::rc::Rc;
use std::env;
use std::thread;
use std::fs::File;
use std::sync::{Arc, Once, ONCE_INIT};
use std::cell::UnsafeCell;
use std::io::{self, BufReader, BufRead};
use std::ffi::CString;
use intercore::bus::{Ctx,Channel};
use reactors::boot::Boot;
use reactors::console::Console;
use reactors::selector::Selector;
use reactors::scheduler::Scheduler;
use handle::{self, Handle};
use queues::publisher::Publisher;
use queues::pubsub::PubSub;
use sys;

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
    scheds: Vec<Scheduler<'a>>,
}

impl<'a> Host<'a> {
    pub fn new() -> Self {
        let mut ctxs = Vec::new();
        ctxs.push(Rc::new(Ctx::new()));
        Host {
            args: args(),
            scheds: Vec::new(),
            boot: handle::new(Boot::new()),
        }
    }

    fn init(&mut self) -> io::Result<()> {
        let f = try!(File::open("./etc/init.boot"));
        let mut file = BufReader::new(&f);
        for line in file.lines() {
            let l = line.unwrap();
            println!("{}", l);
        }
        Ok(())
    }

    fn connect_w(c1: &mut Scheduler<'a>, c2: &mut Scheduler<'a>) {
        let s1 = c1.subscribe();
        let s2 = c2.subscribe();
        c1.add_subscriber(s2);
        c2.add_subscriber(s1);
    }

    fn connect_w_host(sched: &'a mut Scheduler<'static>) {
        let s0 = host().borrow_mut().boot.subscribe();
        sched.add_subscriber(s0);
        host().borrow_mut().boot.add_selected(Selector::Sb(sched.subscribe()));
    }

    fn connect_scheds(sched: &'a mut Scheduler<'static>) {
        Host::connect_w_host(sched);
        for s in &mut host().borrow_mut().scheds {
            Host::connect_w(s, sched);
        }
    }

    pub fn run(&mut self) {
        self.init();
        let mut o = Selector::Rx(Console::new());
        // let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
        // let mut w = Selector::Ws(WsServer::new(&addr));
        // self.boot.add_selected(w);
        self.boot.add_selected(o);
        Host::connect(&self.args);
        self.park_scheds();
        self.boot.init();
    }

    fn connect(args: &Args<'a>) {
        for i in 1..args.cores.expect("Please, specify number of cores.") {
            let chan = Channel {
                id: i,
                publisher: Publisher::with_mirror(CString::new(format!("/pub_{}", i)).unwrap(), 8),
                subscribers: Vec::new(),
            };
            let mut sched = Scheduler::new().set_channel(chan);
            Host::connect_scheds(&mut sched);
            host().borrow_mut().scheds.push(sched);
        }
    }

    pub fn park_scheds(&mut self) {
        for i in 0..self.args.cores.expect("Please, specify number of cores.") - 1 {
            thread::Builder::new()
                .name(format!("core_{}", i + 1))
                .spawn(move || {
                    sys::set_affinity(1 << i);
                    host().borrow_mut().scheds.get_mut(i).expect(&format!("There is scheduler at {:?}", i)).run();
                })
                .expect("Can't spawn new thread!");
        }
    }
}

#[derive(Clone)]
pub struct HostSingleton {
    pub inner: Arc<UnsafeCell<Host<'static>>>,
}

impl HostSingleton {
    pub fn borrow_mut(&mut self) -> &'static mut Host<'static> {
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
