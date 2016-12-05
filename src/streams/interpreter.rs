
// O-CPS INTERPRETER by 5HT et all

use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use streams::verb;
use streams::verb::*;
use streams::env::*;
use commands::ast::*;

// Interpreter, Trampoline and Continuation
// are Embedded Contexts, Lazy Type and Combinators respectively

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
    Expressions(AST, Rc<RefCell<Environment>>, Box<Continuation>),
    Assign(AST, Rc<RefCell<Environment>>, Box<Continuation>),
    Cond(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    Func(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    Call(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    Verb(Verb, AST, u8, Rc<RefCell<Environment>>, Box<Continuation>),
    Adverb(AST, AST, Rc<RefCell<Environment>>, Box<Continuation>),
    Return,
}

fn process(exprs: AST, env: Rc<RefCell<Environment>>) -> Result<AST, Error> {
    if exprs.len() == 0 {
        return Ok(AST::Nil);
    }
    let mut a = 0;
    let mut b = try!(evaluate_expressions(exprs, env.clone(), Box::new(Continuation::Return)));
    loop {
        debug!("[Trampoline:{}]:{:?}\n", a, b);
        match b {
            Trampoline::Defer(a, e, k) => b = try!(handle_defer(a, e, k)),
            Trampoline::Force(x, k) => {
                a = a + 1;
                b = try!(k.run(x))
            }
            Trampoline::Return(a) => return Ok(a),
        }
    }
}

fn handle_defer(a: AST,
                env: Rc<RefCell<Environment>>,
                k: Continuation)
                -> Result<Trampoline, Error> {
    match a {
        AST::Assign(box name, box body) => {
            Ok(Trampoline::Defer(body,
                                 env.clone(),
                                 Continuation::Assign(name, env, Box::new(k))))
        }
        AST::List(box x) => evaluate_expressions(x, env, box k),
        AST::Call(box callee, box args) => {
            Ok(Trampoline::Defer(args.clone(),
                                 env.clone(),
                                 Continuation::Call(callee, args, env, box k)))
        }
        AST::Verb(verb, box left, box right) => {
            match (left.clone(), right.clone()) {
                (AST::Number(x), _) => {
                    Ok(Trampoline::Defer(right,
                                         env.clone(),
                                         Continuation::Verb(verb,
                                                            AST::Number(x),
                                                            0,
                                                            env.clone(),
                                                            box k)))
                }
                (_, AST::Number(y)) => {
                    Ok(Trampoline::Defer(left,
                                         env.clone(),
                                         Continuation::Verb(verb,
                                                            AST::Number(y),
                                                            1,
                                                            env.clone(),
                                                            box k)))
                }
                (x, y) => {
                    Ok(Trampoline::Defer(x,
                                         env.clone(),
                                         Continuation::Verb(verb, y, 0, env, box k)))
                }
            }
        }
        AST::Cond(box val, box left, box right) => {
            match val {
                AST::Number(x) => {
                    Ok(Trampoline::Force(val, Continuation::Cond(left, right, env, box k)))
                }
                x => {
                    Ok(Trampoline::Defer(x,
                                         env.clone(),
                                         Continuation::Cond(left, right, env, box k)))
                }
            }
        }
        AST::Name(name) => {
            match lookup(name, env) {
                Ok(v) => k.run(v),
                Err(x) => Err(x),
            }
        }
        x => k.run(x),
    }
}

fn lookup(name: String, env: Rc<RefCell<Environment>>) -> Result<AST, Error> {
    match env.borrow().get(&name) {
        Some(v) => Ok(v),
        None => {
            Err(Error::EvalError {
                desc: "Identifier not found".to_string(),
                ast: AST::Name(name),
            })
        }
    }
}

fn evaluate_function(fun: AST,
                     env: Rc<RefCell<Environment>>,
                     args: AST,
                     k: Continuation)
                     -> Result<Trampoline, Error> {
    match fun {
        AST::Lambda(box names, box body) => {
            Ok(Trampoline::Force(body, Continuation::Func(names, args, env, box k)))
        }
        AST::Name(s) => {
            match env.borrow().find(&s) {
                Some((v, x)) => evaluate_function(v, x, args, k),
                None => {
                    Err(Error::EvalError {
                        desc: "Function Name in all Contexts".to_string(),
                        ast: AST::Name(s),
                    })
                }
            }
        }
        x => {
            Err(Error::EvalError {
                desc: "Call Error".to_string(),
                ast: x,
            })
        }
    }
}

fn evaluate_expressions(exprs: AST,
                        env: Rc<RefCell<Environment>>,
                        k: Box<Continuation>)
                        -> Result<Trampoline, Error> {
    match exprs.shift() {
        Some((car, cdr)) => {
            Ok(Trampoline::Defer(car, env.clone(), Continuation::Expressions(cdr, env, k)))
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
        match self {
            Continuation::Expressions(rest, env, k) => {
                if rest.is_cons() || !rest.is_empty() {
                    evaluate_expressions(rest, env, k)
                } else {
                    Ok(Trampoline::Defer(val, env, *k))
                }
            }
            Continuation::Call(callee, args, env, k) => {
                match args.clone() {
                    AST::Dict(box v) => evaluate_function(callee, env, v, *k),
                    _ => evaluate_function(callee, env, val, *k),
                }
            }
            Continuation::Func(names, args, env, k) => {
                let local_env = Environment::new_child(env);
                for (name, value) in names.into_iter().zip(args.into_iter()) {
                    try!(local_env.borrow_mut().define(name.to_string(), value));
                }
                evaluate_expressions(val, local_env, k)

            }
            Continuation::Cond(if_expr, else_expr, env, k) => {
                match val {
                    AST::Number(0) => Ok(Trampoline::Defer(else_expr, env, *k)),
                    AST::Number(_) => Ok(Trampoline::Defer(if_expr, env, *k)),
                    x => {
                        Ok(Trampoline::Defer(x,
                                             env.clone(),
                                             Continuation::Cond(if_expr, else_expr, env, k)))
                    }
                }
            }
            Continuation::Verb(verb, right, swap, env, k) => {
                match (right.clone(), val.clone()) {
                    (AST::Number(_), AST::Number(_)) => {
                        match swap {
                            0 => Ok(Trampoline::Force(verb::eval(verb, right, val).unwrap(), *k)),
                            _ => Ok(Trampoline::Force(verb::eval(verb, val, right).unwrap(), *k)),
                        }
                    }
                    (x, y) => {
                        Ok(Trampoline::Defer(x,
                                             env.clone(),
                                             Continuation::Verb(verb, y, 0, env, k)))
                    }
                }
            }
            Continuation::Assign(name, env, k) => {
                match name {
                    AST::Name(ref s) => {
                        try!(env.borrow_mut().define(s.to_string(), val.clone()));
                        evaluate_expressions(val, env.clone(), k)
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
