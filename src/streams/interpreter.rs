
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
#[derive(Clone, Debug)]
pub enum Lazy<'ast> {
    Defer(&'ast AST<'ast>, &'ast Cont<'ast>),
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

#[derive(Clone, Debug)]
pub enum Cont<'ast> {
    Expressions(&'ast AST<'ast>, &'ast Cont<'ast>),
    Lambda(Code, &'ast AST<'ast>, &'ast AST<'ast>, &'ast Cont<'ast>),
    Assign(&'ast AST<'ast>, &'ast Cont<'ast>),
    Cond(&'ast AST<'ast>, &'ast AST<'ast>, &'ast Cont<'ast>),
    Func(&'ast AST<'ast>, &'ast AST<'ast>, &'ast Cont<'ast>),
    Call(&'ast AST<'ast>, &'ast Cont<'ast>),
    Verb(Verb, &'ast AST<'ast>, u8, &'ast Cont<'ast>),
    Adverb(Adverb, &'ast AST<'ast>, &'ast Cont<'ast>),
    Return,
}

pub struct Interpreter<'ast> {
    arena: &'ast Arena<'ast>,
    env: Environment,
}

impl<'ast> Interpreter<'ast> {
    pub fn new(arena: &'ast Arena<'ast) -> Result<Interpreter<'ast>, Error<'ast>> {
        let env = try!(Environment::new_root());
        Ok(Interpreter {
            arena: arena,
            env: env,
        })
    }

