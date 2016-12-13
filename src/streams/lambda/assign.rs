
use commands::ast::{self, AST, Error};
use streams::interpreter::*;
use streams::interpreter;
use std::rc::Rc;
use std::cell::RefCell;
use streams::env::*;

pub struct Assign {
    var: AST<'ast>,
    args: AST<'ast>,
    env: Rc<RefCell<Environment>>,
    val: AST<'ast>,
    cont: Cont,
}

pub fn new(var: AST<'ast>,
           args: AST<'ast>,
           env: Rc<RefCell<Environment>>,
           val: AST<'ast>,
           cont: Cont)
           -> Assign {
    Assign {
        var: var,
        args: args,
        env: env,
        val: val,
        cont: cont,
    }
}

impl Iterator for Assign {
    type Item = Result<Lazy, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.var.clone() {
            AST::NameInt(s) => {
                self.env.borrow_mut().define(s, self.val.clone());
                Some(interpreter::evaluate_expr(self.val.clone(),
                                                self.env.clone(),
                                                box self.cont.clone()))
            }
            x => {
                Some(Err(Error::EvalError {
                    desc: "Can assign only to var".to_string(),
                    ast: x,
                }))
            }

        }
    }
}

impl<'a> Iterator for &'a Assign {
    type Item = Result<Lazy, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.var.clone() {
            AST::NameInt(s) => {
                self.env.borrow_mut().define(s, self.val.clone());
                Some(interpreter::evaluate_expr(self.val.clone(),
                                                self.env.clone(),
                                                box self.cont.clone()))
            }
            x => {
                Some(Err(Error::EvalError {
                    desc: "Can assign only to var".to_string(),
                    ast: x,
                }))
            }

        }
    }
}
