
// O-CPS INTERPRETER by 5HT et all

use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;
use streams::verb::plus;
use streams::env::*;

// Interpreter, Trampoline and Continuation
//     -- are Embedded Contexts, Lazy Type and Combinators respectively

#[derive(Clone)]
pub struct Interpreter {
    root: Rc<RefCell<Environment>>,
}

#[derive(Clone, Debug)]
pub enum Trampoline {
    Defer(AST, Rc<RefCell<Environment>>, Continuation),
    Force(AST, Continuation),
    Return(AST),
}

// Plug Any Combinators here

#[derive(PartialEq, Clone, Debug)]
pub enum Continuation {
    EvaluateExpressions(AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateAssign(AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateCond(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateFunc(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateVerb(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    EvaluateAdverb(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    Return,
}

fn process(exprs: AST, env: Rc<RefCell<Environment>>) -> Result<AST, Error> {
    if exprs.clone().len() == 0 {
        return Ok(AST::Nil);
    }
    let mut b = try!(evaluate_expressions(exprs.clone(), env, Box::new(Continuation::Return)));
    loop {
        println!("Trampoline: {:?}", b);
        match b {
            Trampoline::Defer(a, env, k) => {
                b = match a.clone() {
                    AST::Assign(box name, box body) => {
                        Trampoline::Defer(body,
                                          env.clone(),
                                          Continuation::EvaluateAssign(name, env, Box::new(k)))
                    }
                    AST::Call(box callee, box args) => {
                        let mut fun = try!(process(callee, env.clone()));
                        match fun {
                            AST::Lambda(box names, box body) => {
                                Trampoline::Force(body,
                                                  Continuation::EvaluateFunc(names,
                                                                             args,
                                                                             env,
                                                                             Box::new(k)))
                            }
                            x => {
                                return Err(Error::EvalError {
                                    desc: "Call Error".to_string(),
                                    ast: a,
                                })
                            }
                        }
                    }
                    AST::Name(name) => {
                        match lookup(name, env) {
                            Ok(v) => try!(k.run(v)),
                            Err(x) => return Err(x),
                        }
                    }
                    _ => try!(k.run(a)),
                }
            }
            Trampoline::Force(x, k) => b = try!(k.run(x)),
            Trampoline::Return(a) => {
                return Ok(a);
            }
        }
    }
}

fn lookup(name: String, env: Rc<RefCell<Environment>>) -> Result<AST, Error> {
    return match env.borrow().get(&name) {
        Some(v) => Ok(v),
        None => {
            Err(Error::EvalError {
                desc: "Identifier not found".to_string(),
                ast: AST::Name(name),
            })
        }
    };
}

fn evaluate_expressions(exprs: AST,
                        env: Rc<RefCell<Environment>>,
                        k: Box<Continuation>)
                        -> Result<Trampoline, Error> {
    match exprs.shift() {
        Some((car, cdr)) => {
            Ok(Trampoline::Defer(car,
                                 env.clone(),
                                 Continuation::EvaluateExpressions(cdr, env, k)))
        }
        None => {
            Err(Error::EvalError {
                desc: "Empty list".to_string(),
                ast: AST::Nil,
            })
        }
    }
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

impl Continuation {
    pub fn run(self, val: AST) -> Result<Trampoline, Error> {
        println!("Continuation::run {:?}", val);
        match self {
            Continuation::EvaluateExpressions(rest, env, k) => {
                if rest.is_cons() || !rest.is_empty() {
                    evaluate_expressions(rest, env, k)
                } else {
                    Ok(Trampoline::Force(val, *k))
                }
            }
            Continuation::EvaluateFunc(names, args, env, k) => {
                let local_env = Environment::new_child(env);
                for (name, value) in names.into_iter().zip(args.into_iter()) {
                    try!(local_env.borrow_mut().define(name.to_string(), value));
                }
                evaluate_expressions(val, local_env, k)
            }
            Continuation::EvaluateCond(if_expr, else_expr, env, k) => {
                match val {
                    AST::Bool(false) => Ok(Trampoline::Defer(else_expr, env, *k)),
                    _ => Ok(Trampoline::Defer(if_expr, env, *k)),
                }
            }
            Continuation::EvaluateAssign(name, env, k) => {
                match name {
                    AST::Name(ref s) => {
                        try!(env.borrow_mut().define(s.to_string(), val));
                        Ok(Trampoline::Force(AST::Nil, *k))
                    }
                    x => {
                        Err(Error::EvalError {
                            desc: "Can assign only to var".to_string(),
                            ast: x,
                        })
                    }

                }
            }
            _ => Ok(Trampoline::Return(val)),
        }
    }
}

impl fmt::Display for Trampoline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Trampoline::Defer(ref value, ref env, ref cc) => {
                let a = unsafe { env.as_unsafe_cell().get() };
                write!(f, "Bounce {:?} env {:?} cc {:?}", value, unsafe { &*a }, cc)
            }
            Trampoline::Force(ref value, ref cc) => write!(f, "Run {:?}", value),
            Trampoline::Return(ref value) => write!(f, "Land {:?}", value),
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
            Continuation::EvaluateCond(ref value, ref value2, ref env, ref cc) => {
                write!(f, "EvaluateIf {} {}", value, value2)
            }
            Continuation::EvaluateFunc(ref value, ref list, ref env, ref cc) => {
                write!(f, "EvaluateFunc {} list {}", value, list)
            }
            Continuation::EvaluateAssign(ref value, ref env, ref cc) => {
                write!(f, "EvaluateDefine {}", value)
            }
            Continuation::EvaluateVerb(ref value, ref list, ref env, ref cc) => {
                write!(f, "EvaluateVerb {}", value)
            }
            Continuation::EvaluateAdverb(ref value, ref list, ref env, ref cc) => {
                write!(f, "EvaluateAdverb {}", value)
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
