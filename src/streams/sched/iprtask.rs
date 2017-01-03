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

    fn poll(&'a mut self, c: Context<'a>) -> Poll<Context<'a>, Error> {
        match c {
            Context::Node(n) => {
                match self.interpreter.run(n) {
                    Ok(r) => Poll::Yield(Context::Node(r)),
                    Err(e) => Poll::Err(Error::RuntimeError),
                }
            }            
            _ => Poll::Err(Error::WrongContext),
        }
    }
}