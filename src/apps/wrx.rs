extern crate kernel;

use std::thread;
use kernel::reactors::scheduler::Scheduler;
use kernel::intercore::bus::Channel;
use kernel::sys;
use kernel::args;
use std::fs::File;
use std::io::Read;

pub fn star<'a>(channel_num: usize) -> Vec<Channel> {
    let mut channels: Vec<Channel> = Vec::new();
    for i in 0..channel_num {
        let mut channel = Channel::new(i, 12);//TODO: use cap as param
        for c in &mut channels {
            c.subscribers.push(channel.publisher.subscribe());
            channel.subscribers.push(c.publisher.subscribe());
        }
        channel.subscribers.push(channel.publisher.subscribe());
        channels.push(channel);
    }
    channels
}


pub fn park<'a>(mut channels: Vec<Channel>) -> Scheduler<'a> {
    let sz = channels.len();
    for id in 1..sz {
        if let Some(mut channel) = channels.pop() {
            thread::spawn(move || {
                let mut sched = Scheduler::with_channel2(channel);
                sched.run();
            });
        }
    }
    let zero = channels.pop().expect("No BSP");
    Scheduler::with_channel2(zero)
}


fn main() {
    let mut p = args::Parser::new();
    let f = p.get("-init", true);
    let mut inp = String::new();
    let input = match f {
        Ok(i) => {
            let file = File::open(i.expect("A real filename expected."));
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
