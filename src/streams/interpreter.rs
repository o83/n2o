
use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;
use streams::verb::plus;

#[derive(PartialEq)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum List {
    Cons(Box<Value>, Box<List>),
    Nil,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Symbol(String),
    Integer(i64),
    Boolean(bool),
    String(String),
    List(List),
    Lambda(List, AST),
}

#[derive(Clone)]
pub struct Interpreter {
    root: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Result<Interpreter, Error> {
        let env = try!(Environment::new_root());
        Ok(Interpreter { root: env })
    }

    pub fn run(&mut self, program: AST) -> Result<Value, Error> {
        match program {
            AST::Verb(vt, box lv, box rv) => {
                match vt {
                    Verb::Plus => {
                        let mut a = plus::new(lv, rv);
                        Ok(a.next().unwrap().unwrap().unwrap())
                    }
                    x => {
                        Err(Error::EvalError {
                            desc: format!("Not implemented Verb: {:?}", &x).to_string(),
                            ast: AST::Verb(x, box lv, box rv),
                        })
                    } 
                }
            }
            x => {
                Err(Error::EvalError {
                    desc: format!("Not implemented AST node: {:?}", &x).to_string(),
                    ast: x,
                })
            } 
        }
    }
}

impl Environment {
    fn new_root() -> Result<Rc<RefCell<Environment>>, Error> {
        let mut env = Environment {
            parent: None,
            values: HashMap::new(),
        };
        Ok(Rc::new(RefCell::new(env)))
    }

    fn new_child(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let env = Environment {
            parent: Some(parent),
            values: HashMap::new(),
        };
        Rc::new(RefCell::new(env))
    }
}
