use std::rc::Rc;
use streams::intercore::ctx::Ctx;
use queues::publisher::Publisher;
use queues::publisher::Subscriber;
use reactors::core::Core;
use reactors::selector::{Selector, Slot, Async, Pool};
use reactors::console::Console;
use reactors::ws::WsServer;
use reactors::task::Task;
use reactors::cpstask::CpsTask;
use reactors::scheduler::{self, Scheduler, TaskTermination};
use std::io::Read;
use handle;
use std::str;
use streams::intercore::api::*;

pub struct Hub<'a> {
    core: Core,
    scheduler: Scheduler<'a, CpsTask<'a>>,
    ctx: Rc<Ctx>,
    intercore: Slot,
}

impl<'a> Hub<'a> {
    pub fn new(ctx: Rc<Ctx>) -> Self {
        Hub {
            core: Core::new(),
            scheduler: Scheduler::new(),
            ctx: ctx,
            intercore: Slot(!0 as usize),
        }
    }

    pub fn add_intercore(&mut self, s: Selector) {
        let slot = self.core.spawn(s);
        self.intercore = slot;
    }

    pub fn add_selected(&mut self, s: Selector) {
        self.core.spawn(s);
    }

    pub fn boil(&mut self) {
        let cps = CpsTask::new(self.ctx.clone());
        let h: *mut Hub<'a> = self;
        let h0: &mut Hub<'a> = unsafe { &mut *h };
        let task_id = h0.scheduler.spawn(cps, TaskTermination::Corecursive, None);
        loop {
            let h1: &mut Hub<'a> = unsafe { &mut *h };
            match h1.core.poll() {
                Async::Ready((s, p)) => {
                    match p {
                        Pool::Raw(b) => {
                            let h2: &mut Hub<'a> = unsafe { &mut *h };
                            let h3: &mut Hub<'a> = unsafe { &mut *h };
                            let h4: &mut Hub<'a> = unsafe { &mut *h };
                            if b.len() == 1 && b[0] == 0x0A {
                                h2.core.write_all(&[0u8; 0]);
                            } else {
                                if s == self.intercore {
                                    // Here will be intercore messages handling
                                    self.core.write_all(format!("Intercore msg: {:?}\n", b).as_bytes());
                                } else {
                                    let x = str::from_utf8(b).unwrap();
                                    h3.scheduler.exec(task_id, Some(x));
                                    let r = h4.scheduler.run();
                                    self.core.write_all(format!("{:?}\n", r).as_bytes());
                                }
                            }
                        }
                        _ => Async::NotReady,
                    }
                }
                Async::NotReady => (),
            }
        }
    }
}