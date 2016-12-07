extern crate time;
extern crate kernel;
extern crate libc;

use std::thread;
use std::u64;
use std::sync::mpsc::channel;
use time::precise_time_ns;
use kernel::queues::publisher::Publisher;
use std::ffi::CString;

fn bench_publisher_one2n(iterations: u64, consumers: usize, capacity: usize) {
    let mut publisher: Publisher<u64> = Publisher::with_capacity(capacity);
    //let mut publisher: Publisher<u64> = Publisher::with_mirror(CString::new("/test").unwrap(), capacity);
    let (tx, rx) = channel::<u64>();

    for t in 0..consumers {
        let cons = publisher.subscribe();
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
                                cons.commit();
                                break 'outer;
                            }
                            //assert!(*v == expected);
                            //expected += 1;
                            cons.commit();
                            break 'inner;
                        },
                        None => {}
                    }
                }
            }
            let stop = precise_time_ns();
            let ns = stop - start;
            println!("cons {} recved {} msgs in {}ns. {}ns/msg", t, iterations, ns, ns / iterations);
            println!("{:?}", cons);
        });
    }

    let start = precise_time_ns();
    for i in 0..iterations {
        loop {
            match publisher.next() {
                Some(v) => {
                    *v = i as u64;
                    publisher.commit();
                    break;
                },
                None => {}
            }
        }
    }

    loop {
        match publisher.next() {
            Some(v) => {
                *v = u64::MAX;
                publisher.commit();
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
    bench_publisher_one2n(10_000_000, 2, 10_000_000);
}
