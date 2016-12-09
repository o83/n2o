
// O-CPS INTERPRETER by 5HT et all

use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
// use streams::lambda::{self, call, func};
// use streams::verb::{self, plus, minus, div, mul};
use streams::verb::{self, plus};
use streams::env::*;
use commands::ast::*;
use commands::ast;

// Interpreter, Lazy and Cont

#[derive(Clone)]
pub struct Interpreter<'ast> {
    pub root: Rc<RefCell<Environment<'ast>>>,
    pub names_size: u16,
    pub symbols_size: u16,
    pub sequences_size: u16,
    pub names: HashMap<String, u16>,
    pub symbols: HashMap<String, u16>,
    pub sequences: HashMap<String, u16>,
}

#[derive(Clone, Debug)]
pub enum Lazy<'ast> {
    Defer(&'ast AST<'ast>, Tape<'ast>),
    Return(&'ast AST<'ast>),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Code {
    Assign,
    Cond,
    Func,
    Call,
}

// Plug Any Combinators here

#[derive(PartialEq, Clone, Debug)]
pub enum Cont<'ast> {
    Expressions(&'ast AST<'ast>, Tape<'ast>),
    Lambda(Code, &'ast AST<'ast>, &'ast AST<'ast>, Tape<'ast>),
    Assign(&'ast AST<'ast>, Tape<'ast>),
    Cond(&'ast AST<'ast>, &'ast AST<'ast>, Tape<'ast>),
    Func(&'ast AST<'ast>, &'ast AST<'ast>, Tape<'ast>),
    Call(&'ast AST<'ast>, Tape<'ast>),
    Verb(Verb, &'ast AST<'ast>, u8, Tape<'ast>),
    Adverb(Adverb, &'ast AST<'ast>, Tape<'ast>),
    Return,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Tape<'ast> {
    env: Rc<RefCell<Environment<'ast>>>,
    cont: Box<Cont<'ast>>,
}

fn handle_defer<'ast>(a: &'ast AST<'ast>, mut tape: Tape<'ast>) -> Result<Lazy<'ast>, Error<'ast>> {
    match *a {
        AST::Assign(name, body) => {
            Ok(Lazy::Defer(body,
                           Tape {
                               env: tape.clone().env.clone(),
                               cont: Cont::Assign(name, tape),
                           }))
        }
        AST::Cond(val, left, right) => {
            match *val {
                AST::Number(x) => tape.run(val), 
                x => {
                    Ok(Lazy::Defer(&x,
                                   Tape {
                                       env: tape.env.clone(),
                                       cont: Cont::Cond(left, right, tape),
                                   }))
                }
            }
        }
        AST::List(x) => evaluate_expr(x, tape),
        AST::Call(c, a) => {
            Ok(Lazy::Defer(a,
                           Tape {
                               env: tape.env.clone(),
                               cont: Cont::Call(c, tape),
                           }))
        }
        AST::Verb(verb, left, right) => {
            match (*left, *right) {
                (AST::Number(_), _) => {
                    Ok(Lazy::Defer(right,
                                   Tape {
                                       env: tape.env.clone(),
                                       cont: Cont::Verb(verb, left, 0, tape),
                                   }))
                }
                (_, AST::Number(_)) => {
                    Ok(Lazy::Defer(left,
                                   Tape {
                                       env: tape.env.clone(),
                                       cont: Cont::Verb(verb, right, 1, tape),
                                   }))
                }
                (x, y) => {
                    Ok(Lazy::Defer(x,
                                   Tape {
                                       env: tape.env.clone(),
                                       cont: Cont::Verb(verb, y, 0, tape),
                                   }))
                }
            }
        }
        AST::NameInt(name) => {
            match lookup(name, tape.env.clone()) {
                Ok(v) => tape.run(v),
                Err(x) => Err(x),
            }
        }
        x => tape.run(x),
    }
}

fn lookup<'ast>(name: u16, env: Rc<RefCell<Environment<'ast>>>) -> Result<AST<'ast>, Error<'ast>> {
    match env.borrow().get(&name) {
        Some(v) => Ok(v),
        None => {
            Err(Error::EvalError {
                desc: "Identifier not found".to_string(),
                ast: AST::NameInt(name),
            })
        }
    }
}

pub fn evaluate_fun<'ast>(fun: &'ast AST<'ast>,
                          args: &'ast AST<'ast>,
                          tape: Tape<'ast>)
                          -> Result<Lazy<'ast>, Error<'ast>> {
    match *fun {
        AST::Lambda(names, body) => {
            Tape {
                    env: tape.env.clone(),
                    cont: Cont::Func(names, args, tape),
                }
                .run(body)
            // Ok(Lazy::Force(body, ))
            // Ok(Lazy::Force(body, Cont::Func(names, args, env, k)))
        }
        AST::NameInt(s) => {
            match tape.env.borrow().find(&s) {
                Some((v, x)) => {
                    evaluate_fun(v,
                                 args,
                                 Tape {
                                     env: x,
                                     cont: tape.cont,
                                 })
                }
                None => {
                    Err(Error::EvalError {
                        desc: "Unknown variable".to_string(),
                        ast: AST::NameInt(s),
                    })
                }
            }
        }
        x => {
            println!("{:?}", x);
            Err(Error::EvalError {
                desc: "Call Error".to_string(),
                ast: x,
            })
        }
    }
}

pub fn evaluate_expr<'ast>(exprs: &'ast AST<'ast>,
                           tape: Tape<'ast>)
                           -> Result<Lazy<'ast>, Error<'ast>> {
    match *exprs {
        AST::Cons(car, cdr) => {
            Ok(Lazy::Defer(car,
                           Tape {
                               env: tape.env.clone(),
                               cont: Cont::Expressions(cdr, tape),
                           }))
        }
        AST::Nil => {
            Err(Error::EvalError {
                desc: "Empty list".to_string(),
                ast: AST::Nil,
            })
        }
        x => Ok(Lazy::Defer(x, tape)),
    }
}

impl<'ast> Interpreter<'ast> {
    pub fn new() -> Result<Interpreter<'ast>, Error<'ast>> {
        let env = try!(Environment::new_root());
        Ok(Interpreter {
            root: env,
            names_size: 0,
            symbols_size: 0,
            sequences_size: 0,
            names: HashMap::new(),
            symbols: HashMap::new(),
            sequences: HashMap::new(),
        })
    }

    pub fn run(&mut self, ast: &mut AST<'ast>) -> Result<AST<'ast>, Error<'ast>> {
        let mut a = 0;
        let mut b = try!(evaluate_expr(ast.clone(),
                                       Tape {
                                           env: self.root.clone(),
                                           cont: Cont::Return,
                                       }));
        //  while a < 5 {
        loop {
            debug!("[Trampoline:{}]:{:?}\n", a, b);
            match b {
                Lazy::Defer(a, t) => b = try!(handle_defer(a, t)),
                Lazy::Return(a) => return Ok(a),
            }
        }
        Err(Error::EvalError {
            desc: "Program is terminated abnormally".to_string(),
            ast: AST::Nil,
        })
    }
}


impl<'ast> Tape<'ast> {
    pub fn run(mut self, val: AST<'ast>) -> Result<Lazy<'ast>, Error<'ast>> {
        let x = self.cont;
        match x {
            // Cont::Lambda(code, left, right, env, k) =>
            // {
            // lambda::eval(code, left, right, env, val, k)
            // }
            //
            Cont::Call(callee, tape) => {
                match val {
                    AST::Dict(v) => evaluate_fun(callee, v, tape),
                    _ => evaluate_fun(callee, val, tape),
                }
            }
            Cont::Func(names, args, tape) => {
                let local_env = Environment::new_child(tape.env);
                for (name, value) in names.into_iter().zip(args.into_iter()) {
                    try!(local_env.borrow_mut().define(ast::extract_name(name), value));
                }
                evaluate_expr(val,
                              Tape {
                                  env: local_env,
                                  cont: tape.cont,
                              })

            }
            Cont::Cond(if_expr, else_expr, tape) => {
                match val {
                    AST::Number(0) => Ok(Lazy::Defer(else_expr, tape)),
                    AST::Number(_) => Ok(Lazy::Defer(if_expr, tape)),
                    x => {
                        Ok(Lazy::Defer(x,
                                       Tape {
                                           env: tape.env.clone(),
                                           cont: Cont::Cond(if_expr, else_expr, tape),
                                       }))
                    }
                }
            }
            Cont::Assign(name, tape) => {
                match name {
                    AST::NameInt(s) => {
                        try!(tape.env.borrow_mut().define(s, val.clone()));
                        evaluate_expr(val, tape)
                    }
                    x => {
                        Err(Error::EvalError {
                            desc: "Can assign only to var".to_string(),
                            ast: x,
                        })
                    }

                }
            }
            Cont::Verb(verb, right, swap, mut tape) => {
                match (right.clone(), val.clone()) {
                    (AST::Number(_), AST::Number(_)) => {
                        match swap {
                            0 => tape.run(verb::eval(verb, right, val).unwrap()), // Ok(Lazy::Force(verb::eval(verb, right, val).unwrap(), k)),
                            _ => tape.run(verb::eval(verb, val, right).unwrap()), // Ok(Lazy::Force(verb::eval(verb, val, right).unwrap(), k)),
                        }
                    }
                    (x, y) => {
                        Ok(Lazy::Defer(x,
                                       Tape {
                                           env: tape.env.clone(),
                                           cont: Cont::Verb(verb, y, 0, tape),
                                       }))
                    }
                }
            }

            Cont::Expressions(rest, mut tape) => {
                if rest.is_cons() || !rest.is_empty() {
                    evaluate_expr(rest, tape)
                } else {
                    tape.run(val)
                    // Ok(Lazy::Force(val, *k))
                }
            }
            _ => Ok(Lazy::Return(val)),
        }
    }
}
