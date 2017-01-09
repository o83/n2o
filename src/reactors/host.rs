use std::rc::Rc;
use reactors::scheduler::Scheduler;
use streams::intercore::ctx::Ctxs;
use reactors::hub::Hub;
use std::mem;
use handle::*;

pub struct Host<'a, T: 'a, C: 'a> {
    cores: Vec<Scheduler<'a, T>>,
    junk: Hub<'a>,
    ring: Rc<Ctxs<C>>,
}
