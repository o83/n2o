
// O-CPS INTERPRETER by 5HT et all

use std::fmt;
use std::cell::UnsafeCell;
use std::iter;
use std::vec;
// use streams::lambda::{self, call, func};
// use streams::verb::{self, plus, minus, div, mul};
use streams::verb::{self, plus};
use streams::env::*;
use commands::ast::*;
use commands::ast;
use commands::command;

// Interpreter, Lazy and Cont
#[derive(Clone, Debug)]
pub enum Lazy<'a> {
    Defer(&'a AST<'a>, &'a Cont<'a>),
    Return(&'a AST<'a>),
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
pub enum Cont<'a> {
    Expressions(&'a AST<'a>, &'a Cont<'a>),
    Lambda(Code, &'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Assign(&'a AST<'a>, &'a Cont<'a>),
    Cond(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Func(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Call(&'a AST<'a>, &'a Cont<'a>),
    Verb(Verb, &'a AST<'a>, u8, &'a Cont<'a>),
    Adverb(Adverb, &'a AST<'a>, &'a Cont<'a>),
    Return,
}

pub struct Interpreter<'a> {
    arena: Arena<'a>,
    env: Environment<'a>, // frame: usize,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Result<Interpreter<'a>, Error> {
        Ok(Interpreter {
            arena: Arena::new(),
            env: try!(Environment::new_root()),
        })
    }

    pub fn parse(&'a self, s: &String) -> &'a AST<'a> {
        command::parse_Mex(&self.arena, s).unwrap()
    }

    pub fn run(&'a self, ast: &'a AST<'a>) -> Result<&'a AST<'a>, Error> {
        let mut a = 0;
        let mut b = try!(self.evaluate_expr(None, ast, self.arena.cont(Cont::Return)));
        loop {
            // debug!("[Trampoline:{}]:{:?}\n", a, b);
            match b {
                &Lazy::Defer(x, t) => {
                    a = a + 1;
                    b = try!(self.handle_defer(None, x, t))
                }
                &Lazy::Return(a) => return Ok(a),
            }
        }
        Err(Error::EvalError {
            desc: "Program is terminated abnormally".to_string(),
            ast: format!("{:?}", AST::Nil),
        })
    }

    pub fn gc(&self) -> Result<usize, Error> {
        self.env.clean();
        Ok(1)
    }

    fn handle_defer(&'a self, frame: Option<usize>, a: &'a AST<'a>, cont: &'a Cont<'a>) -> Result<&'a Lazy<'a>, Error> {
        match a {
            &AST::Assign(name, body) => {
                Ok(self.arena.lazy(Lazy::Defer(body, self.arena.cont(Cont::Assign(name, cont)))))
            }
            &AST::Cond(val, left, right) => {
                match val {
                    &AST::Number(x) => self.run_cont(frame, val, cont), 
                    x => {
                        Ok(self.arena
                            .lazy(Lazy::Defer(x, self.arena.cont(Cont::Cond(left, right, cont)))))
                    }
                }
            }
            &AST::List(x) => self.evaluate_expr(frame, x, cont),
            &AST::Call(c, a) => Ok(self.arena.lazy(Lazy::Defer(a, self.arena.cont(Cont::Call(c, cont))))),
            &AST::Verb(ref verb, left, right) => {
                match (left, right) {
                    (&AST::Number(_), _) => {
                        Ok(self.arena.lazy(Lazy::Defer(right,
                                                       self.arena.cont(Cont::Verb(verb.clone(), left, 0, cont)))))
                    }
                    (_, &AST::Number(_)) => {
                        Ok(self.arena.lazy(Lazy::Defer(left,
                                                       self.arena
                                                           .cont(Cont::Verb(verb.clone(), right, 1, cont)))))
                    }
                    (x, y) => {
                        Ok(self.arena.lazy(Lazy::Defer(x,
                                                       self.arena
                                                           .cont(Cont::Verb(verb.clone(), y, 0, cont)))))
                    }
                }
            }
            &AST::NameInt(name) => {
                match self.lookup(frame, name, &self.env) {
                    Ok(v) => self.run_cont(frame, v, cont),
                    Err(x) => Err(x),
                }
            }
            x => self.run_cont(frame, x, cont),
        }
    }

    fn lookup(&'a self, frame: Option<usize>, name: u16, env: &'a Environment<'a>) -> Result<&'a AST<'a>, Error> {
        match env.get(name, frame) {
            Some(v) => Ok(v),
            None => {
                Err(Error::EvalError {
                    desc: "Identifier not found".to_string(),
                    ast: format!("{:?}", AST::NameInt(name)),
                })
            }
        }
    }

    pub fn evaluate_fun(&'a self,
                        frame: Option<usize>,
                        fun: &'a AST<'a>,
                        args: &'a AST<'a>,
                        cont: &'a Cont<'a>)
                        -> Result<&'a Lazy<'a>, Error> {
        match fun {
            &AST::Lambda(names, body) => self.run_cont(frame, body, self.arena.cont(Cont::Func(names, args, cont))),
            &AST::NameInt(s) => {
                match self.env.get(s, None) {
                    Some(v) => self.evaluate_fun(frame, v, args, cont),
                    None => {
                        Err(Error::EvalError {
                            desc: "Unknown variable".to_string(),
                            ast: format!("{:?}", AST::NameInt(s)),
                        })
                    }
                }
            }
            x => {
                Err(Error::EvalError {
                    desc: "Call Error".to_string(),
                    ast: format!("{:?}", x),
                })
            }
        }
    }

    pub fn evaluate_expr(&'a self,
                         frame: Option<usize>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<&'a Lazy<'a>, Error> {
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(self.arena.lazy(Lazy::Defer(car,
                                               self.arena
                                                   .cont(Cont::Expressions(cdr, cont)))))
            }
            &AST::Nil => {
                Err(Error::EvalError {
                    desc: "Empty list".to_string(),
                    ast: format!("{:?}", AST::Nil),
                })
            }
            x => Ok(self.arena.lazy(Lazy::Defer(x, cont))),
        }
    }

    pub fn run_cont(&'a self,
                    frame: Option<usize>,
                    val: &'a AST<'a>,
                    cont: &'a Cont<'a>)
                    -> Result<&'a Lazy<'a>, Error> {
        match cont {
            &Cont::Call(callee, cont) => {
                match val {
                    &AST::Dict(v) => self.evaluate_fun(frame, callee, v, cont),
                    x => self.evaluate_fun(frame, callee, x, cont),
                }
            }
            &Cont::Func(names, args, cont) => {
                let f = self.env.new_child();
                names.into_iter().zip(args.into_iter()).map(|(k, v)| self.env.define(ast::extract_name(k), v));
                self.evaluate_expr(f, val, cont)
            }
            &Cont::Cond(if_expr, else_expr, cont) => {
                match val {
                    &AST::Number(0) => Ok(self.arena.lazy(Lazy::Defer(else_expr, cont))),
                    &AST::Number(_) => Ok(self.arena.lazy(Lazy::Defer(if_expr, cont))),
                    x => Ok(self.arena.lazy(Lazy::Defer(x, self.arena.cont(Cont::Cond(if_expr, else_expr, cont))))),
                }
            }
            &Cont::Assign(name, cont) => {
                match name {
                    &AST::NameInt(s) => {
                        try!(self.env.define(s, val));
                        self.evaluate_expr(frame, val, cont)
                    }
                    x => {
                        Err(Error::EvalError {
                            desc: "Can assign only to var".to_string(),
                            ast: format!("{:?}", x),
                        })
                    }

                }
            }
            &Cont::Verb(ref verb, right, swap, cont) => {
                match (right, val) {
                    (&AST::Number(_), &AST::Number(_)) => {
                        match swap {
                            0 => {
                                let a = verb::eval(verb.clone(), right, val).unwrap();
                                self.run_cont(frame, self.arena.ast(a), cont)
                            }
                            _ => {
                                let a = verb::eval(verb.clone(), val, right).unwrap();
                                self.run_cont(frame, self.arena.ast(a), cont)
                            }
                        }
                    }
                    (x, y) => {
                        Ok(self.arena
                            .lazy(Lazy::Defer(x,
                                              self.arena
                                                  .cont(Cont::Verb(verb.clone(), y, 0, cont)))))
                    }
                }
            }
            &Cont::Expressions(rest, cont) => {
                if rest.is_cons() || !rest.is_empty() {
                    self.evaluate_expr(frame, rest, cont)
                } else {
                    self.run_cont(frame, val, cont)
                }
            }
            x => Ok(self.arena.lazy(Lazy::Return(val))),
        }
    }
}
