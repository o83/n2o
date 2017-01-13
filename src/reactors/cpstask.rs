use reactors::task::{Task, Context, Poll, Error};
use streams::interpreter::*;
use commands::ast::AST;
use handle::*;
use std::rc::Rc;
use streams::intercore::ctx::Ctx;

pub struct CpsTask<'a> {
    interpreter: Interpreter<'a>,
    ast: Option<&'a AST<'a>>,
}

impl<'a> CpsTask<'a> {
    pub fn new(ctx: Rc<Ctx>) -> Self {
        CpsTask {
            interpreter: Interpreter::new2(ctx).unwrap(),
            ast: None,
        }
    }

    #[inline]
    fn run(&'a mut self, n: &'a AST<'a>) -> Poll<Context<'a>, Error> {
        let r = self.interpreter.run(n);
        match r {
            Ok(r) => {
                match *r {
                    AST::Yield => Poll::Yield(Context::Nil),
                    _ => Poll::End(Context::Node(r)),
                }
            }
            Err(e) => Poll::Err(Error::RuntimeError),
        }
    }
}

impl<'a> Task<'a> for CpsTask<'a> {
    fn init(&'a mut self, input: Option<&'a str>) {
        let (s1, s2) = split(self);
        s1.interpreter.define_primitives();
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
                match c {
                    Context::Node(n) => self.run(n),
                    Context::Nil => self.run(a),
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