// Getter messages Stream from queue.

use streams::adverb::stream::*;

pub struct Proto {
    msg_type: u32,
}

pub struct Messages {
    queue_desc: u32,
    proto: Proto,
    cnt: u64,
}

pub fn new(p: u32, fd: u32) -> Messages {
    Messages {
        queue_desc: fd,
        proto: Proto { msg_type: p },
        cnt: 0,
    }
}

impl Iterator for Messages {
    type Item = Poll<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        // potentially read from queue
        self.cnt += 1;
        Some(Ok(Async::Ready(self.cnt)))
    }
}

impl<'a> Iterator for &'a Messages {
    type Item = Poll<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        // potentially read from queue
        let offset = 123;
        Some(Ok(Async::Ready(offset)))
    }
}