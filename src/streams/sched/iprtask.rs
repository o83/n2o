use streams::sched::task::{self, Task, Context, Poll, Error};
use streams::interpreter::*;

pub struct IprTask<'a> {
    interpreter: Interpreter<'a>,
}

impl<'a> IprTask<'a> {
    pub fn new() -> Self {
        IprTask { interpreter: Interpreter::new().unwrap() }
    }
}

impl<'a> Task<'a> for IprTask<'a> {
    fn init(&'a mut self) {
        self.interpreter.define_primitives();
    }
    fn poll(&mut self, c: Context<'a>) -> Result<Poll<Context<'a>>, Error> {
        // self.interpreter.run();
        // i + 1
        Err(Error::RuntimeError)
    }
}