
use commands::ast::{self, AST, Error};
use streams::interpreter;
use streams::interpreter::*;
use streams::env::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Lambda {
    names: AST,
    args: AST,
    cont: Cont,
    val: AST,
    env: Rc<RefCell<Environment>>,
}

pub fn new(names: AST, args: AST, env: Rc<RefCell<Environment>>, val: AST, cont: Cont) -> Lambda {
    Lambda {
        names: names,
        args: args,
        val: val,
        cont: cont,
        env: env,
    }
}

impl Iterator for Lambda {
    type Item = Result<Lazy, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let local_env = Environment::new_child(self.env.clone());
        for (name, value) in self.names.clone().into_iter().zip(self.args.clone().into_iter()) {
            local_env.borrow_mut().define(ast::extract_name(name), value);
        }
        Some(interpreter::evaluate_expr(self.val.clone(), local_env, box self.cont.clone()))
    }
}

impl<'a> Iterator for &'a Lambda {
    type Item = Result<Lazy, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let local_env = Environment::new_child(self.env.clone());
        for (name, value) in self.names.clone().into_iter().zip(self.args.clone().into_iter()) {
            local_env.borrow_mut().define(ast::extract_name(name), value);
        }
        Some(interpreter::evaluate_expr(self.val.clone(), local_env, box self.cont.clone()))
    }
}
