
// O-CPS INTERPRETER by 5HT et all

use streams::{verb, adverb, env, otree};
use commands::ast::{self, Error, AST, Verb, Adverb, Arena};

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
    Return(&'a AST<'a>),
}

pub struct Interpreter<'a> {
    arena: Arena<'a>,
    env: env::Environment<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Result<Interpreter<'a>, Error> {
        Ok(Interpreter {
            arena: Arena::new(),
            env: try!(env::Environment::new_root()),
        })
    }

    pub fn parse(&'a self, s: &String) -> &'a AST<'a> {
        ast::parse(&self.arena, s)
    }

    pub fn run(&'a self, ast: &'a AST<'a>) -> Result<&'a AST<'a>, Error> {
        let mut a = 0;
        let mut b = try!(self.evaluate_expr(self.env.last(), ast, self.arena.cont(Cont::Return)));
        loop {
            // println!("[Trampoline:{}]:{:?}\n", a, b);
            match b {
                &Lazy::Defer(f, x, t) => {
                    // a = a + 1;
                    b = try!(self.handle_defer(f, x, t))
                }
                &Lazy::Return(a) => {
                    // println!("Res: {:?}", a);
                    return Ok(a);
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

    fn handle_defer(&'a self,
                    node: &'a otree::Node<'a>,
                    a: &'a AST<'a>,
                    cont: &'a Cont<'a>)
                    -> Result<&'a Lazy<'a>, Error> {
        // println!("handle_defer: val: {:?} #### cont: {:?}\n", a, cont);
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
            &AST::Dict(x) => self.evaluate_dict(node, self.arena.ast(AST::Nil), x, cont),
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
                self.run_cont(closure.unwrap(), body, self.arena.cont(Cont::Func(names, rev, cont)))
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
                         node: &'a otree::Node<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<&'a Lazy<'a>, Error> {
        //println!("Eval Expr: {:?}", exprs);
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

    pub fn parse_return(&'a self, val: &'a AST<'a>, cont: &'a Cont<'a>) ->
        Result<&'a Lazy<'a>, Error> {
                match val {
                    &AST::Dict(x) => {
                        let mut rev = ast::rev_dict(x, &self.arena);
                        Ok(self.arena.lazy(Lazy::Return(self.arena.ast(AST::Dict(rev)))))
                    }
                    &AST::Cons(x, y) => {
                        let mut rev = ast::rev_dict(x, &self.arena);
                        let mut rev2 = ast::rev_dict(y, &self.arena);
                        Ok(self.arena
                            .lazy(Lazy::Return(self.arena
                                .ast(AST::Dict(self.arena.ast(AST::Cons(rev2, rev)))))))
                    }
                    x => {
                        Ok(self.arena.lazy(Lazy::Return(x)))
                    }
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
            &Cont::Dict(acc, rest, cont) => {
                // println!("Cont Dict: rest {:?} val {:?} acc {:?}", rest, val, acc);
                let new_acc;
                match val {
                    &AST::Cons(x, y) => new_acc = self.arena.ast(AST::Cons(self.arena.ast(AST::Dict(val)), acc)),
                    _ => {
                        // println!("Acc: {:?}", val);
                        new_acc = self.arena.ast(AST::Cons(val, acc))
                    }
                }
                match rest {
                    &AST::Cons(head, tail) => {
                        match head {
                            &AST::Number(_) => {
                                self.run_cont(node,
                                              head,
                                              self.arena
                                                  .cont(Cont::Dict(new_acc, tail, cont)))
                            }
                            x => {
                                self.evaluate_dict(node,
                                                   self.arena.ast(AST::Nil),
                                                   head,
                                                   self.arena
                                                       .cont(Cont::Dict(new_acc, tail, cont)))
                            }
                        }
                    }
                    &AST::Number(s) => {
                        // println!("Number: {:?} -- {:?}", acc, val);
                        self.run_cont(node, self.arena.ast(AST::Cons(rest, new_acc)), cont)
                    }
                    &AST::Nil => {
                        // println!("Nil: {:?} -- {:?}", acc, val);
                        self.run_cont(node, new_acc, cont)
                    }
                    &AST::NameInt(name) => {
                        let l = self.env.get(name, node);
                        match l {
                            Some((v, f)) => self.run_cont(f, self.arena.ast(AST::Cons(v, new_acc)), cont),
                            None => {
                                Err(Error::EvalError {
                                    desc: "".to_string(),
                                    ast: "".to_string(),
                                })
                            }
                        }
                    }
                    x => {
                        Ok(self.arena.lazy(Lazy::Defer(node,
                                                       x,
                                                       self.arena
                                                           .cont(Cont::Dict(new_acc, self.arena.ast(AST::Nil), cont)))))
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
                        //println!("Assign: {:?}:{:?}", s, val);
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
                self.parse_return(val, con)
                // println!("Return: {:?} {:?}", cont, val);
            }
        }
    }
}

