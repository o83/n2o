extern crate time;
extern crate kernel;

use std::thread;
use std::u64;
use std::sync::mpsc::channel;
use time::precise_time_ns;
use kernel::queues::enso::Enso;

fn bench_enso_one2n(iterations: u64, consumers: usize, capacity: usize) {
    let mut enso: Enso<u64> = Enso::with_capacity(capacity);
    let (tx, rx) = channel::<u64>();

    for t in 0..consumers {
        let cons = enso.new_consumer();
        let tx_c = tx.clone();
        thread::spawn(move|| {
            let start = precise_time_ns();
            //let mut expected = 0u64;
            'outer: loop {
                'inner: loop {
                    match cons.recv() {
                        Some(v) => {
                            if *v == u64::MAX {
                                let _ = tx_c.send(*v);
                                cons.release();
                                break 'outer;
                            }
                            //assert!(*v == expected);
                            //expected += 1;
                            cons.release();
                            break 'inner;
                        },
                        None => {}
                    }
                }
            }
            let stop = precise_time_ns();
            let ns = stop - start;
            println!("cons {} recved {} msgs in {}ns. {}ns/msg", t, iterations, ns, ns / iterations);
        });
    }

    let start = precise_time_ns();
    for i in 0..iterations {
        loop {
            match enso.next() {
                Some(v) => {
                    *v = i as u64;
                    enso.flush();
                    break;
                },
                None => {}
            }
        }
    }

    loop {
        match enso.next() {
            Some(v) => {
                *v = u64::MAX;
                enso.flush();
                break;
            },
            None => {}
        }
    }
    let stop = precise_time_ns();
    let ns = stop - start;
    println!("sent {} msgs in {}ns. {}ns/msg", iterations, ns, ns / iterations);
    for _ in 0..consumers {
        let _ = rx.recv(); //wait for readers
    }
    println!("Done!");
}

fn main() {
    bench_enso_one2n(10_000_000, 4, 2048 * 1024);
}
