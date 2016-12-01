
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
    values: HashMap<String, AST>,
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

    pub fn run(&mut self, program: AST) -> Result<AST, Error> {
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

fn evaluate_expressions(exprs: AST,
                        env: Rc<RefCell<Environment>>,
                        k: Box<Continuation>)
                        -> Result<Trampoline, Error> {
    match exprs.shift() {
        Some((car, cdr)) => {
            Ok(Trampoline::Bounce(car,
                                  env.clone(),
                                  Continuation::EvaluateExpressions(cdr, env, k)))
        }
        None => {
            println!("Trying to evaluate an empty expression list");
            Err(Error::EvalError {
                desc: "empty list".to_string(),
                ast: AST::Nil,
            })
        }
    }
}

fn process(exprs: AST, env: Rc<RefCell<Environment>>) -> Result<AST, Error> {
    if exprs.len() == 0 {
        return Ok(AST::Number(0));
    }
    let mut b = try!(evaluate_expressions(exprs, env, Box::new(Continuation::Return)));
    loop {
        println!("Tramploline: {}", b);
        match b {
            Trampoline::Bounce(a, env, k) => b = Trampoline::Land(a),
            Trampoline::Run(a, k) => b = try!(k.run(a)),
            Trampoline::Land(a) => {
                return Ok(a);
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

    fn define(&mut self, key: String, value: AST) -> Result<(), Error> {
        if self.values.contains_key(&key) {
            println!("Duplicate define: {:?}", key);
            Ok(())
        } else {
            self.values.insert(key, value);
            Ok(())
        }
    }

    fn set(&mut self, key: String, value: AST) -> Result<(), Error> {
        if self.values.contains_key(&key) {
            self.values.insert(key, value);
            Ok(())
        } else {
            match self.parent {
                Some(ref parent) => parent.borrow_mut().set(key, value),
                None => {
                    println!("Can't set! an undefined variable: {:?}", key);
                    Ok(())
                }
            }
        }
    }

    fn get(&self, key: &String) -> Option<AST> {
        match self.values.get(key) {
            Some(val) => Some(val.clone()),
            None => {
                match self.parent {
                    Some(ref parent) => parent.borrow().get(key),
                    None => None,
                }
            }
        }
    }

    fn get_root(env_ref: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let env = env_ref.borrow();
        match env.parent {
            Some(ref parent) => Environment::get_root(parent.clone()),
            None => env_ref.clone(),
        }
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
    Bounce(AST, Rc<RefCell<Environment>>, Continuation),
    Run(AST, Continuation),
    Land(AST),
}

impl fmt::Display for Trampoline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Trampoline::Bounce(ref value, ref env, ref cc) => {
                let a = unsafe { env.as_unsafe_cell().get() };
                write!(f, "Bounce {} env {} cc {}", value, unsafe { &*a }, cc)
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
    EvaluateCond(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateDefine(String, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateLambda(String, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateFunc(AST, AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateLet(String, AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    ExecuteEval(Rc<RefCell<Environment>>, Box<Continuation>),
    ExecuteApply(AST, Box<Continuation>),
    Return,
}

impl Continuation {
    pub fn run(self, val: AST) -> Result<Trampoline, Error> {
        match self {
            Continuation::EvaluateExpressions(rest, env, k) => {
                if !rest.is_empty() {
                    evaluate_expressions(rest, env, k)
                } else {
                    Ok(Trampoline::Run(val, *k))
                }
            }
            Continuation::EvaluateCond(if_expr, else_expr, env, k) => {
                match val {
                    AST::Bool(false) => Ok(Trampoline::Bounce(else_expr, env, *k)),
                    _ => Ok(Trampoline::Bounce(if_expr, env, *k)),
                }
            }
            Continuation::EvaluateDefine(name, env, k) => {
                try!(env.borrow_mut().define(name, val));
                Ok(Trampoline::Run(list(AST::Nil), *k))
            }
            _ => Ok(Trampoline::Land(AST::Nil)),
        }
    }
}

impl fmt::Display for Continuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Continuation::EvaluateExpressions(ref value, ref env, ref cc) => {
                let a = unsafe { env.as_unsafe_cell().get() };
                write!(f, "EvaluateExpressions {}", unsafe { &*a })
            }
            Continuation::BeginFunc(ref value, ref env, ref cc) => write!(f, "BeginFunc {}", value),
            Continuation::EvaluateCond(ref value, ref value2, ref env, ref cc) => {
                write!(f, "EvaluateIf {} {}", value, value2)
            }
            Continuation::EvaluateFunc(ref value, ref list, ref list2, ref env, ref cc) => {
                write!(f, "EvaluateFunc {} list {}", value, list2)
            }   
            Continuation::ExecuteEval(ref env, ref cc) => {
                let a = unsafe { env.as_unsafe_cell().get() };
                write!(f, "ExecuteEval {}", unsafe { &*a })
            }       
            Continuation::EvaluateDefine(ref value, ref env, ref cc) => {
                write!(f, "EvaluateDefine {}", value)
            }  
            Continuation::Return => write!(f, "Return"),
            Continuation::ExecuteApply(ref value, ref cc) => write!(f, "ExecuteApply {}", value), 
            _ => write!(f, "Unknown {}", 1),
        }
    }
}
