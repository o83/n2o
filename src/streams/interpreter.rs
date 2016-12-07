
// O-CPS INTERPRETER by 5HT et all

use std::fmt;
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use streams::verb;
use streams::verb::*;
use streams::env::*;
use commands::ast::*;
use commands::ast;
use fnv::*;
type Linked<K, V> = FnvHashMap<K, V>;

// Interpreter, Lazy and Cont

#[derive(Clone)]
pub struct Interpreter {
    pub root: Rc<RefCell<Environment>>,
    pub names_size: u16,
    pub symbols_size: u16,
    pub sequences_size: u16,
    pub names: Linked<String, u16>,
    pub symbols: Linked<String, u16>,
    pub sequences: Linked<String, u16>,
}

#[derive(Clone, Debug)]
pub enum Lazy {
    Defer(AST, Rc<RefCell<Environment>>, Cont),
    Force(AST, Cont),
    Return(AST),
}

// Plug Any Combinators here

#[derive(PartialEq, Clone, Debug)]
pub enum Cont {
    Expressions(AST, Rc<RefCell<Environment>>, Box<Cont>),
    Assign(AST, Rc<RefCell<Environment>>, Box<Cont>),
    Cond(AST, AST, Rc<RefCell<Environment>>, Box<Cont>),
    Func(AST, AST, Rc<RefCell<Environment>>, Box<Cont>),
    Call(AST, AST, Rc<RefCell<Environment>>, Box<Cont>),
    Verb(Verb, AST, u8, Rc<RefCell<Environment>>, Box<Cont>),
    Adverb(Adverb, AST, Rc<RefCell<Environment>>, Box<Cont>),
    Return,
}

fn process(exprs: &mut AST, env: Rc<RefCell<Environment>>) -> Result<AST, Error> {
    if exprs.len() == 0 {
        return Ok(AST::Nil);
    }
    let mut a = 0;
    let mut b = try!(evaluate_expressions(exprs.clone(), env.clone(), Box::new(Cont::Return)));
    //  while a < 5 {
    loop {
        debug!("[Trampoline:{}]:{:?}\n", a, b);
        match b {
            Lazy::Defer(a, e, k) => b = try!(handle_defer(a, e, k)),
            Lazy::Force(x, k) => {
                a = a + 1;
                b = try!(k.run(x))
            }
            Lazy::Return(a) => return Ok(a),
        }
    }
    Err(Error::EvalError {
        desc: "".to_string(),
        ast: exprs.clone(),
    })
}

fn handle_defer(a: AST, env: Rc<RefCell<Environment>>, k: Cont) -> Result<Lazy, Error> {
    match a {
        AST::Assign(box name, box body) => {
            Ok(Lazy::Defer(body, env.clone(), Cont::Assign(name, env, box k)))
        }
        AST::List(box x) => evaluate_expressions(x, env, box k),
        AST::Call(box c, box a) => {
            Ok(Lazy::Defer(a.clone(), env.clone(), Cont::Call(c, a, env, box k)))
        }
        AST::Verb(verb, box left, box right) => {
            match (left.clone(), right.clone()) {
                (AST::Number(_), _) => {
                    Ok(Lazy::Defer(right, env.clone(), Cont::Verb(verb, left, 0, env, box k)))
                }
                (_, AST::Number(_)) => {
                    Ok(Lazy::Defer(left, env.clone(), Cont::Verb(verb, right, 1, env, box k)))
                }
                (x, y) => Ok(Lazy::Defer(x, env.clone(), Cont::Verb(verb, y, 0, env, box k))),
            }
        }
        AST::Cond(box val, box left, box right) => {
            match val {
                AST::Number(x) => Ok(Lazy::Force(val, Cont::Cond(left, right, env, box k))),
                x => Ok(Lazy::Defer(x, env.clone(), Cont::Cond(left, right, env.clone(), box k))),
            }
        }
        AST::NameInt(name) => {
            match lookup(name, env) {
                Ok(v) => k.run(v),
                Err(x) => Err(x),
            }
        }
        x => k.run(x),
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

fn evaluate_function(fun: AST,
                     env: Rc<RefCell<Environment>>,
                     args: AST,
                     k: Cont)
                     -> Result<Lazy, Error> {
    match fun {
        AST::Lambda(box names, box body) => {
            Ok(Lazy::Force(body, Cont::Func(names, args, env, box k)))
        }
        AST::NameInt(s) => {
            match env.borrow().find(&s) {
                Some((v, x)) => evaluate_function(v, x, args, k),
                None => {
                    Err(Error::EvalError {
                        desc: "Function Name in all Contexts".to_string(),
                        ast: AST::NameInt(s),
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
                        k: Box<Cont>)
                        -> Result<Lazy, Error> {
    match exprs.shift() {
        Some((car, cdr)) => Ok(Lazy::Defer(car, env.clone(), Cont::Expressions(cdr, env, k))),
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
            names: FnvHashMap::with_capacity_and_hasher(10, Default::default()),
            symbols: FnvHashMap::with_capacity_and_hasher(10, Default::default()),
            sequences: FnvHashMap::with_capacity_and_hasher(10, Default::default()),
        })
    }

    pub fn run(&mut self, program: &mut AST) -> Result<AST, Error> {
        process(program, self.root.clone())
    }

}


impl Cont {
    pub fn run(self, val: AST) -> Result<Lazy, Error> {
        match self {
            Cont::Expressions(rest, env, k) => {
                if rest.is_cons() || !rest.is_empty() {
                    evaluate_expressions(rest, env, k)
                } else {
                    Ok(Lazy::Force(val, *k))
                }
            }
            Cont::Call(callee, args, env, k) => {
                match args {
                    AST::Dict(box v) => evaluate_function(callee, env, v, *k),
                    _ => evaluate_function(callee, env, val, *k),
                }
            }
            Cont::Func(names, args, env, k) => {
                let local_env = Environment::new_child(env);
                for (name, value) in names.into_iter().zip(args.into_iter()) {
                    try!(local_env.borrow_mut().define(ast::extract_name(name), value));
                }
                evaluate_expressions(val, local_env, k)

            }
            Cont::Cond(if_expr, else_expr, env, k) => {
                match val {
                    AST::Number(0) => Ok(Lazy::Defer(else_expr, env.clone(), *k)),
                    AST::Number(_) => Ok(Lazy::Defer(if_expr, env.clone(), *k)),
                    x => Ok(Lazy::Defer(x, env.clone(), Cont::Cond(if_expr, else_expr, env, k))),
                }
            }
            Cont::Verb(verb, right, swap, env, box k) => {
                match (right.clone(), val.clone()) {
                    (AST::Number(_), AST::Number(_)) => {
                        match swap {
                            0 => Ok(Lazy::Force(verb::eval(verb, right, val).unwrap(), k)),
                            _ => Ok(Lazy::Force(verb::eval(verb, val, right).unwrap(), k)),
                        }
                    }
                    (x, y) => {
                        Ok(Lazy::Defer(x, env.clone(), Cont::Verb(verb, y, 0, env.clone(), box k)))
                    }
                }
            }
            Cont::Assign(name, env, k) => {
                match name {
                    AST::NameInt(s) => {
                        try!(env.borrow_mut().define(s, val.clone()));
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
            _ => Ok(Lazy::Return(val)),
        }
    }
}
