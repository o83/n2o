use streams::sched::task::Task;

const TASKS_MAX_CNT: usize = 256;

pub struct Reactor<T> {
    tasks: Vec<T>,
}

impl<T> Reactor<T>
    where T: Task
{
    pub fn new() -> Self {
        Reactor { tasks: Vec::with_capacity(TASKS_MAX_CNT) }
    }

    pub fn spawn(&mut self, t: T) {
        self.tasks.push(t);
    }

    pub fn run(&mut self) {
        loop {
            for t in &mut self.tasks {
                // let r = t.poll(0 as usize);
                // print!("Task Poll: {:?}", r);
            }
        }
    }
}