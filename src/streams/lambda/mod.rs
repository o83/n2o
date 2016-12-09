pub mod func;
pub mod call;
pub mod cond;
pub mod assign;
use streams::env::*;
use streams::interpreter::*;
use std::rc::Rc;
use std::cell::RefCell;
use commands::ast::*;
use commands::ast;

pub fn eval(code: Code,
            left: AST,
            right: AST,
            env: Rc<RefCell<Environment>>,
            val: AST,
            cont: Cont)
            -> Result<Lazy, Error> {
    match code {
        Code::Call => {
            let mut a = call::new(left, val, env, cont);
            a.next().unwrap()
        }
        Code::Func => {
            let mut a = func::new(left, right, env, val, cont);
            a.next().unwrap()
        }
        Code::Cond => {
            let mut a = cond::new(right, left, env, val, cont);
            a.next().unwrap()
        }
        Code::Assign => {
            let mut a = assign::new(left, right, env, val, cont);
            a.next().unwrap()
        }
    }
}