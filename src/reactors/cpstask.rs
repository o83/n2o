use reactors::task::{Task, Context, Poll, Error};
use streams::interpreter::*;
use commands::ast::AST;
use handle::*;
use std::rc::Rc;
use intercore::bus::Ctx;

pub struct CpsTask<'a> {
    interpreter: Interpreter<'a>,
    task_id: usize,
    ast: Option<&'a AST<'a>>,
}

impl<'a> CpsTask<'a> {
    pub fn new(ctx: Rc<Ctx>) -> Self {
        CpsTask {
            interpreter: Interpreter::new2(ctx).unwrap(),
            task_id: 0,
            ast: None,
        }
    }

    #[inline]
    fn run(&'a mut self, n: &'a AST<'a>, xchg: Context<'a>) -> Poll<Context<'a>, Error> {
//        match xchg.clone() {
//            Context::Nil => return Poll::Yield(Context::Nil),
//                       x => { println!("Context: {:?}", x); }
//        }
        let r = self.interpreter.run(n, xchg);
        match r {
            Ok(r) => {
                match *r {
                    AST::Yield(ref c) => {
                       Poll::Yield(c.clone())
                    }
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

    fn poll(&'a mut self, c: Context<'a>) -> Poll<Context<'a>, Error> {
        match self.ast {
            Some(a) => {
                match c.clone() {
                    Context::Node(n) => self.run(n, c),
                    Context::NodeAck(task_id, n) => { println!("CpsTasl::poll NodeAck {:?}",n); self.run(a, c) },
                    Context::Nil => self.run(a, c),
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