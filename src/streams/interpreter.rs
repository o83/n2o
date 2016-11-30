
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

#[derive(PartialEq, Clone)]
pub enum List {
    Cons(Box<Value>, Box<List>),
    Nil,
}

impl List {
    pub fn shift(self) -> Option<(Value, List)> {
        match self {
            List::Cons(car, cdr) => Some((*car, *cdr)),
            List::Nil => None,
        }
    }
    fn to_vec(self) -> Vec<Value> {
        let mut out = vec![];
        let mut l = self;
        loop {
            match l.shift() {
                Some((car, cdr)) => {
                    out.push(car);
                    l = cdr;
                }
                None => break,
            }
        }
        out
    }
}

impl iter::IntoIterator for List {
    type Item = Value;
    type IntoIter = vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strs: Vec<String> = self.clone().into_iter().map(|v| format!("{}", v)).collect();
        write!(f, "({})", &strs.connect(" "))
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strs: Vec<String> = self.clone().into_iter().map(|v| format!("{:?}", v)).collect();
        write!(f, "({})", &strs.connect(" "))
    }
}

#[derive(PartialEq, Clone)]
pub enum Value {
    Symbol(String),
    Integer(i64),
    Boolean(bool),
    String(String),
    List(List),
    Lambda(List, AST),
    Continuation(AST),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Symbol(ref val) => write!(f, "{}", val),
            Value::Integer(val) => write!(f, "{}", val),
            Value::Boolean(val) => write!(f, "#{}", if val { "t" } else { "f" }),
            Value::String(ref val) => write!(f, "{}", val),
            Value::List(ref list) => write!(f, "List'{}", list),
            Value::Lambda(_, _) => write!(f, "#<procedure>"),
            Value::Continuation(_) => write!(f, "#<continuation>"),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::String(ref val) => write!(f, "\"{}\"", val),
            Value::List(ref list) => write!(f, "{:?}", list),
            _ => write!(f, "{}", self),
        }
    }
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


impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent {
            Some(ref parent) => write!(f, "{:?}", self.values),
            None => write!(f, "{:?} ", self.values),
        }
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent {
            Some(ref parent) => write!(f, "{:?}", self.values),
            None => write!(f, "{:?} ", self.values),
        }
    }
}

#[derive(Clone)]
pub enum Trampoline {
    Bounce(Value, Rc<RefCell<Environment>>, Continuation),
    QuasiBounce(Value, Rc<RefCell<Environment>>, Continuation),
    Run(Value, Continuation),
    Land(Value),
}

impl fmt::Display for Trampoline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Trampoline::Bounce(ref value, ref env, ref cc) => {
                let a = unsafe { env.as_unsafe_cell().get() };
                write!(f, "Bounce {} env {}", value, unsafe { &*a })
            }
            Trampoline::QuasiBounce(ref value, ref env, ref cc) => {
                write!(f, "QuasiBounce {}", value)
            }
            Trampoline::Run(ref value, ref cc) => write!(f, "Run {}", value),
            Trampoline::Land(ref value) => write!(f, "Land {}", value),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Continuation {
    EvaluateExpressions(AST, Rc<RefCell<Environment>>, Box<Continuation>),
    BeginFunc(AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateCond(AST, Value, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateLambda(String, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateFunc(Value, AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateLet(String, AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    ExecuteEval(Rc<RefCell<Environment>>, Box<Continuation>),
    ExecuteApply(AST, Box<Continuation>),
    Return,
}
