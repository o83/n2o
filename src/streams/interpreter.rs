
// O-CPS INTERPRETER by 5HT et all

use streams::{verb, env, otree};
use commands::ast::{self, Error, AST, Verb, Adverb, Arena};
use streams::intercore::ctx::Ctx;
use streams::intercore::internals;
use std::cell::UnsafeCell;
use handle::split;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Cont<'a> {
    Expressions(&'a AST<'a>, &'a Cont<'a>),
    Assign(&'a AST<'a>, &'a Cont<'a>),
    Cond(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Func(&'a AST<'a>, &'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    List(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Dict(&'a AST<'a>, &'a AST<'a>, &'a Cont<'a>),
    Call(&'a AST<'a>, &'a Cont<'a>),
    Verb(Verb, &'a AST<'a>, u8, &'a Cont<'a>),
    Adverb(Adverb, &'a AST<'a>, &'a Cont<'a>),
    Return,
    Yield(&'a Cont<'a>),
}

#[derive(Clone, Debug)]
pub enum Lazy<'a> {
    Defer(&'a otree::Node<'a>, &'a AST<'a>, &'a Cont<'a>),
    Continuation(&'a otree::Node<'a>, &'a AST<'a>, &'a Cont<'a>),
    Return(&'a AST<'a>),
    Start,
}


pub struct Interpreter<'a> {
    env: env::Environment<'a>,
    arena: Arena<'a>,
    ctx: Rc<Ctx>,
    registers: Lazy<'a>,
    counter: u64,
}

