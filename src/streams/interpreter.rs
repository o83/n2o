
// O-CPS INTERPRETER by 5HT et all

use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
// use streams::lambda::{self, call, func};
use streams::verb::{self, plus, minus, div, mul};
use streams::env::*;
use commands::ast::*;
use commands::ast;

// Interpreter, Lazy and Cont

#[derive(Clone)]
pub struct Interpreter {
    pub root: Rc<RefCell<Environment>>,
    pub names_size: u16,
    pub symbols_size: u16,
    pub sequences_size: u16,
    pub names: HashMap<String, u16>,
    pub symbols: HashMap<String, u16>,
    pub sequences: HashMap<String, u16>,
}

#[derive(Clone, Debug)]
pub enum Lazy {
    Defer(AST, Tape),
    Return(AST),
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
pub enum Cont {
    Expressions(AST, Tape),
    Lambda(Code, AST, AST, Tape),
    Assign(AST, Tape),
    Cond(AST, AST, Tape),
    Func(AST, AST, Tape),
    Call(AST, Tape),
    Verb(Verb, AST, u8, Tape),
    Adverb(Adverb, AST, Tape),
    Return,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Tape {
    env: Rc<RefCell<Environment>>,
    cont: Box<Cont>,
}

fn handle_defer(a: AST, mut tape: Tape) -> Result<Lazy, Error> {
    match a {
        AST::Assign(box name, box body) => {
            Ok(Lazy::Defer(body,
                           Tape {
                               env: tape.clone().env.clone(),
                               cont: box Cont::Assign(name, tape),
                           }))
        }
        AST::Cond(box val, box left, box right) => {
            match val {
                AST::Number(x) => tape.run(val), 
                x => {
                    Ok(Lazy::Defer(x,
                                   Tape {
                                       env: tape.env.clone(),
                                       cont: box Cont::Cond(left, right, tape),
                                   }))
                }
            }
        }
        AST::List(box x) => evaluate_expr(x, tape),
        AST::Call(box c, box a) => {
            Ok(Lazy::Defer(a.clone(),
                           Tape {
                               env: tape.env.clone(),
                               cont: box Cont::Call(c, tape),
                           }))
        }
        AST::Verb(verb, box left, box right) => {
            match (left.clone(), right.clone()) {
                (AST::Number(_), _) => {
                    Ok(Lazy::Defer(right,
                                   Tape {
                                       env: tape.env.clone(),
                                       cont: box Cont::Verb(verb, left, 0, tape),
                                   }))
                }
                (_, AST::Number(_)) => {
                    Ok(Lazy::Defer(left,
                                   Tape {
                                       env: tape.env.clone(),
                                       cont: box Cont::Verb(verb, right, 1, tape),
                                   }))
                }
                (x, y) => {
                    Ok(Lazy::Defer(x,
                                   Tape {
                                       env: tape.env.clone(),
                                       cont: box Cont::Verb(verb, y, 0, tape),
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

fn lookup(name: u16, env: Rc<RefCell<Environment>>) -> Result<AST, Error> {
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

pub fn evaluate_fun(fun: AST, args: AST, tape: Tape) -> Result<Lazy, Error> {
    match fun {
        AST::Lambda(box names, box body) => {
            Tape {
                    env: tape.env.clone(),
                    cont: box Cont::Func(names, args, tape),
                }
                .run(body)
            // Ok(Lazy::Force(body, ))
            // Ok(Lazy::Force(body, Cont::Func(names, args, env, box k)))
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

pub fn evaluate_expr(exprs: AST, tape: Tape) -> Result<Lazy, Error> {
    match exprs.shift() {
        Some((car, cdr)) => {
            Ok(Lazy::Defer(car,
                           Tape {
                               env: tape.env.clone(),
                               cont: box Cont::Expressions(cdr, tape),
                           }))
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

    pub fn run(&mut self, ast: &mut AST) -> Result<AST, Error> {
        let mut a = 0;
        let mut b = try!(evaluate_expr(ast.clone(),
                                       Tape {
                                           env: self.root.clone(),
                                           cont: box Cont::Return,
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


impl Tape {
    pub fn run(mut self, val: AST) -> Result<Lazy, Error> {
        let box x = self.cont;
        match x {
            // Cont::Lambda(code, left, right, env, box k) =>
            // {
            // lambda::eval(code, left, right, env, val, k)
            // }
            //
            Cont::Call(callee, tape) => {
                match val {
                    AST::Dict(box v) => evaluate_fun(callee, v, tape),
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
                                           cont: box Cont::Cond(if_expr, else_expr, tape),
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
                                           cont: box Cont::Verb(verb, y, 0, tape),
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