    pub fn run(&'ast mut self, ast: &'ast AST<'ast>) -> Result<&'ast AST<'ast>, Error<'ast>> {
        let mut a = 0;
        let mut b = try!(evaluate_expr(ast, self.arena.cont(Cont::Return)));
        loop {
            debug!("[Trampoline:{}]:{:?}\n", a, b);
            match b {
                &Lazy::Defer(a, t) => b = try!(handle_defer(a, t)),
                &Lazy::Return(a) => return Ok(a),
            }
        }
        Err(Error::EvalError {
            desc: "Program is terminated abnormally".to_string(),
            ast: AST::Nil,
        })
    }

    fn handle_defer(&'ast mut self, a: &'ast AST<'ast>, cont: &'ast Cont<'ast>) -> Result<&'ast Lazy<'ast>, Error<'ast>> {
        match a {
            &AST::Assign(name, body) => {
                Ok(self.arena.lazy(Lazy::Defer(body, self.arena.cont(Cont::Assign(name, cont)))))
            }
            &AST::Cond(val, left, right) => {
                match val {
                    &AST::Number(x) => cont.run(val), 
                    x => {
                        Ok(self.arena.lazy(Lazy::Defer(x, self.arena.cont(Cont::Cond(left, right, tape)))))
                    }
                }
            }
            &AST::List(x) => evaluate_expr(x, tape),
            &AST::Call(c, a) => {
                Ok(tape.arena.lazy(Lazy::Defer(a, self.arena.cont(Cont::Call(c, cont)))))
            }
            &AST::Verb(verb, left, right) => {
                match (left, right) {
                    (&AST::Number(_), _) => {
                        Ok(self.arena.lazy(Lazy::Defer(right, self.arena.cont(Cont::Verb(verb.clone(), left, 0, cont)))))
                    }
                    (_, &AST::Number(_)) => {
                        Ok(self.arena.lazy(Lazy::Defer(left, self.arena
                                                               .cont(Cont::Verb(verb.clone(),
                                                                                right,
                                                                                1,
                                                                                cont)),
                                                       )))
                    }
                    (x, y) => {
                        Ok(self.arena.lazy(Lazy::Defer(x, self.arena
                                                      .cont(Cont::Verb(verb.clone(), y, 0, cont)),
                                              )))
                    }
                }
            }
            &AST::NameInt(name) => {
                match lookup(name, self.env) {
                    Ok(v) => cont.run(v),
                    Err(x) => Err(x),
                }
            }
            x => cont.run(x),
        }
    }

    fn lookup(&'ast mut self,
              name: u16,
              env: &'ast Environment<'ast>)
              -> Result<&'ast AST<'ast>, Error<'ast>> {
        match env.get(name) {
            Some(v) => Ok(v),
            None => {
                Err(Error::EvalError {
                    desc: "Identifier not found".to_string(),
                    ast: AST::NameInt(name),
                })
            }
        }
    }

    pub fn evaluate_fun(&'ast mut self,
                        fun: &'ast AST<'ast>,
                        args: &'ast AST<'ast>,
                        cont: &'ast Cont<'ast>)
                        -> Result<&'ast Lazy<'ast>, Error<'ast>> {
        match *fun {
            AST::Lambda(names, body) => {
                        self.arena.cont(Cont::Func(names, args, cont)).run(&body)
            }
            AST::NameInt(s) => {
                match self.env.get(s) {
                    Some(v) => {
                        evaluate_fun(v, cont)
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

    pub fn evaluate_expr(&'ast mut self,
                         exprs: &'ast AST<'ast>,
                         cont: &'ast Cont<'ast>)
                         -> Result<&'ast Lazy<'ast>, Error<'ast>> {
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(self.arena.lazy(Lazy::Defer(car, self.arena
                                                       .cont(Cont::Expressions(cdr, cont)),
                                               )))
            }
            &AST::Nil => {
                Err(Error::EvalError {
                    desc: "Empty list".to_string(),
                    ast: AST::Nil,
                })
            }
            x => Ok(self.arena.lazy(Lazy::Defer(x, cont))),
        }
    }
}


impl<'ast> Cont<'ast> {
    pub fn run(&self, val: &'ast AST<'ast>) -> Result<&'ast Lazy<'ast>, Error<'ast>> {
        match self {
            &Cont::Call(callee, tape) => {
                match val {
                    &AST::Dict(v) => evaluate_fun(callee, v, tape),
                    x => evaluate_fun(callee, x, tape),
                }
            }
            &Cont::Func(names, args, tape) => {
                tape.env.new_child();
                // for (name, value) in *names.into_iter().zip(*args.into_iter()) {
                //    try!(local_env.borrow_mut().define(ast::extract_name(name), value));
                // }
                evaluate_expr(val,
                              Tape {
                                  env: tape.env,
                                  arena: tape.arena,
                                  cont: tape.cont,
                              })
            
            }
            &Cont::Cond(if_expr, else_expr, tape) => {
                match val {
                    &AST::Number(0) => Ok(tape.arena.lazy(Lazy::Defer(else_expr, tape))),
                    &AST::Number(_) => Ok(tape.arena.lazy(Lazy::Defer(if_expr, tape))),
                    x => {
                        Ok(tape.arena.lazy(Lazy::Defer(x,
                                                       Tape {
                                                           env: tape.env,
                                                           arena: tape.arena,
                                                           cont: tape.arena
                                                               .cont(Cont::Cond(if_expr,
                                                                                else_expr,
                                                                                tape)),
                                                       })))
                    }
                }
            }
            &Cont::Assign(name, tape) => {
                match name {
                    &AST::NameInt(s) => {
                        try!(tape.env.define(s, val.clone()));
                        evaluate_expr(val, tape)
                    }
                    x => {
                        Err(Error::EvalError {
                            desc: "Can assign only to var".to_string(),
                            ast: *x,
                        })
                    }
            
                }
            }
            &Cont::Verb(verb, right, swap, tape) => {
                match (right, val) {
                    (&AST::Number(_), &AST::Number(_)) => {
                        match swap {
                            0 => tape.run(&verb::eval(verb, right, val).unwrap()),
                            _ => tape.run(&verb::eval(verb, val, right).unwrap()),
                        }
                    }
                    (x, y) => {
                        Ok(tape.arena.lazy(Lazy::Defer(&x,
                                                       Tape {
                                                           env: tape.env.clone(),
                                                           arena: tape.arena,
                                                           cont: tape.arena
                                                               .cont(Cont::Verb(verb, &y, 0, tape)),
                                                       })))
                    }
                }
            }
            &Cont::Expressions(rest, tape) => {
                if rest.is_cons() || !rest.is_empty() {
                    evaluate_expr(rest, tape)
                } else {
                    tape.run(val)
                }
            }
            x => Ok(self.arena.lazy(Lazy::Return(val))),
            _ => Err(Error::InternalError),
        }
    }
}
