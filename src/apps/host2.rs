#[macro_use]
extern crate kernel;
use std::{thread, time};
use kernel::reactors::scheduler::Scheduler;
use kernel::sys;
use std::sync::Arc;
use std::io::{self, BufReader, BufRead};
use kernel::reactors::console::Console;
use kernel::reactors::selector::{Selector, Async, Pool};
use kernel::reactors::system::IO;


fn construct_scheds<'a>(sched_num: usize) -> Vec<Scheduler<'a>> {
    let mut scheds: Vec<Scheduler<'a>> = Vec::new();
    for i in 0..sched_num {
        let mut sched = Scheduler::with_channel(i);
        for s in &mut scheds {
            s.bus.subscribers.push(sched.bus.publisher.subscribe());
            sched.bus.subscribers.push(s.bus.publisher.subscribe());
        }
        scheds.push(sched);
    }
    scheds
}


fn park_scheds<'a>(mut scheds: Vec<Scheduler<'a>>) {
    let sz = scheds.len();
    for id in 0..sz {
        if let Some(mut s) = scheds.pop() {
            unsafe {
                spawn_on(id, || {
                    s.run();
                });
            }
        }
    }
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
    println!("spawn on core_id {:?}", id);
    thread::Builder::new()
        .name(format!("core_{}", id))
        .spawn(move || {
            sys::set_affinity(1 << id);
            closure.call_box()
        })
        .expect("Can't spawn new thread!")
}

fn io_loop(io: IO) {}

fn main() {
    park_scheds(construct_scheds(5));
    loop {
        thread::sleep(time::Duration::from_millis(10000));
    }
    print!("done!");
    // let mut io = IO::new();
    // let mut o = Selector::Rx(Console::new());
    // io.spawn(o);
    // loop {
    //     thread::sleep(time::Duration::from_millis(100));
    //     match io.poll() {
    //         Async::Ready((_, Pool::Raw(buf))) => println!("Raw: {:?}", buf),
    //         Async::Ready((_, _)) => (),
    //         Async::NotReady => (),
    //     }
    // }
}