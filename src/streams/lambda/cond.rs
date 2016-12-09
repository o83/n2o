
use commands::ast::{self, AST, Error};
use streams::interpreter;
use streams::interpreter::*;
use streams::env::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Cond {
    left: AST,
    right: AST,
    cont: Cont,
    val: AST,
    env: Rc<RefCell<Environment>>,
}

pub fn new(left: AST, right: AST, env: Rc<RefCell<Environment>>, val: AST, cont: Cont) -> Cond {
    Cond {
        left: left,
        right: right,
        val: val,
        cont: cont,
        env: env,
    }
}

impl Iterator for Cond {
    type Item = Result<Lazy, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.val.clone() {
            AST::Number(0) => {
                Ok(Lazy::Defer(self.left.clone(), self.env.clone(), self.cont.clone()))
            }
            AST::Number(_) => {
                Ok(Lazy::Defer(self.right.clone(), self.env.clone(), self.cont.clone()))
            }
            x => {
                Ok(Lazy::Defer(x,
                               self.env.clone(),
                               Cont::Lambda(Code::Cond,
                                            self.left.clone(),
                                            self.right.clone(),
                                            self.env.clone(),
                                            box self.cont.clone())))
            }
        })
    }
}

impl<'a> Iterator for &'a Cond {
    type Item = Result<Lazy, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.val.clone() {
            AST::Number(0) => {
                Ok(Lazy::Defer(self.left.clone(), self.env.clone(), self.cont.clone()))
            }
            AST::Number(_) => {
                Ok(Lazy::Defer(self.right.clone(), self.env.clone(), self.cont.clone()))
            }
            x => {
                Ok(Lazy::Defer(x,
                               self.env.clone(),
                               Cont::Lambda(Code::Cond,
                                            self.left.clone(),
                                            self.right.clone(),
                                            self.env.clone(),
                                            box self.cont.clone())))
            }
        })
    }
}
