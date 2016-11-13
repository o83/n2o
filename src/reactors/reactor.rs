
// Task Reactor with Priorities by Anton

use std::cmp::Ordering;
use reactors::streams::stream::Stream;

pub struct Reactor<S> {
    tasks: Vec<S>,
    run: bool,
}

impl<S: Stream> Reactor<S> {
    pub fn new() -> Self {
        Reactor {
            tasks: Vec::new(),
            run: true,
        }
    }

    pub fn drop() {
        // drop task from list
    }

    pub fn spawn<'a>(&mut self, t: S) {
        self.tasks.push(t);
    }

    fn reorder(&mut self, slice: &mut [i32]) {
        if slice.len() <= 1 {
            return;
        }
        let partition_idx = self.partition(slice);
        self.reorder(&mut slice[..partition_idx]);
        self.reorder(&mut slice[partition_idx + 1..]);
    }

    fn partition(&mut self, slice: &mut [i32]) -> usize {
        let mut partition_idx = 0;
        for i in 1..slice.len() {
            match slice[i].cmp(&slice[partition_idx]) {
                Ordering::Less => {
                    slice.swap(i, partition_idx + 1);
                    slice.swap(partition_idx, partition_idx + 1);
                    partition_idx += 1;
                }
                _ => (),
            }
        }
        return partition_idx;
    }

    pub fn run(&mut self) {
        while self.run {
            // process tasks by their priorities:
            for t in &mut self.tasks {
                let res = t.poll();
                match res {
                    Ok(None) => {
                        self.run = false;
                        break;
                    }
                    _ => continue,
                }
            }
        }
    }
}
