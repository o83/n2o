
use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;

#[derive(PartialEq)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}

#[derive(PartialEq, Clone)]
pub enum List {
    Cons(Box<Value>, Box<List>),
    Nil,
}

#[derive(PartialEq, Clone)]
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
    root: Rc<RefCell<Environment>>
}

impl Interpreter {
    pub fn new() -> Result<Interpreter, Error> {
        let env = try!(Environment::new_root());
        Ok(Interpreter { root: env })
    }

    pub fn run(&self, program: AST) -> Result<Value, Error> {
        process(program, self.root.clone())
    }
}

pub fn process(program: AST, env: Rc<RefCell<Environment>>) -> Result<Value, Error> {
   Ok(Value::Integer(100))
}
