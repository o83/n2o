
use commands::ast::{self, AST, Error};
use streams::interpreter::*;
use streams::interpreter;
use std::rc::Rc;
use std::cell::RefCell;
use streams::env::*;

pub struct Call {
    callee: AST<'ast>,
    args: AST<'ast>,
    env: Rc<RefCell<Environment>>,
    cont: Cont,
}

pub fn new(callee: AST<'ast>, args: AST<'ast>, env: Rc<RefCell<Environment>>, cont: Cont) -> Call {
    Call {
        callee: callee,
        args: args,
        env: env,
        cont: cont,
    }
}

impl Iterator for Call {
    type Item = Result<Lazy, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.args.clone() {
            AST::Dict(box v) => {
                Some(interpreter::evaluate_fun(self.callee.clone(),
                                               self.env.clone(),
                                               v,
                                               self.cont.clone()))
            }
            x => {
                Some(interpreter::evaluate_fun(self.callee.clone(),
                                               self.env.clone(),
                                               x,
                                               self.cont.clone()))
            }
        }
    }
}

impl<'a> Iterator for &'a Call {
    type Item = Result<Lazy, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.args.clone() {
            AST::Dict(box v) => {
                Some(interpreter::evaluate_fun(self.callee.clone(),
                                               self.env.clone(),
                                               v,
                                               self.cont.clone()))
            }
            x => {
                Some(interpreter::evaluate_fun(self.callee.clone(),
                                               self.env.clone(),
                                               x,
                                               self.cont.clone()))
            }
        }
    }
}
