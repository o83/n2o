
// O-CPS INTERPRETER by 5HT et all

use streams::{verb, env, otree};
use commands::ast::{self, Error, AST, Verb, Adverb, Arena, Value};
use intercore::bus::Memory;
use intercore::client::{handle_context, internals};
use reactors::task::Context;
use intercore::message::Message;
use reactors::scheduler::Scheduler;
use handle::{self, into_raw, from_raw, UnsafeShared};

unsafe impl<'a> Sync for Cont<'a> {}
unsafe impl<'a> Sync for AST<'a> {}
unsafe impl Sync for Message {}

const PREEMPTION: u64 = 20000000; // Yield each two instructions

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
    Intercore(Message, &'a Cont<'a>),
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
    pub env: env::Environment<'a>,
    pub arena: Arena<'a>,
    pub queues: UnsafeShared<Memory>,
    pub edge: Message,
    pub registers: Lazy<'a>,
    pub counter: u64,
    pub task_id: usize,
}

impl<'a> Interpreter<'a> {
    pub fn new(mem_ptr: UnsafeShared<Memory>) -> Result<Interpreter<'a>, Error> {
        let mut env = try!(env::Environment::new_root());
        let mut arena = Arena::new();
        let mut interpreter = Interpreter {
            arena: arena,
            env: env,
            queues: mem_ptr,
            edge: Message::Nop,
            registers: Lazy::Start,
            task_id: 0,
            counter: 1,
        };
        Ok(interpreter)
    }

    pub fn define_primitives(&'a mut self) {
        let (s1, s2) = handle::split(self);
        let print = s1.arena.intern("print".to_string());
        let publ = s1.arena.intern("pub".to_string());
        let subs = s1.arena.intern("sub".to_string());
        let snd = s1.arena.intern("snd".to_string());
        let rcv = s1.arena.intern("rcv".to_string());
        let spawn = s1.arena.intern("spawn".to_string());
        let select = s1.arena.intern("select".to_string());
        s1.env.define(ast::extract_name(print), print);
        s1.env.define(ast::extract_name(publ), publ);
        s1.env.define(ast::extract_name(subs), subs);
        s1.env.define(ast::extract_name(snd), snd);
        s1.env.define(ast::extract_name(rcv), rcv);
        s1.env.define(ast::extract_name(spawn), spawn);
        s1.env.define(ast::extract_name(select), select);
        let x = unsafe { &mut *s1.arena.asts.get() };
        s2.arena.builtins = x.len() as u16;
    }

    pub fn parse(&'a mut self, s: &String) -> &'a AST<'a> {
        ast::parse(&self.arena, s)
    }

    pub fn load(&'a mut self, ast: &'a AST<'a>) {
        let (s1, s2) = handle::split(self);
        match s2.registers {
            Lazy::Continuation(node, _, cont) => {
                s1.env = env::Environment::new_root().unwrap();
                s1.registers = Lazy::Continuation(s2.env.last(), ast, s2.arena.cont(Cont::Return))
            }
            ref x => (),
        }
    }

    pub fn run(&'a mut self,
               ast: &'a AST<'a>,
               intercore: Context<'a>,
               sched: Option<&'a Scheduler<'a>>)
               -> Result<&'a AST<'a>, Error> {
        let h = into_raw(self);
        let mut tick;
        let mut value_from_sched = from_raw(h).arena.nil();


        match from_raw(h).registers {
            Lazy::Start => {
                tick = try!(from_raw(h).evaluate_expr(from_raw(h).env.last(),
                                                      ast,
                                                      from_raw(h).arena.cont(Cont::Return)))
            }
            _ => tick = from_raw(h).registers.clone(),
        }

        match intercore.clone() {
            Context::NodeAck(value) => {
                value_from_sched = from_raw(h).arena.ast(AST::Value(Value::Number(value as i64)));
                // println!("intercore: {:?}", value_from_sched);
            }
            //            Context::Nil => return Ok(self.arena.nil()),
            _ => (),
        }

        // println!("Counter: {:?}", ast);
        loop {
            let mut counter = from_raw(h).counter;
            match tick {
                Lazy::Defer(node, ast_, cont) => {
                    if counter % PREEMPTION == 0 {
                        from_raw(h).registers = tick;
                        from_raw(h).counter = counter + 1;
                        return Ok(from_raw(h).arena.ast(AST::Yield(Context::Nil)));
                    } else {
                        tick = try!({
                            from_raw(h).edge = Message::Nop;
                            let a = match ast_ {
                                &AST::Yield(..) => value_from_sched,
                                x => x,
                            };
                            // println!("ast_: {:?}", a);
                            from_raw(h).counter = counter + 1;
                            from_raw(h).handle_defer(node, a, cont)
                        })
                    }
                }
                Lazy::Start => break,
                Lazy::Continuation(node, ast, cont) => {
                    let ast_ = from_raw(h).arena.ast(AST::Yield(Context::Intercore(&from_raw(h).edge)));
                    from_raw(h).registers = Lazy::Defer(node, ast, cont);
                    from_raw(h).counter = counter + 1;
                    return Ok(ast_);
                }
                Lazy::Return(ast) => {
                    // DEBUG
                    // println!("env: {:?}", se3.env.dump());
                    // println!("arena: {:?}", se4.arena.dump());
                    // INFO
/*
                    println!("Instructions: {}", counter);
                    println!("Conts: {}", from_raw(h).arena.cont_len());
                    println!("ASTs: {}", from_raw(h).arena.ast_len());
                    println!("ENV: ({},{})",
                             from_raw(h).env.len().0,
                             from_raw(h).env.len().1);
                    // NORMAL
                    println!("Result: {}", ast);
*/
                    from_raw(h).counter = counter + 1;
                    from_raw(h).edge = Message::Nop;
                    from_raw(h).registers = Lazy::Start;
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
        let h = into_raw(self);

        match a {
            &AST::Assign(name, body) => Ok(Lazy::Defer(node, body, from_raw(h).arena.cont(Cont::Assign(name, cont)))),
            &AST::Cond(val, left, right) => {
                Ok(Lazy::Defer(node,
                               val,
                               from_raw(h).arena.cont(Cont::Cond(left, right, cont))))
            }
            &AST::List(x) => from_raw(h).evaluate_expr(node, x, cont),
            &AST::Dict(x) => from_raw(h).evaluate_dict(node, from_raw(h).arena.nil(), x, cont),
            &AST::Call(c, a) => Ok(Lazy::Defer(node, a, from_raw(h).arena.cont(Cont::Call(c, cont)))),
            &AST::Verb(ref verb, left, right) => {
                // println!("Defer Verb: {:?} {:?}", left, right);
                match (left, right) {
                    (&AST::Value(_), _) => {
                        Ok(Lazy::Defer(node,
                                       right,
                                       from_raw(h).arena.cont(Cont::Verb(verb.clone(), left, 0, cont))))
                    }
                    (_, &AST::Value(_)) => {
                        Ok(Lazy::Defer(node,
                                       left,
                                       from_raw(h)
                                           .arena
                                           .cont(Cont::Verb(verb.clone(), right, 1, cont))))
                    }
                    (x, y) => {
                        Ok(Lazy::Defer(node,
                                       x,
                                       from_raw(h)
                                           .arena
                                           .cont(Cont::Verb(verb.clone(), y, 0, cont))))
                    }
                }
            }
            &AST::NameInt(name) => {
                let l = from_raw(h).lookup(node, name, &from_raw(h).env);
                match l {
                    Ok((v, f)) => from_raw(h).run_cont(f, v, cont),
                    Err(x) => Err(x),
                }
            }
            &AST::Lambda(_, x, y) => {
                from_raw(h).run_cont(node,
                                     from_raw(h).arena.ast(AST::Lambda(Some(node), x, y)),
                                     cont)
            }
            x => from_raw(h).run_cont(node, x, cont),
        }
    }

    fn lookup(&'a mut self,
              node: &'a otree::Node<'a>,
              name: u16,
              env: &'a env::Environment<'a>)
              -> Result<(&'a AST<'a>, &'a otree::Node<'a>), Error> {
        let h = into_raw(self);
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

    pub fn evaluate_fun(&'a mut self,
                        node: &'a otree::Node<'a>,
                        fun: &'a AST<'a>,
                        args: &'a AST<'a>,
                        cont: &'a Cont<'a>)
                        -> Result<Lazy<'a>, Error> {
        // println!("Eval Fun: {:?}", fun);
        let h = into_raw(self);
        match fun {
            &AST::Lambda(closure, names, body) => {
                let mut rev = ast::rev_dict(args, &from_raw(h).arena);
                // println!("Args Fun: {:?} orig: {:?}, names: {:?}", rev, args, names);
                from_raw(h).run_cont(if closure == None {
                                         node
                                     } else {
                                         closure.unwrap()
                                     },
                                     body,
                                     from_raw(h).arena.cont(Cont::Func(names, rev, body, cont)))
            }
            &AST::NameInt(s) => {
                // println!("{:?}", s);
                let v = from_raw(h).lookup(node, s, &from_raw(h).env);
                match v {
                    Ok((c, f)) => {
                        match c {
                            &AST::NameInt(n) if n < from_raw(h).arena.builtins => {
                                handle_context(f,
                                               from_raw(h),
                                               internals(from_raw(h),
                                                         n,
                                                         args,
                                                         &from_raw(h).arena),
                                               cont)
                            }
                            _ => from_raw(h).evaluate_fun(f, c, args, cont),
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

    pub fn evaluate_expr(&'a mut self,
                         node: &'a otree::Node<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<Lazy<'a>, Error> {
        // println!("Eval Expr: {:?}", exprs);
        let h = into_raw(self);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(Lazy::Defer(node,
                               car,
                               from_raw(h)
                                   .arena
                                   .cont(Cont::Expressions(cdr, cont))))
            }
            &AST::Nil => from_raw(h).run_cont(node, exprs, cont),
            x => Ok(Lazy::Defer(node, x, cont)),
        }
    }

    pub fn evaluate_list(&'a mut self,
                         node: &'a otree::Node<'a>,
                         acc: &'a AST<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<Lazy<'a>, Error> {
        // println!("Eval List: {:?}", exprs);
        let h = into_raw(self);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(Lazy::Defer(node,
                               car,
                               from_raw(h)
                                   .arena
                                   .cont(Cont::List(acc, cdr, cont))))
            }
            &AST::Nil => from_raw(h).run_cont(node, acc, cont),
            x => Ok(Lazy::Defer(node, x, cont)),
        }
    }

    pub fn evaluate_dict(&'a mut self,
                         node: &'a otree::Node<'a>,
                         acc: &'a AST<'a>,
                         exprs: &'a AST<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<Lazy<'a>, Error> {
        // println!("Eval Dict: {:?}", exprs);
        let h = into_raw(self);
        match exprs {
            &AST::Cons(car, cdr) => {
                Ok(Lazy::Defer(node,
                               car,
                               from_raw(h)
                                   .arena
                                   .cont(Cont::Dict(acc, cdr, cont))))
            }
            &AST::Nil => from_raw(h).run_cont(node, acc, cont),
            x => Ok(Lazy::Defer(node, x, cont)),
        }
    }

    pub fn emit_return(&'a mut self, val: &'a AST<'a>, cont: &'a Cont<'a>) -> Result<Lazy<'a>, Error> {
        // println!("Emit: {:?}", val);
        let h = into_raw(self);
        match val {
            &AST::Dict(x) => {
                let mut dict = ast::rev_dict(x, &from_raw(h).arena);
                Ok(Lazy::Return(from_raw(h).arena.ast(AST::Dict(dict))))
            }
            &AST::Cons(x, y) => {
                Ok(Lazy::Return(from_raw(h)
                    .arena
                    .ast(AST::Dict(ast::rev_dict(from_raw(h).arena.ast(AST::Cons(x, y)), &from_raw(h).arena)))))
            }
            x => Ok(Lazy::Return(x)),
        }
    }

    pub fn run_cont(&'a mut self,
                    node: &'a otree::Node<'a>,
                    val: &'a AST<'a>,
                    con: &'a Cont<'a>)
                    -> Result<Lazy<'a>, Error> {
        // println!("run_cont: val: {:?} #### cont: {:?}\n", val, cont);
        let h = into_raw(self);
        match con {
            &Cont::Intercore(ref m, cc) => {
                // println!("Intercore: val: {:?} #### cont: {:?}\n", val, cc);
                from_raw(h).edge = m.clone();
                Ok(Lazy::Continuation(node, val, cc))
            }
            &Cont::Yield(cc) => Ok(Lazy::Continuation(node, val, cc)),
            &Cont::Call(callee, cont) => {
                let c;
                match val {
                    &AST::Dict(v) => c = from_raw(h).evaluate_fun(node, callee, v, cont),
                    x => c = from_raw(h).evaluate_fun(node, callee, x, cont),
                };
                c
            }
            &Cont::Func(names, args, body, cont) => {
                // println!("names={:?} args={:?}", names, args);
                let f = from_raw(h).env.new_child(node);
                let mut partial = from_raw(h).arena.nil(); // empty list of unfilled/empty arguments
                for (k, v) in names.into_iter().zip(args.into_iter()) {
                    match v {
                        &AST::Any => partial = from_raw(h).arena.ast(AST::Cons(k, partial)),
                        _ => {
                            from_raw(h).env.define(ast::extract_name(k), v);
                        }
                    };
                }

                match partial {
                    &AST::Nil => from_raw(h).evaluate_expr(f, val, cont),
                    _ => {
                        Ok(Lazy::Defer(f,
                                       from_raw(h).arena.ast(AST::Lambda(Some(f), partial, body)),
                                       cont))
                    }
                }
            }
            &Cont::Cond(if_expr, else_expr, cont) => {
                match val {
                    &AST::Value(Value::Number(0)) => Ok(Lazy::Defer(node, else_expr, cont)),
                    &AST::Value(_) => Ok(Lazy::Defer(node, if_expr, cont)),
                    x => {
                        Ok(Lazy::Defer(node,
                                       x,
                                       from_raw(h).arena.cont(Cont::Cond(if_expr, else_expr, cont))))
                    }
                }
            }
            &Cont::Assign(name, cont) => {
                match name {
                    &AST::NameInt(s) => {
                        // println!("Assign: {:?}:{:?}", s, val);
                        try!(from_raw(h).env.define(s, val));
                        from_raw(h).evaluate_expr(node, val, cont)
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
                    &AST::Cons(x, y) => {
                        new_acc = from_raw(h).arena.ast(AST::Cons(from_raw(h).arena.ast(AST::Dict(val)), acc))
                    }
                    _ => new_acc = from_raw(h).arena.ast(AST::Cons(val, acc)),
                };
                match rest {
                    &AST::Cons(head, tail) => {
                        from_raw(h).evaluate_dict(node,
                                                  from_raw(h).arena.nil(),
                                                  head,
                                                  from_raw(h)
                                                      .arena
                                                      .cont(Cont::Dict(new_acc, tail, cont)))
                    }
                    &AST::Value(Value::Number(s)) => {
                        from_raw(h).run_cont(node, from_raw(h).arena.ast(AST::Cons(rest, new_acc)), cont)
                    }
                    &AST::Nil => from_raw(h).run_cont(node, new_acc, cont),
                    &AST::NameInt(name) => {
                        match from_raw(h).lookup(node, name, &from_raw(h).env) {
                            Ok((v, f)) => from_raw(h).run_cont(f, from_raw(h).arena.ast(AST::Cons(v, new_acc)), cont),
                            Err(x) => Err(x),
                        }
                    }
                    x => {
                        Ok(Lazy::Defer(node,
                                       x,
                                       from_raw(h)
                                           .arena
                                           .cont(Cont::Dict(new_acc, from_raw(h).arena.nil(), cont))))
                    }
                }
            }
            &Cont::Verb(ref verb, right, swap, cont) => {
                // println!("Cont Verb: {:?}", val);
                match (right, val) {
                    (&AST::Value(_), &AST::Value(_)) => {
                        match swap {
                            0 => {
                                let a = verb::eval(verb.clone(), right, val).unwrap();
                                from_raw(h).run_cont(node, from_raw(h).arena.ast(a), cont)
                            }
                            _ => {
                                let a = verb::eval(verb.clone(), val, right).unwrap();
                                from_raw(h).run_cont(node, from_raw(h).arena.ast(a), cont)
                            }
                        }
                    }
                    (x, &AST::Nil) => Ok(Lazy::Defer(node, x, cont)),
                    (x, y) => {
                        Ok(Lazy::Defer(node,
                                       x,
                                       from_raw(h)
                                           .arena
                                           .cont(Cont::Verb(verb.clone(), y, 0, cont))))
                    }
                }
            }
            &Cont::Expressions(rest, cont) => {
                if rest.is_cons() || !rest.is_empty() {
                    from_raw(h).evaluate_expr(node, rest, cont)
                } else {
                    from_raw(h).run_cont(node, val, cont)
                }
            }
            x => {
                let o = from_raw(h).emit_return(val, con);
                // println!("Return: {:?}", o);
                o
            }
        }
    }
}
