use streams::sched::task::{self, Task};
use streams::interpreter::*;

pub struct IprTask<'a> {
    interpreter: Interpreter<'a>,
}

impl IprTask {
    pub fn new() -> Self {
        let mut i = Interpreter::new().unwrap();
        i.define_primitives();
        IprTask { interpreter: i }
    }
}

impl<'a> Task for IprTask<'a> {
    type Item = usize;
    type Error = task::Error;

    fn poll(&mut self, i: Self::Item) -> Result<Poll<Self::Item>, Error> {
        // self.interpreter.run();
        i + 1
    }
}