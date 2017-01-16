use std::rc::Rc;
use streams::intercore::ctx::Ctx;
use reactors::system::IO;
use reactors::selector::{Selector, Async, Pool};
use reactors::scheduler::{Scheduler, TaskTermination, TaskId};
use reactors::cpstask::CpsTask;
use handle;
use std::str;
use reactors::core::Core;

pub struct Boot<'a> {
    pub core: Core<'a>,
    io: IO,
    scheduler: Scheduler<'a, CpsTask<'a>>,
    ctx: Rc<Ctx>,
}

impl<'a> Boot<'a> {
    pub fn new(ctx: Rc<Ctx>) -> Self {
        Boot {
            core: Core::new(0), // predefined 0 id
            io: IO::new(),
            scheduler: Scheduler::new(),
            ctx: ctx,
        }
    }

    pub fn add_selected(&mut self, s: Selector) {
        self.io.spawn(s);
    }

    #[inline]
    fn handle_raw(&'a mut self, t: TaskId, b: &'a [u8]) {
        if b.len() == 0 {
            return;
        }
        if b.len() == 1 && b[0] == 0x0A {
            self.io.write_all(&[0u8; 0]);
            return;
        }
        let x = str::from_utf8(b).unwrap();
        let (s1, s2) = handle::split(self);
        s1.scheduler.exec(t, Some(x));
        let r = s2.scheduler.run();
        s2.io.write_all(format!("{:?}\n", r).as_bytes());
    }

    #[inline]
    fn ready(&'a mut self, p: Pool<'a>, t: TaskId) {
        match p {
            Pool::Raw(b) => self.handle_raw(t, b),            
            Pool::Msg(x) => println!("Intercore: {:?}", x.buf),
        }
    }

    pub fn init(&mut self) {
        let cps = CpsTask::new(self.ctx.clone());
        let h: *mut Boot<'a> = self;
        let h0: &mut Boot<'a> = unsafe { &mut *h };
        let task_id = h0.scheduler.spawn(cps, TaskTermination::Corecursive, None);
        loop {
            let h1: &mut Boot<'a> = unsafe { &mut *h };
            let h2: &mut Boot<'a> = unsafe { &mut *h };
            match h1.io.poll() {
                Async::Ready((_, p)) => h2.ready(p, task_id),
                Async::NotReady => (),
            }
        }
    }
}