use reactors::task::{self, Task, Context, Poll, Error};
use streams::interpreter::*;
use commands::ast::AST;
use handle::*;
use std::rc::Rc;
use streams::intercore::ctx::{Ctx, Ctxs};

pub struct IprTask<'a> {
    interpreter: Interpreter<'a>,
    ast: Option<&'a AST<'a>>,
}

impl<'a> IprTask<'a> {
    pub fn new(ctx: Rc<Ctx<u64>>) -> Self {
        IprTask {
            interpreter: Interpreter::new2(ctx).unwrap(),
            ast: None,
        }
    }
}

impl<'a> Task<'a> for IprTask<'a> {
    fn init(&'a mut self, input: Option<&'a str>) {
        let (s1, s2) = split(self);
        s1.interpreter.define_primitives();
        if input.is_some() {
            let s = input.unwrap().to_string();
            s2.ast = Some(s2.interpreter.parse(&s));
        }
    }

    fn poll(&'a mut self, c: Context<'a>) -> Poll<Context<'a>, Error> {
        match c {
            Context::Node(n) => {
                match self.interpreter.run(n) {
                    Ok(r) => Poll::Yield(Context::Node(r)),
                    Err(e) => Poll::Err(Error::RuntimeError),
                }
            }
            Context::Nil => {
                match self.interpreter.run(self.ast.unwrap()) {
                    Ok(r) => Poll::Yield(Context::Node(r)),
                    Err(e) => Poll::Err(Error::RuntimeError),
                }
            }
            _ => Poll::Err(Error::WrongContext),
        }
    }
}