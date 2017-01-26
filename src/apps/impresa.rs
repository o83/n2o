#[macro_use]
extern crate kernel;

use std::thread;
use kernel::reactors::scheduler::Scheduler;
use kernel::sys;
use kernel::args;
use std::fs::File;
use std::io::Read;

pub fn star<'a>(sched_num: usize) -> Vec<Scheduler<'a>> {
    let mut scheds: Vec<Scheduler<'a>> = Vec::new();
    for i in 0..sched_num {
        let mut sched = Scheduler::with_channel(i);
        for s in &mut scheds {
            s.bus.subscribers.push(sched.bus.publisher.subscribe());
            sched.bus.subscribers.push(s.bus.publisher.subscribe());
        }
        sched.bus.subscribers.push(sched.bus.publisher.subscribe());
        scheds.push(sched);
    }
    scheds
}


pub fn park<'a>(mut scheds: Vec<Scheduler<'a>>) -> Scheduler<'a> {
    let sz = scheds.len();
    for id in 1..sz {
        if let Some(mut core) = scheds.pop() {
            unsafe {
                spawn_on(id, move || {
                    core.run();
                });
            }
        }
    }
    scheds.pop().expect("No BSP")
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<Self>) {
        (*self)()
    }
}

// Like `std::thread::spawn`, but without the closure bounds.
pub unsafe fn spawn_on<'a, F>(id: usize, f: F) -> thread::JoinHandle<()>
    where F: FnOnce() + Send + 'a
{
    use std::mem;
    let closure: Box<FnBox + 'a> = Box::new(f);
    let closure: Box<FnBox + Send> = mem::transmute(closure);
    thread::Builder::new()
        .name(format!("core_{}", id))
        .spawn(move || {
            sys::set_affinity(1 << id);
            closure.call_box()
        })
        .expect("Can't spawn new thread!")
}

fn main() {
    let mut p = args::Parser::new();
    let f = p.get("-init", true);
    let mut inp = String::new();
    let input = match f {
        Ok(i) => {
            let mut file = File::open(i.expect("A real filename expected."));
            file.expect(&format!("Can't open file {:?}.", i))
                .read_to_string(&mut inp)
                .expect(&format!("Can't load src: {:?}", f));
            Some(&inp[..])
        }
        Err(e) => {
            println!("{:?}", e);
            None
        }
    };
    park(star(4)).run0(input);
}