impl<'a> Interpreter<'a> {
    pub fn new2(ctx: Rc<Ctx>) -> Result<Interpreter<'a>, Error> {
        let mut env = try!(env::Environment::new_root());
        let mut arena = Arena::new();
        let mut interpreter = Interpreter {
            arena: arena,
            env: env,
            ctx: ctx,
            registers: Lazy::Start,
            counter: 1,
        };
        Ok(interpreter)
    }

    pub fn new() -> Result<Interpreter<'a>, Error> {
        let mut env = try!(env::Environment::new_root());
        let mut arena = Arena::new();
        let mut interpreter = Interpreter {
            arena: arena,
            env: env,
            ctx: Rc::new(Ctx::new()),
            registers: Lazy::Start,
            counter: 1,
        };
        Ok(interpreter)
    }

    pub fn define_primitives(&'a mut self) {
        let (s1, s2) = split(self);
        let print = s1.arena.intern("print".to_string());
        let publ = s1.arena.intern("pub".to_string());
        let subs = s1.arena.intern("sub".to_string());
        let snd = s1.arena.intern("snd".to_string());
        let rcv = s1.arena.intern("rcv".to_string());
        let spawn = s1.arena.intern("spawn".to_string());
        s1.env.define(ast::extract_name(print), print);
        s1.env.define(ast::extract_name(publ), publ);
        s1.env.define(ast::extract_name(subs), subs);
        s1.env.define(ast::extract_name(snd), snd);
        s1.env.define(ast::extract_name(rcv), rcv);
        s1.env.define(ast::extract_name(spawn), spawn);
        let x = unsafe { &mut *s1.arena.asts.get() };
        s2.arena.builtins = x.len() as u16;
    }

    pub fn parse(&'a mut self, s: &String) -> &'a AST<'a> {
        ast::parse(&self.arena, s)
    }

    pub fn load(&'a mut self, ast: &'a AST<'a>) {
        let (s1, s2) = split(self);
        match s2.registers {
            Lazy::Continuation(node, _, cont) => {
                s1.env = env::Environment::new_root().unwrap();
                s1.registers = Lazy::Continuation(s2.env.last(), ast, s2.arena.cont(Cont::Return))
            }
            ref x => (),
        }
    }

    pub fn run(&'a mut self, ast: &'a AST<'a>) -> Result<&'a AST<'a>, Error> {
        let uc = UnsafeCell::new(self);
        let se1: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        let se2: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        let se3: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        let mut tick;
        match se3.registers {
            Lazy::Start => tick = try!(se1.evaluate_expr(se2.env.last(), ast, se3.arena.cont(Cont::Return))),
            _ => tick = se3.registers.clone(),
        }
        println!("Count: {:?}", se1.counter);
        loop {
            let se4: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
            let mut counter = se1.counter;
            match tick {
                Lazy::Defer(node, ast, cont) => {
                    if counter % 1000000 == 0 {
                        se4.registers = tick;
                        se3.counter = counter + 1;
                        return Ok(se4.arena.ast(AST::Yield));
                    } else {
                        tick = try!({
                            se3.counter = counter + 1;
                            se4.handle_defer(node, ast, cont)
                        })
                    }
                }
                Lazy::Start => break,
                Lazy::Continuation(node, ast, cont) => {
                    se4.registers = Lazy::Defer(node, ast, cont);
                    se3.counter = counter + 1;
                    return Ok(se4.arena.ast(AST::Yield));
                }
                Lazy::Return(ast) => {
                    // println!("env: {:?}", se3.env.dump());
                    // println!("arena: {:?}", se4.arena.dump());
                    // println!("Result: {}", ast);
                    se3.counter = counter + 1;
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
                    -> Result<Lazy<'a>, Error> {
        match a {
            &AST::Assign(name, body) => Ok(Lazy::Defer(node, body, self.arena.cont(Cont::Assign(name, cont)))),
            &AST::Cond(val, left, right) => Ok(Lazy::Defer(node, val, self.arena.cont(Cont::Cond(left, right, cont)))),
            &AST::List(x) => self.evaluate_expr(node, x, cont),
            &AST::Dict(x) => self.evaluate_dict(node, self.arena.nil(), x, cont),
            &AST::Call(c, a) => Ok(Lazy::Defer(node, a, self.arena.cont(Cont::Call(c, cont)))),
            &AST::Verb(ref verb, left, right) => {
                match (left, right) {
                    (&AST::Number(_), _) => {
                        Ok(Lazy::Defer(node,
                                       right,
                                       self.arena.cont(Cont::Verb(verb.clone(), left, 0, cont))))
                    }
                    (_, &AST::Number(_)) => {
                        Ok(Lazy::Defer(node,
                                       left,
                                       self.arena
                                           .cont(Cont::Verb(verb.clone(), right, 1, cont))))
                    }
                    (x, y) => {
                        Ok(Lazy::Defer(node,
                                       x,
                                       self.arena
                                           .cont(Cont::Verb(verb.clone(), y, 0, cont))))
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
                        -> Result<Lazy<'a>, Error> {
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
                              self.arena.cont(Cont::Func(names, rev, body, cont)))
            }
            &AST::NameInt(s) => {
                // println!("{:?}", s);
                let v = self.lookup(node, s, &self.env);
                match v {
                    Ok((c, f)) => {
                        match c {
                            &AST::NameInt(n) if n < self.arena.builtins => {
                                let x = self.arena.ast(internals(n, args, &self.ctx));
                                match x {
                                    &AST::Yield => self.run_cont(f, x, self.arena.cont(Cont::Yield(cont))),
                                    _ => self.run_cont(f, x, cont),
                                }
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
                         -> Result<Lazy<'a>, Error> {
        // println!("Eval Expr: {:?}", exprs);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(Lazy::Defer(node,
                               car,
                               self.arena
                                   .cont(Cont::Expressions(cdr, cont))))
            }
            &AST::Nil => self.run_cont(node, exprs, cont),
            x => Ok(Lazy::Defer(node, x, cont)),
        }
    }

    pub fn evaluate_list(&'a self,
                         node: &'a otree::Node<'a>,
                         acc: &'a AST<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<Lazy<'a>, Error> {
        // println!("Eval List: {:?}", exprs);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(Lazy::Defer(node,
                               car,
                               self.arena
                                   .cont(Cont::List(acc, cdr, cont))))
            }
            &AST::Nil => self.run_cont(node, acc, cont),
            x => Ok(Lazy::Defer(node, x, cont)),
        }
    }

    pub fn evaluate_dict(&'a self,
                         node: &'a otree::Node<'a>,
                         acc: &'a AST<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<Lazy<'a>, Error> {
        // println!("Eval Dict: {:?}", exprs);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(Lazy::Defer(node,
                               car,
                               self.arena
                                   .cont(Cont::Dict(acc, cdr, cont))))
            }
            &AST::Nil => self.run_cont(node, acc, cont),
            x => Ok(Lazy::Defer(node, x, cont)),
        }
    }

    pub fn emit_return(&'a self, val: &'a AST<'a>, cont: &'a Cont<'a>) -> Result<Lazy<'a>, Error> {
        // println!("Emit: {:?}", val);
        match val {
            &AST::Dict(x) => {
                let mut dict = ast::rev_dict(x, &self.arena);
                Ok(Lazy::Return(self.arena.ast(AST::Dict(dict))))
            }
            &AST::Cons(x, y) => {
                Ok(Lazy::Return(self.arena
                    .ast(AST::Dict(ast::rev_dict(self.arena.ast(AST::Cons(x, y)), &self.arena)))))
            }
            x => Ok(Lazy::Return(x)),
        }
    }

    pub fn run_cont(&'a self,
                    node: &'a otree::Node<'a>,
                    val: &'a AST<'a>,
                    con: &'a Cont<'a>)
                    -> Result<Lazy<'a>, Error> {
        // println!("run_cont: val: {:?} #### cont: {:?}\n", val, cont);
        match con {
            &Cont::Yield(cc) => Ok(Lazy::Continuation(node, val, cc)),
            &Cont::Call(callee, cont) => {
                let c;
                match val {
                    &AST::Dict(v) => c = self.evaluate_fun(node, callee, v, cont),
                    x => c = self.evaluate_fun(node, callee, x, cont),
                };
                c
            }
            &Cont::Func(names, args, body, cont) => {
                // println!("names={:?} args={:?}", names, args);
                let f = self.env.new_child(node);
                let mut partial = self.arena.nil(); // empty list of unfilled/empty arguments
                for (k, v) in names.into_iter().zip(args.into_iter()) {
                    match v {
                        &AST::Any => partial = self.arena.ast(AST::Cons(k, partial)),
                        _ => {
                            self.env.define(ast::extract_name(k), v);
                        }
                    };
                }
                if partial == &AST::Nil {
                    self.evaluate_expr(f, val, cont)
                } else {
                    Ok(Lazy::Defer(f, self.arena.ast(AST::Lambda(Some(f), partial, body)), cont))
                }
            }
            &Cont::Cond(if_expr, else_expr, cont) => {
                match val {
                    &AST::Number(0) => Ok(Lazy::Defer(node, else_expr, cont)),
                    &AST::Number(_) => Ok(Lazy::Defer(node, if_expr, cont)),
                    x => {
                        Ok(Lazy::Defer(node,
                                       x,
                                       self.arena.cont(Cont::Cond(if_expr, else_expr, cont))))
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
                // println!("Cont Dict: {:?} {:?}", val, rest);
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
                        Ok(Lazy::Defer(node,
                                       x,
                                       self.arena
                                           .cont(Cont::Dict(new_acc, self.arena.nil(), cont))))
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
                    (&AST::VecInt(_), &AST::Number(_)) => {
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
                    (&AST::VecInt(_), &AST::VecInt(_)) => {
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
                        Ok(Lazy::Defer(node,
                                       x,
                                       self.arena
                                           .cont(Cont::Verb(verb.clone(), y, 0, cont))))
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
                let o = self.emit_return(val, con);
                // println!("Return: {:?}", o);
                o
            }
        }
    }
}
