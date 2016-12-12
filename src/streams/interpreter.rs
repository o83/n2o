
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

pub struct Interpreter<'ast> {
    pub root: Environment<'ast>,
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Tape<'ast> {
    arena: &'ast Arena<'ast>,
    cont: &'ast Cont<'ast>,
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

    pub fn run(&'ast mut self,
               ast: &'ast AST<'ast>,
               arena: &'ast Arena<'ast>)
               -> Result<&'ast AST<'ast>, Error<'ast>> {
        let mut a = 0;
        let mut b = try!(evaluate_expr(ast,
                                       Tape {
                                           env: &self.root,
                                           arena: arena,
                                           cont: arena.cont(Cont::Return),
                                       }));
        //  while a < 5 {
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

    fn handle_defer(&'ast mut self,
                    a: &'ast AST<'ast>,
                    tape: Tape<'ast>)
                    -> Result<&'ast Lazy<'ast>, Error<'ast>> {
        match a {
            &AST::Assign(name, body) => {
                Ok(tape.arena.lazy(Lazy::Defer(body,
                                               Tape {
                                                   arena: tape.arena,
                                                   cont: tape.arena.cont(Cont::Assign(name, tape)),
                                               })))
            }
            &AST::Cond(val, left, right) => {
                match val {
                    &AST::Number(x) => tape.run(val), 
                    x => {
                        Ok(tape.arena
                            .lazy(Lazy::Defer(x,
                                              Tape {
                                                  env: tape.env,
                                                  arena: tape.arena,
                                                  cont: tape.arena
                                                      .cont(Cont::Cond(left, right, tape)),
                                              })))
                    }
                }
            }
            &AST::List(x) => evaluate_expr(x, tape),
            &AST::Call(c, a) => {
                Ok(tape.arena.lazy(Lazy::Defer(a,
                                               Tape {
                                                   env: tape.env,
                                                   arena: tape.arena,
                                                   cont: tape.arena.cont(Cont::Call(c, tape)),
                                               })))
            }
            &AST::Verb(verb, left, right) => {
                match (left, right) {
                    (&AST::Number(_), _) => {
                        Ok(tape.arena
                            .lazy(Lazy::Defer(right,
                                              Tape {
                                                  env: tape.env,
                                                  arena: tape.arena,
                                                  cont: tape.arena
                                                      .cont(Cont::Verb(verb.clone(),
                                                                       left,
                                                                       0,
                                                                       tape)),
                                              })))
                    }
                    (_, &AST::Number(_)) => {
                        Ok(tape.arena.lazy(Lazy::Defer(left,
                                                       Tape {
                                                           env: tape.env,
                                                           arena: tape.arena,
                                                           cont: tape.arena
                                                               .cont(Cont::Verb(verb.clone(),
                                                                                right,
                                                                                1,
                                                                                tape)),
                                                       })))
                    }
                    (x, y) => {
                        Ok(tape.arena
                            .lazy(Lazy::Defer(x,
                                              Tape {
                                                  env: tape.env,
                                                  arena: tape.arena,
                                                  cont: tape.arena
                                                      .cont(Cont::Verb(verb.clone(), y, 0, tape)),
                                              })))
                    }
                }
            }
            &AST::NameInt(name) => {
                match lookup(name, tape.env) {
                    Ok(v) => tape.run(v),
                    Err(x) => Err(x),
                }
            }
            x => tape.run(x),
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
                        tape: Tape<'ast>)
                        -> Result<&'ast Lazy<'ast>, Error<'ast>> {
        match *fun {
            AST::Lambda(names, body) => {
                Tape {
                        env: tape.env,
                        arena: tape.arena,
                        cont: tape.arena.cont(Cont::Func(names, args, tape)),
                    }
                    .run(&body)
            }
            AST::NameInt(s) => {
                match tape.env.get(s) {
                    Some(v) => {
                        evaluate_fun(v,
                                     args,
                                     Tape {
                                         arena: tape.arena,
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

    pub fn evaluate_expr(&'ast mut self,
                         exprs: &'ast AST<'ast>,
                         tape: Tape<'ast>)
                         -> Result<&'ast Lazy<'ast>, Error<'ast>> {
        match *exprs {
            AST::Cons(car, cdr) => {
                Ok(tape.arena.lazy(Lazy::Defer(car,
                                               Tape {
                                                   env: tape.env,
                                                   arena: tape.arena,
                                                   cont: tape.arena
                                                       .cont(Cont::Expressions(cdr, tape)),
                                               })))
            }
            AST::Nil => {
                Err(Error::EvalError {
                    desc: "Empty list".to_string(),
                    ast: AST::Nil,
                })
            }
            x => Ok(tape.arena.lazy(Lazy::Defer(&x, tape))),
        }
    }
}


impl<'ast> Tape<'ast> {
    pub fn run(self, val: &'ast AST<'ast>) -> Result<&'ast Lazy<'ast>, Error<'ast>> {
        match self.cont {
            &Cont::Call(callee, tape) => {
                match val {
                    &AST::Dict(v) => evaluate_fun(callee, v, tape),
                    x => evaluate_fun(callee, x, tape),
                }
            }
            // &Cont::Func(names, args, tape) => {
            //     tape.env.new_child();
            //     // for (name, value) in *names.into_iter().zip(*args.into_iter()) {
            //     //    try!(local_env.borrow_mut().define(ast::extract_name(name), value));
            //     // }
            //     evaluate_expr(val,
            //                   Tape {
            //                       env: tape.env,
            //                       arena: tape.arena,
            //                       cont: tape.cont,
            //                   })
            //
            // }
            // &Cont::Cond(if_expr, else_expr, tape) => {
            //     match val {
            //         &AST::Number(0) => Ok(tape.arena.lazy(Lazy::Defer(else_expr, tape))),
            //         &AST::Number(_) => Ok(tape.arena.lazy(Lazy::Defer(if_expr, tape))),
            //         x => {
            //             Ok(tape.arena.lazy(Lazy::Defer(x,
            //                                            Tape {
            //                                                env: tape.env,
            //                                                arena: tape.arena,
            //                                                cont: tape.arena
            //                                                    .cont(Cont::Cond(if_expr,
            //                                                                     else_expr,
            //                                                                     tape)),
            //                                            })))
            //         }
            //     }
            // }
            // &Cont::Assign(name, tape) => {
            //     match name {
            //         &AST::NameInt(s) => {
            //             try!(tape.env.define(s, val.clone()));
            //             evaluate_expr(val, tape)
            //         }
            //         x => {
            //             Err(Error::EvalError {
            //                 desc: "Can assign only to var".to_string(),
            //                 ast: *x,
            //             })
            //         }
            //
            //     }
            // }
            // &Cont::Verb(verb, right, swap, tape) => {
            //     match (right, val) {
            //         (&AST::Number(_), &AST::Number(_)) => {
            //             match swap {
            //                 0 => tape.run(&verb::eval(verb, right, val).unwrap()),
            //                 _ => tape.run(&verb::eval(verb, val, right).unwrap()),
            //             }
            //         }
            //         (x, y) => {
            //             Ok(tape.arena.lazy(Lazy::Defer(&x,
            //                                            Tape {
            //                                                env: tape.env.clone(),
            //                                                arena: tape.arena,
            //                                                cont: tape.arena
            //                                                    .cont(Cont::Verb(verb, &y, 0, tape)),
            //                                            })))
            //         }
            //     }
            // }
            // &Cont::Expressions(rest, tape) => {
            //     if rest.is_cons() || !rest.is_empty() {
            //         evaluate_expr(rest, tape)
            //     } else {
            //         tape.run(val)
            //     }
            // }
            // x => Ok(self.arena.lazy(Lazy::Return(val))),
            _ => Err(Error::InternalError),
        }
    }
}
