
// O-CPS INTERPRETER by 5HT et all

use streams::{verb, adverb, env, otree};
use commands::ast::{self, Error, AST, Verb, Adverb, Arena};
use streams::intercore::ctx::{Ctx, Ctxs};
use streams::intercore::internals;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub enum Cont<'a> {
    Expressions(&'a AST<'a>, &'a Cont<'a>),
    Assign(&'a AST<'a>, &'a Cont<'a>),
    Cond(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Func(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    List(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Dict(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Call(&'a AST<'a>, &'a Cont<'a>),
    Verb(Verb, &'a AST<'a>, u8, &'a Cont<'a>),
    Adverb(Adverb, &'a AST<'a>, &'a Cont<'a>),
    Return,
}

#[derive(Clone, Debug)]
pub enum Lazy<'a> {
    Defer(&'a otree::Node<'a>, &'a AST<'a>, &'a Cont<'a>),
    Continuation(&'a otree::Node<'a>, &'a AST<'a>, &'a Cont<'a>),
    Return(&'a AST<'a>),
}

pub struct Interpreter<'a> {
    env: env::Environment<'a>,
    arena: Arena<'a>,
    ctx: Ctx<u64>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Result<Interpreter<'a>, Error> {
        let env = try!(env::Environment::new_root());
        let interpreter = Interpreter {
            arena: Arena::new(),
            env: env,
            ctx: Ctx::new(),
        };
        Ok(interpreter)
    }

    pub fn define_primitives(&'a mut self) {
        let print = self.arena.intern("print".to_string());
        let publ = self.arena.intern("pub".to_string());
        let subs = self.arena.intern("sub".to_string());
        let snd = self.arena.intern("snd".to_string());
        let rcv = self.arena.intern("rcv".to_string());

        self.env.define(ast::extract_name(print), print);
        self.env.define(ast::extract_name(publ), publ);
        self.env.define(ast::extract_name(subs), subs);
        self.env.define(ast::extract_name(snd), snd);
        self.env.define(ast::extract_name(rcv), rcv);
    }

    pub fn parse(&'a mut self, s: &String) -> &'a AST<'a> {
        let (s1, s2) = split(self);
        let x = unsafe { &mut *s2.arena.asts.get() };
        s1.define_primitives();
        s2.arena.builtins = x.len() as u16;
        println!("Primitives: {:?}", s2.arena.builtins);
        ast::parse(&s2.arena, s)
    }

    pub fn run(&'a mut self, ast: &'a AST<'a>) -> Result<&'a AST<'a>, Error> {

        let mut counter = 0;
        println!("Input: {:?}", ast);
        let uc = UnsafeCell::new(self);
        let se1: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        let se2: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        let se3: &mut Interpreter<'a> = unsafe { &mut *uc.get() };


        let mut tick = try!(se1.evaluate_expr(se2.env.last(), ast, se3.arena.cont(Cont::Return)));
        loop {
            let se4: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
            match tick {
                &Lazy::Defer(node, ast, cont) => {
                    tick = try!({
                        counter = counter + 1;
                        se4.handle_defer(node, ast, cont)
                    })
                }
                &Lazy::Continuation(node, ast, cont) => {
                    return Ok(se4.arena.ast(AST::Retry));
                }
                &Lazy::Return(ast) => {
                    // println!("Result: {:?}", ast);
                    // println!("env: {:?}", self.env.dump());
                    // println!("arena: {:?}", self.arena.dump());
                    return Ok(ast);
                }
            }
        }
        Err(Error::EvalError {
            desc: "Program is terminated abnormally".to_string(),
            ast: format!("{:?}", AST::Nil),
        })
    }

    pub fn gc(&self) -> usize {
        self.env.clean() + self.arena.clean()
    }

    fn handle_defer(&'a mut self,
                    node: &'a otree::Node<'a>,
                    a: &'a AST<'a>,
                    cont: &'a Cont<'a>)
                    -> Result<&'a Lazy<'a>, Error> {
        // println!("handle_defer: val: {:?} #### cont: {:?}\n", a, cont);
        // let se = UnsafeCell::new(self);
        // let s1 = unsafe {&mut *se.get()};
        match a {
            &AST::Assign(name, body) => {
                Ok(self.arena.lazy(Lazy::Defer(node, body, self.arena.cont(Cont::Assign(name, cont)))))
            }
            &AST::Cond(val, left, right) => {
                Ok(self.arena
                    .lazy(Lazy::Defer(node, val, self.arena.cont(Cont::Cond(left, right, cont)))))
            }
            &AST::List(x) => self.evaluate_expr(node, x, cont),
            &AST::Dict(x) => self.evaluate_dict(node, self.arena.nil(), x, cont),
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
            &AST::Lambda(_, x, y) => self.run_cont(node, self.arena.ast(AST::Lambda(Some(node), x, y)), cont),
            x => self.run_cont(node, x, cont),
        }
    }

    fn lookup(&'a self,
              node: &'a otree::Node<'a>,
              name: u16,
              env: &'a env::Environment<'a>)
              -> Result<(&'a AST<'a>, &'a otree::Node<'a>), Error> {
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
                        node: &'a otree::Node<'a>,
                        fun: &'a AST<'a>,
                        args: &'a AST<'a>,
                        cont: &'a Cont<'a>)
                        -> Result<&'a Lazy<'a>, Error> {
        // println!("Eval Fun: {:?}", fun);
        match fun {
            &AST::Lambda(closure, names, body) => {
                let mut rev = ast::rev_dict(args, &self.arena);
                // println!("Args Fun: {:?} orig: {:?}, names: {:?}", rev, args, names);
                self.run_cont(if closure == None {
                                  node
                              } else {
                                  closure.unwrap()
                              },
                              body,
                              self.arena.cont(Cont::Func(names, rev, cont)))
            }
            &AST::NameInt(s) => {
                println!("lookup: {:?}", s);
                let v = self.lookup(node, s, &self.env);
                match v {
                    Ok((c, f)) => {
                        match c {
                            &AST::NameInt(n) if n < self.arena.builtins => {
                                self.run_cont(f, self.arena.ast(internals(n, args, &self.ctx)), cont)
                            }
                            _ => self.evaluate_fun(f, c, args, cont),
                        }
                    }
                    Err(x) => Err(x),
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
                         node: &'a otree::Node<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<&'a Lazy<'a>, Error> {
        // println!("Eval Expr: {:?}", exprs);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(self.arena.lazy(Lazy::Defer(node,
                                               car,
                                               self.arena
                                                   .cont(Cont::Expressions(cdr, cont)))))
            }
            &AST::Nil => self.run_cont(node, exprs, cont),
            x => Ok(self.arena.lazy(Lazy::Defer(node, x, cont))),
        }
    }

    pub fn evaluate_list(&'a self,
                         node: &'a otree::Node<'a>,
                         acc: &'a AST<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<&'a Lazy<'a>, Error> {
        // println!("Eval List: {:?}", exprs);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(self.arena.lazy(Lazy::Defer(node,
                                               car,
                                               self.arena
                                                   .cont(Cont::List(acc, cdr, cont)))))
            }
            &AST::Nil => self.run_cont(node, acc, cont),
            x => Ok(self.arena.lazy(Lazy::Defer(node, x, cont))),
        }
    }

    pub fn evaluate_dict(&'a self,
                         node: &'a otree::Node<'a>,
                         acc: &'a AST<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<&'a Lazy<'a>, Error> {
        // println!("Eval Dict: {:?}", exprs);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(self.arena.lazy(Lazy::Defer(node,
                                               car,
                                               self.arena
                                                   .cont(Cont::Dict(acc, cdr, cont)))))
            }
            &AST::Nil => self.run_cont(node, acc, cont),
            x => Ok(self.arena.lazy(Lazy::Defer(node, x, cont))),
        }
    }

    pub fn emit_return(&'a self, val: &'a AST<'a>, cont: &'a Cont<'a>) -> Result<&'a Lazy<'a>, Error> {
        match val {
            &AST::Dict(x) => {
                let mut dict = ast::rev_dict(x, &self.arena);
                Ok(self.arena.lazy(Lazy::Return(self.arena.ast(AST::Dict(dict)))))
            }
            &AST::Cons(x, y) => {
                let mut head = ast::rev_dict(x, &self.arena);
                let mut tail = ast::rev_dict(y, &self.arena);
                Ok(self.arena
                    .lazy(Lazy::Return(self.arena
                        .ast(AST::Dict(self.arena.ast(AST::Cons(tail, head)))))))
            }
            x => Ok(self.arena.lazy(Lazy::Return(x))),
        }
    }

    pub fn run_cont(&'a self,
                    node: &'a otree::Node<'a>,
                    val: &'a AST<'a>,
                    con: &'a Cont<'a>)
                    -> Result<&'a Lazy<'a>, Error> {
        // println!("run_cont: val: {:?} #### cont: {:?}\n", val, cont);
        match con {
            &Cont::Call(callee, cont) => {
                match val {
                    &AST::Dict(v) => self.evaluate_fun(node, callee, v, cont),
                    x => self.evaluate_fun(node, callee, x, cont),
                }
            }
            &Cont::Func(names, args, cont) => {
                let f = self.env.new_child(node);
                for (k, v) in names.into_iter().zip(args.into_iter()) {
                    self.env.define(ast::extract_name(k), v);
                }
                self.evaluate_expr(f, val, cont)
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
                        // println!("Assign: {:?}:{:?}", s, val);
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
            &Cont::Dict(acc, rest, cont) => {
                let new_acc;
                match val {
                    &AST::Cons(x, y) => new_acc = self.arena.ast(AST::Cons(self.arena.ast(AST::Dict(val)), acc)),
                    _ => new_acc = self.arena.ast(AST::Cons(val, acc)), 
                };
                match rest {
                    &AST::Cons(head, tail) => {
                        self.evaluate_dict(node,
                                           self.arena.nil(),
                                           head,
                                           self.arena
                                               .cont(Cont::Dict(new_acc, tail, cont)))
                    }
                    &AST::Number(s) => self.run_cont(node, self.arena.ast(AST::Cons(rest, new_acc)), cont),
                    &AST::Nil => self.run_cont(node, new_acc, cont),
                    &AST::NameInt(name) => {
                        match self.lookup(node, name, &self.env) {
                            Ok((v, f)) => self.run_cont(f, self.arena.ast(AST::Cons(v, new_acc)), cont),
                            Err(x) => Err(x),
                        }
                    }
                    x => {
                        Ok(self.arena.lazy(Lazy::Defer(node,
                                                       x,
                                                       self.arena
                                                           .cont(Cont::Dict(new_acc, self.arena.nil(), cont)))))
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
                self.emit_return(val, con)
                // println!("Return: {:?} {:?}", cont, val);
            }
        }
    }
}

pub struct Handle<T>(UnsafeCell<T>);

impl<T> Handle<T> {
    pub fn borrow(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    pub fn borrow_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

pub fn handle<T>(t: T) -> Handle<T> {
    Handle(UnsafeCell::new(t))
}

impl<T> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.borrow()
    }
}

impl<T> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.borrow_mut()
    }
}

pub fn split<T>(t: &mut T) -> (&mut T, &mut T) {
    let f: *mut T = t;
    let uf: &mut T = unsafe { &mut *f };
    let us: &mut T = unsafe { &mut *f };
    (uf, us)
}