
// O-CPS INTERPRETER by 5HT et all

use std::fmt;
use std::cell::UnsafeCell;
use std::iter;
use std::vec;
use streams::verb::{self, plus};
use streams::env::*;
use commands::ast::*;
use commands::ast;
use commands::command;
use streams::otree::Node;

#[derive(Clone, Debug)]
pub enum Lazy<'a> {
    Defer(&'a Node<'a>, &'a AST<'a>, &'a Cont<'a>),
    Return(&'a AST<'a>),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Code {
    Assign,
    Cond,
    Func,
    Call,
}

#[derive(Clone, Debug)]
pub enum Cont<'a> {
    Expressions(&'a AST<'a>, &'a Cont<'a>),
    Lambda(Code, &'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Assign(&'a AST<'a>, &'a Cont<'a>),
    Cond(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Func(&'a Node<'a>, &'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    List(&'a AST<'a>, &'a Cont<'a>),
    Dict(&'a AST<'a>, &'a Cont<'a>),
    Call(&'a AST<'a>, &'a Cont<'a>),
    Verb(Verb, &'a AST<'a>, u8, &'a Cont<'a>),
    Adverb(Adverb, &'a AST<'a>, &'a Cont<'a>),
    Return,
}

pub struct Interpreter<'a> {
    arena: Arena<'a>,
    env: Environment<'a>,
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
        let mut b = try!(self.evaluate_expr(self.env.last(), ast, self.arena.cont(Cont::Return)));
        loop {
            // debug!("[Trampoline:{}]:{:?}\n", a, b);
            match b {
                &Lazy::Defer(f, x, t) => {
                    // a = a + 1;
                    b = try!(self.handle_defer(f, x, t))
                }
                &Lazy::Return(a) => return Ok(a),
            }
        }
        Err(Error::EvalError {
            desc: "Program is terminated abnormally".to_string(),
            ast: format!("{:?}", AST::Nil),
        })
    }

    pub fn gc(&self) -> usize {
        self.env.clean()
    }

    fn handle_defer(&'a self, node: &'a Node<'a>, a: &'a AST<'a>, cont: &'a Cont<'a>) -> Result<&'a Lazy<'a>, Error> {
        match a {
            &AST::Assign(name, body) => {
                Ok(self.arena.lazy(Lazy::Defer(node, body, self.arena.cont(Cont::Assign(name, cont)))))
            }
            &AST::Cond(val, left, right) => {
                match val {
                    &AST::Number(x) => self.run_cont(node, val, cont), 
                    x => {
                        Ok(self.arena
                            .lazy(Lazy::Defer(node, x, self.arena.cont(Cont::Cond(left, right, cont)))))
                    }
                }
            }
            &AST::List(x) => self.evaluate_expr(node, x, cont),
            // &AST::Cons(x, y) => self.run_cont(node, a, cont),
            &AST::Dict(x) => {
                println!("Defer Dict: {:?}", x);
                self.evaluate_dict(node, x, cont)
            }
            &AST::Call(c, a) => Ok(self.arena.lazy(Lazy::Defer(node, a, self.arena.cont(Cont::Call(c, cont))))),
            &AST::Verb(ref verb, left, right) => {
                match (left, right) {
                    (&AST::Number(_), _) => {
                        Ok(self.arena.lazy(Lazy::Defer(node,
                                                       right,
                                                       self.arena.cont(Cont::Verb(verb.clone(), left, 0, cont)))))
                    }
                    (_, &AST::Number(_)) => {
                        Ok(self.arena.lazy(Lazy::Defer(node,
                                                       left,
                                                       self.arena
                                                           .cont(Cont::Verb(verb.clone(), right, 1, cont)))))
                    }
                    (x, y) => {
                        Ok(self.arena.lazy(Lazy::Defer(node,
                                                       x,
                                                       self.arena
                                                           .cont(Cont::Verb(verb.clone(), y, 0, cont)))))
                    }
                }
            }
            &AST::NameInt(name) => {
                let l = self.lookup(node, name, &self.env);
                match l {
                    Ok((v, f)) => self.run_cont(f, v, cont),
                    Err(x) => Err(x),
                }
            }
            x => self.run_cont(node, x, cont),
        }
    }

    fn lookup(&'a self,
              node: &'a Node<'a>,
              name: u16,
              env: &'a Environment<'a>)
              -> Result<(&'a AST<'a>, &'a Node<'a>), Error> {
        match env.get(name, node) {
            Some((v, f)) => Ok((v, f)),
            None => {
                Err(Error::EvalError {
                    desc: "Identifier not found".to_string(),
                    ast: format!("{:?}", AST::NameInt(name)),
                })
            }
        }
    }

    pub fn evaluate_fun(&'a self,
                        node: &'a Node<'a>,
                        fun: &'a AST<'a>,
                        args: &'a AST<'a>,
                        cont: &'a Cont<'a>)
                        -> Result<&'a Lazy<'a>, Error> {
        println!("Eval Fun: {:?}", fun);
        match fun {
            &AST::Lambda(names, body) => {
                let mut rev = ast::rev_dict(args, &self.arena);
                println!("Args Fun: {:?} orig: {:?}, names: {:?}", rev, args, names);
                self.run_cont(node,
                              body,
                              self.arena.cont(Cont::Func(node, names, rev, cont)))
            }
            &AST::NameInt(s) => {
                let v = self.env.get(s, node);
                match v {
                    Some((v, f)) => self.evaluate_fun(f, v, args, cont), 
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
                         node: &'a Node<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<&'a Lazy<'a>, Error> {
        // println!("Eval Expr: {:?}", cont);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(self.arena.lazy(Lazy::Defer(node,
                                               car,
                                               self.arena
                                                   .cont(Cont::Expressions(cdr, cont)))))
            }
            &AST::Nil => {
                Err(Error::EvalError {
                    desc: "Empty list".to_string(),
                    ast: format!("{:?}", AST::Nil),
                })
            }
            x => Ok(self.arena.lazy(Lazy::Defer(node, x, cont))),
        }
    }

    pub fn evaluate_list(&'a self,
                         node: &'a Node<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<&'a Lazy<'a>, Error> {
        // println!("Eval List: {:?}", exprs);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(self.arena.lazy(Lazy::Defer(node,
                                               car,
                                               self.arena
                                                   .cont(Cont::List(cdr, cont)))))
            }
            &AST::Nil => {
                Err(Error::EvalError {
                    desc: "Empty list".to_string(),
                    ast: format!("{:?}", AST::Nil),
                })
            }
            x => Ok(self.arena.lazy(Lazy::Defer(node, x, cont))),
        }
    }

    pub fn evaluate_dict(&'a self,
                         node: &'a Node<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<&'a Lazy<'a>, Error> {
        // println!("Eval Dict: {:?}", exprs);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(self.arena.lazy(Lazy::Defer(node,
                                               car,
                                               self.arena
                                                   .cont(Cont::Dict(cdr, cont)))))
            }
            &AST::Nil => {
                Err(Error::EvalError {
                    desc: "Empty list".to_string(),
                    ast: format!("{:?}", AST::Nil),
                })
            }
            x => Ok(self.arena.lazy(Lazy::Defer(node, x, cont))),
        }
    }

    pub fn run_cont(&'a self, node: &'a Node<'a>, val: &'a AST<'a>, cont: &'a Cont<'a>) -> Result<&'a Lazy<'a>, Error> {
        match cont {
            &Cont::Call(callee, cont) => {
                match val {
                    &AST::Dict(v) => self.evaluate_fun(node, callee, v, cont),
                    x => self.evaluate_fun(node, callee, x, cont),
                }
            }
            &Cont::Func(fnode, names, args, cont) => {
                //  println!("Cont Func Args: {:?}", args);
                let f = self.env.new_child(fnode);
                for (k, v) in names.into_iter().zip(args.into_iter()) {
                    self.env.define(ast::extract_name(k), v);
                }
                self.evaluate_expr(f, val, cont)
            }
            &Cont::Dict(rest, cont) => {
                println!("Cont Dict: {:?} Val {:?}", rest, val);
                match rest {
                    &AST::Cons(head, tail) => {
                        match head {
                            &AST::Number(_) => {
                                self.run_cont(node,
                                              self.arena.ast(AST::Cons(val, head)),
                                              self.arena.cont(Cont::Dict(tail, cont)))
                            }
                            x => {
                                self.evaluate_dict(node,
                                                   head,
                                                   self.arena
                                                       .cont(Cont::Dict(self.arena.ast(AST::Cons(val, tail)), cont)))
                            }
                        }
                    }
                    &AST::Number(s) => self.run_cont(node, self.arena.ast(AST::Cons(rest, val)), cont),
                    &AST::Nil => self.run_cont(node, val, cont),
                    &AST::NameInt(name) => {
                        let l = self.env.get(name, node);
                        match l {
                            Some((v, f)) => self.run_cont(f, self.arena.ast(AST::Cons(v, val)), cont),
                            None => {
                                Err(Error::EvalError {
                                    desc: "".to_string(),
                                    ast: "".to_string(),
                                })
                            }
                        }
                    }

                    x => {
                        // println!("Lame Dict: {:?}", val);
                        Ok(self.arena.lazy(Lazy::Defer(node, x, self.arena.cont(Cont::Dict(val, cont)))))
                    }

                }

            }
            &Cont::Cond(if_expr, else_expr, cont) => {
                match val {
                    &AST::Number(0) => Ok(self.arena.lazy(Lazy::Defer(node, else_expr, cont))),
                    &AST::Number(_) => Ok(self.arena.lazy(Lazy::Defer(node, if_expr, cont))),
                    x => {
                        Ok(self.arena.lazy(Lazy::Defer(node,
                                                       x,
                                                       self.arena.cont(Cont::Cond(if_expr, else_expr, cont)))))
                    }
                }
            }
            &Cont::Assign(name, cont) => {
                match name {
                    &AST::NameInt(s) => {
                        try!(self.env.define(s, val));
                        self.evaluate_expr(node, val, cont)
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
                // println!("Cont Verb: {:?}", val);
                match (right, val) {
                    (&AST::Number(_), &AST::Number(_)) => {
                        match swap {
                            0 => {
                                let a = verb::eval(verb.clone(), right, val).unwrap();
                                self.run_cont(node, self.arena.ast(a), cont)
                            }
                            _ => {
                                let a = verb::eval(verb.clone(), val, right).unwrap();
                                self.run_cont(node, self.arena.ast(a), cont)
                            }
                        }
                    }
                    (x, y) => {
                        Ok(self.arena
                            .lazy(Lazy::Defer(node,
                                              x,
                                              self.arena
                                                  .cont(Cont::Verb(verb.clone(), y, 0, cont)))))
                    }
                }
            }
            &Cont::Expressions(rest, cont) => {
                if rest.is_cons() || !rest.is_empty() {
                    self.evaluate_expr(node, rest, cont)
                } else {
                    self.run_cont(node, val, cont)
                }
            }
            x => {
                // println!("Return: {:?} ", val);
                Ok(self.arena.lazy(Lazy::Return(val)))
            }
        }
    }
}
