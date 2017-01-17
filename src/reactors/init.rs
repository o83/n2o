use libc;
use std::mem;
use std::rc::Rc;
use std::env;
use std::thread;
use std::fs::File;
use streams::intercore::ctx::Ctx;
use streams::intercore::api::{Message, Spawn};
use reactors::boot::Boot;
use handle::{self, Handle};
use std::sync::{Arc, Once, ONCE_INIT};
use std::cell::UnsafeCell;
use reactors::core::Core;
use reactors::console::Console;
use reactors::ws::WsServer;
use std::net::SocketAddr;
use reactors::selector::Selector;
use std::io::{self, BufReader, BufRead};
// use nix::sched::{self, CpuSet};
use reactors::intercoretask::IntercoreTask;
use reactors::scheduler::TaskTermination;
use reactors::job::Job;
use queues::publisher::{Publisher, Subscriber};
use std::ffi::CString;
use streams::intercore::ctx::Channel;

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

    fn init(&mut self) -> io::Result<()> {
        let f = try!(File::open("./etc/init.boot"));
        let mut file = BufReader::new(&f);
        for line in file.lines() {
            let l = line.unwrap();
            println!("{}", l);
        }
        Ok(())
    }

    fn connect_w(c1: &Core, c2: &Core) {
        // let s = c1.bus().publisher.subscribe();
        // c1.bus().subscribers.push(s);
        // let s = c1.bus().publisher.subscribe();
        // c2.bus().subscribers.push(s);
    }

    fn spawn_intercore_tasks() {
        println!("Cores count: {:?}", host().borrow_mut().cores.len());
        for (i, c) in host().borrow_mut().cores.iter_mut().enumerate() {
            // let p = Publisher::with_mirror(CString::new(format!("/ipc_{}", i)).unwrap(), 8);
            // let s: Vec<Subscriber<Message>> = Vec::new();
            // let mut j = Job::Ipc(IntercoreTask::new(i, p, s));

            // c.spawn_task(j, TaskTermination::Recursive, None);
        }
    }

    fn connect_cores(core: &'a Core) {
        // Host::connect_w(core, &host().borrow_mut().boot.borrow().core);
        for c in &host().borrow_mut().cores {
            // Host::connect_w(c, &core);
        }
    }

    pub fn run(&mut self) {
        self.init();
        let mut o = Selector::Rx(Console::new());
        let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
        let mut w = Selector::Ws(WsServer::new(&addr));
        self.boot.add_selected(o);
        self.boot.add_selected(w);
        Host::connect(&self.args);
        // Host::spawn_intercore_tasks();
        self.park_cores();
        // self.boot.core.publish(|p| {
        //     match p.next_n(3) {
        //         Some(vs) => {
        //             vs[0] = Message::Halt;
        //             vs[1] = Message::Unknown;
        //             vs[2] = Message::Spawn(Spawn { id: 13, id2: 42 });
        //             p.commit();
        //         }
        //         None => {}
        //     }
        // });
        self.boot.init();
    }

    fn connect(args: &Args<'a>) {
        for i in 1..args.cores.expect("Please, specify number of cores.") {
            let c = Channel {
                publisher: Publisher::with_mirror(CString::new(format!("/ipc_{}", i)).unwrap(), 8),
                subscribers: Vec::new(),
            };
            let c = Core::with_channel(i, c);
            // Host::connect_cores(&c);
            host().borrow_mut().cores.push(c);
        }
    }

    pub fn park_cores(&mut self) {
        for i in 1..self.args.cores.expect("Please, specify number of cores.") {
            thread::Builder::new()
                .name(format!("core_{}", i))
                .spawn(move || {
                    let id = unsafe { libc::pthread_self() as isize };
                    // let mut cpu = CpuSet::new();
                    // cpu.set(1 << i);
                    // sched::sched_setaffinity(id, &cpu);
                    for c in &mut host().borrow_mut().cores {
                        c.park();
                    }
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
