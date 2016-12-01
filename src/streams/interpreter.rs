
use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;
use streams::verb::plus;

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
        process(program, self.root.clone())
    }
}

fn process(exprs: AST, env: Rc<RefCell<Environment>>) -> Result<AST, Error> {
    println!("Len: {:?}", exprs.clone().len());
    if exprs.clone().len() == 0 {
        return Ok(AST::Nil);
    }
    let mut b = try!(evaluate_expressions(exprs.clone(), env, Box::new(Continuation::Return)));
    loop {
        println!("Tramploline: {:?}", b);
        match b {
            Trampoline::Bounce(a, env, k) => {
                b = match a.clone() {
                    AST::Assign(box name, box body) => {
                        Trampoline::Bounce(body,
                                           env.clone(),
                                           Continuation::EvaluateAssign(name, env, Box::new(k)))
                    }
                    AST::Lambda(box callee, box args) => {
                        println!("Lambda!");
                        Trampoline::Run(AST::Lambda(callee.boxed(), args.boxed()), k)
                    }
                    AST::Call(box callee, box args) => {
                        let mut fun = try!(evaluate_expressions(callee.clone(),
                                                                env.clone(),
                                                                Box::new(Continuation::Return)));
                        match fun {
                            Trampoline::Land(AST::Lambda(box names, box body)) => {
                                println!("Names: {:?}", names);
                                println!("Body: {:?}", body);
                                println!("Callee: {:?}", callee);
                                println!("Args: {:?}", args);
                                Trampoline::Bounce(body.clone(),
                                                   env.clone(),
                                                   Continuation::EvaluateFunc(names,
                                                                              args,
                                                                              body,
                                                                              env,
                                                                              Box::new(k)))
                            }
                            x => try!(k.run(a)),
                            // {
                            // Err(Error::EvalError {
                            // desc: "Function expected".to_string(),
                            // ast: callee,
                            // })
                            // }
                        }
                    }
                    AST::Name(name) => {
                        let val = match env.borrow().get(&name) {
                            Some(v) => v,
                            None => {
                                return Err(Error::EvalError {
                                    desc: "Identifier not found".to_string(),
                                    ast: AST::Name(name),
                                })
                            }
                        };
                        try!(k.run(val))
                    }
                    _ => try!(k.run(a)),
                }
            }
            Trampoline::Run(x, k) => b = try!(k.run(x)),
            Trampoline::Land(a) => {
                return Ok(a);
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


impl Continuation {
    pub fn run(self, val: AST) -> Result<Trampoline, Error> {
        println!("Continuation::run {:?}", val);
        match self {
            Continuation::EvaluateExpressions(rest, env, k) => {
                if rest.is_cons() || !rest.is_empty() {
                    evaluate_expressions(rest, env, k)
                } else {
                    Ok(Trampoline::Run(val, *k))
                }
            }
            Continuation::EvaluateFunc(names, args, body, env, k) => {
                let local_env = Environment::new_child(env);
                for (name, value) in names.into_iter().zip(args.into_iter()) {
                    try!(local_env.borrow_mut().define(name.to_string(), value));
                }
                evaluate_expressions(body, Environment::new_child(local_env), k)
            }
            Continuation::EvaluateCond(if_expr, else_expr, env, k) => {
                match val {
                    AST::Bool(false) => Ok(Trampoline::Bounce(else_expr, env, *k)),
                    _ => Ok(Trampoline::Bounce(if_expr, env, *k)),
                }
            }
            Continuation::EvaluateAssign(name, env, k) => {
                match name {
                    AST::Name(ref s) => {
                        try!(env.borrow_mut().define(s.to_string(), val));
                        Ok(Trampoline::Run(AST::Nil, *k))
                    }
                    x => {
                        Err(Error::EvalError {
                            desc: "can assign only to name".to_string(),
                            ast: x,
                        })
                    }

                }
            }
            Continuation::Return => Ok(Trampoline::Land(val)),
        }
    }
}

#[derive(PartialEq)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, AST>,
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

#[derive(Clone, Debug)]
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
                write!(f, "Bounce {:?} env {:?} cc {:?}", value, unsafe { &*a }, cc)
            }
            Trampoline::Run(ref value, ref cc) => write!(f, "Run {:?}", value),
            Trampoline::Land(ref value) => write!(f, "Land {:?}", value),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Continuation {
    EvaluateExpressions(AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateCond(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateAssign(AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateFunc(AST, AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    Return,
}


impl fmt::Display for Continuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Continuation::EvaluateExpressions(ref value, ref env, ref cc) => {
                let a = unsafe { env.as_unsafe_cell().get() };
                write!(f, "EvaluateExpressions {}", unsafe { &*a })
            }
            Continuation::EvaluateCond(ref value, ref value2, ref env, ref cc) => {
                write!(f, "EvaluateIf {} {}", value, value2)
            }
            Continuation::EvaluateFunc(ref value, ref list, ref list2, ref env, ref cc) => {
                write!(f, "EvaluateFunc {} list {}", value, list2)
            }   
            Continuation::EvaluateAssign(ref value, ref env, ref cc) => {
                write!(f, "EvaluateDefine {}", value)
            }  
            Continuation::Return => write!(f, "Return"),
        }
    }
}

pub fn test(program: AST) -> Result<AST, Error> {
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
