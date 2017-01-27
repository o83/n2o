use reactors::task::{Task, Context, Poll, Error};
use streams::interpreter::*;
use commands::ast::AST;
use handle::*;
use intercore::bus::{send, Memory};
use reactors::scheduler::Scheduler;

pub struct CpsTask<'a> {
    pub interpreter: Interpreter<'a>,
    pub ast: Option<&'a AST<'a>>,
    task_id: usize,
}

impl<'a> CpsTask<'a> {
    pub fn new(mem_ptr: UnsafeShared<Memory>) -> Self {
        CpsTask {
            interpreter: Interpreter::new(mem_ptr).unwrap(),
            ast: None,
            task_id: 0,
        }
    }

    #[inline]
    fn run(&'a mut self, n: &'a AST<'a>, intercore: Context<'a>, sched: &'a Scheduler<'a>) -> Poll<Context<'a>, Error> {
        let r = self.interpreter.run(n, intercore, sched);
        match r {
            Ok(r) => {
                match *r {
                    AST::Yield(Context::Intercore(msg)) => {
                        send(&sched.bus, msg.clone());
                        Poll::Yield(Context::Nil)
                    }
                    AST::Yield(ref c) => Poll::Yield(c.clone()),
                    _ => Poll::End(Context::Node(r)),
                }
            }
            Err(e) => Poll::Err(Error::RuntimeError),
        }
    }
}

impl<'a> Task<'a> for CpsTask<'a> {
    fn init(&'a mut self, input: Option<&'a str>, task_id: usize) {
        let (s1, s2) = split(self);
        s1.interpreter.define_primitives();
        s2.interpreter.task_id = task_id;
        s2.task_id = task_id;
        println!("TaskId: {:?}", s2.task_id);
        match input {
            Some(i) => {
                let s = i.to_string();
                s2.ast = Some(s2.interpreter.parse(&s));
            }
            None => s2.ast = None,
        }
    }

    fn exec(&'a mut self, input: Option<&'a str>) {
        match input {
            Some(i) => {
                let s = i.to_string();
                let parse = self.interpreter.parse(&s);
                self.ast = Some(parse);
            }
            None => self.ast = None,
        }
    }

    fn poll(&'a mut self, c: Context<'a>, sched: &'a Scheduler<'a>) -> Poll<Context<'a>, Error> {
        match self.ast {
            Some(a) => {
                match c.clone() {
                    Context::Node(n) => self.run(n, c, sched),
                    Context::NodeAck(task_id, n) => self.run(a, c, sched),
                    Context::Nil => self.run(a, c, sched),
                    _ => Poll::Err(Error::WrongContext),
                }
            }
            None => Poll::End(Context::Nil),
        }
    }

    fn finalize(&'a mut self) {
        //
    }
}