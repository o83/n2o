use reactors::task::{Task, Context, TaskId, T3, Termination};
use reactors::job::Job;
use reactors::system::{IO, Async};
use reactors::cps::CpsTask;
use intercore::message::*;
use intercore::bus::{Memory, Channel, send};
use intercore::server::handle_intercore;
use queues::publisher::Publisher;
use std::{thread, time};
use std::ffi::CString;
use handle::{from_raw, into_raw, UnsafeShared};
use reactors::console::Console;
use reactors::selector::Selector;
use std::str;

const TASKS_MAX_CNT: usize = 256;

pub struct Scheduler<'a> {
    pub tasks: Vec<T3<Job<'a>>>,
    pub bus: Channel,
    pub queues: Memory,
    pub io: IO,
}

impl<'a> Scheduler<'a> {
    pub fn with_channel(id: usize) -> Self {
        let chan = Channel {
            id: id,
            publisher: //Publisher::with_mirror(CString::new(format!("/pub_{}", id)).unwrap(), 88),
                       // NOTE: with_mirror is not working in tests
                       Publisher::with_capacity(88),
            subscribers: Vec::new(),
        };
        Scheduler {
            tasks: Vec::with_capacity(TASKS_MAX_CNT),
            bus: chan,
            io: IO::new(),
            queues: Memory::new(),
        }
    }

    pub fn spawn(&'a mut self, t: Job<'a>, l: Termination, input: Option<&'a str>) -> TaskId {
        let last = self.tasks.len();
        self.tasks.push(T3(t, l));
        self.tasks.last_mut().expect("Scheduler: can't retrieve a task.").0.init(input, last);
        TaskId(last, self.bus.id)
    }

    pub fn exec(&'a mut self, t: TaskId, input: Option<&'a str>) {
        self.tasks.get_mut(t.0).expect("Scheduler: can't retrieve a task.").0.exec(input);
    }

    #[inline]
    fn terminate(&'a mut self, t: Termination, i: usize) {
        if t == Termination::Recursive {
            self.tasks.remove(i);
        }
    }

    pub fn poll_bus(&mut self) {
        let x = into_raw(self);
        for s in &from_raw(x).bus.subscribers {
            handle_intercore(from_raw(x), s.recv(), &mut from_raw(x).bus);
            s.commit();
        }
    }

    pub fn handle_shell(&mut self, buf: Option<&'a str>, shell: TaskId) {
        if let Some(x) = buf {
            send(&self.bus, Message::Exec(shell.0, x.to_string()));
        }
    }

    pub fn hibernate(&mut self) {
        thread::sleep(time::Duration::from_millis(10)); // Green Peace
    }

    #[inline]
    fn poll_tasks(&mut self) {
        let a = into_raw(self);
        let l = from_raw(a).tasks.len();
        for i in 1..l {
            from_raw(a).tasks[i].0.poll(Context::Nil, from_raw(a));
        }
    }

    pub fn mem(&mut self) -> UnsafeShared<Memory> {
        unsafe { UnsafeShared::new(&mut self.queues as *mut Memory) }
    }

    pub fn run0(&mut self, input: Option<&'a str>) {
        println!("BSP core {:?}", self.bus.id);
        self.io.spawn(Selector::Rx(Console::new()));
        let x = into_raw(self);
        let shell = from_raw(x).spawn(Job::Cps(CpsTask::new(self.mem())),
                                      Termination::Corecursive,
                                      input);

        self.handle_shell(input, shell);

        loop {
            self.poll_bus();
            match from_raw(x).io.poll() {
                Async::Ready((_, buf)) => self.handle_shell(from_raw(x).io.cmd(buf), shell),
                _ => (),
            }
            self.hibernate();
            self.poll_tasks();
        }
    }

    pub fn run(&mut self) {
        println!("AP core {:?}", self.bus.id);
        loop {
            self.poll_bus();
            self.hibernate();
            self.poll_tasks();
        }
    }
}
