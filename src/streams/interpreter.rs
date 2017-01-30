
// O-CPS INTERPRETER by 5HT et all

use streams::{verb, env, otree};
use commands::ast::{self, Error, AST, ASTNode, Verb, Adverb, Arena, Value, ASTAcc, ASTIter};
use intercore::bus::Memory;
use intercore::client::{eval_context, internals};
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
    Expressions(&'a ASTNode<'a>, Option<ASTIter<'a>>, &'a Cont<'a>),
    Assign(&'a ASTNode<'a>, &'a Cont<'a>),
    Cond(&'a ASTNode<'a>, &'a ASTNode<'a>, &'a Cont<'a>),
    Func(&'a ASTNode<'a>, &'a ASTNode<'a>, &'a ASTNode<'a>, &'a Cont<'a>),
    List(&'a ASTNode<'a>, ASTIter<'a>, &'a Cont<'a>),
    Dict(ASTAcc<'a>, ASTIter<'a>, &'a Cont<'a>),
    DictComplete(ASTAcc<'a>, ASTIter<'a>, usize, &'a Cont<'a>),
    Call(&'a ASTNode<'a>, &'a Cont<'a>),
    Verb(Verb, &'a ASTNode<'a>, u8, &'a Cont<'a>),
    Adverb(Adverb, &'a ASTNode<'a>, &'a Cont<'a>),
    Return,
    Intercore(Message, &'a Cont<'a>),
    Yield(&'a Cont<'a>),
}

#[derive(Clone, Debug)]
pub enum Lazy<'a> {
    Defer(otree::NodeId, &'a ASTNode<'a>, &'a Cont<'a>),
    Continuation(otree::NodeId, &'a ASTNode<'a>, &'a Cont<'a>),
    Return(&'a ASTNode<'a>),
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
        let print = s1.arena.intern_ast("print".to_string());
        let publ = s1.arena.intern_ast("pub".to_string());
        let subs = s1.arena.intern_ast("sub".to_string());
        let snd = s1.arena.intern_ast("snd".to_string());
        let rcv = s1.arena.intern_ast("rcv".to_string());
        let spawn = s1.arena.intern_ast("spawn".to_string());
        s1.env.define(ast::extract_name(&print), print);
        s1.env.define(ast::extract_name(&publ), publ);
        s1.env.define(ast::extract_name(&subs), subs);
        s1.env.define(ast::extract_name(&snd), snd);
        s1.env.define(ast::extract_name(&rcv), rcv);
        s1.env.define(ast::extract_name(&spawn), spawn);
        let x = unsafe { &mut *s1.arena.asts.get() };
        s2.arena.builtins = x.len() as u16;
    }

    pub fn parse(&'a mut self, s: &String) -> &'a ASTNode<'a> {
        ast::parse(&self.arena, s)
    }

    pub fn load(&'a mut self, ast: &'a ASTNode<'a>) {
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
               ast: &'a ASTNode<'a>,
               intercore: Context<'a>,
               sched: Option<&'a Scheduler<'a>>)
               -> Result<&'a ASTNode<'a>, Error> {
        let h = into_raw(self);
        let mut tick;
        let mut ret = from_raw(h).arena.nil();


        match from_raw(h).registers {
            Lazy::Start => {
                tick = try!(from_raw(h).evaluate_expr(from_raw(h).env.last(),
                                                      ast,
                                                      from_raw(h).arena.cont(Cont::Return)))
            }
            _ => tick = from_raw(h).registers.clone(),
        }

        match intercore.clone() {
            Context::NodeAck(_, value) => {
                ret = from_raw(h).arena.ast(ASTNode::AST(AST::Value(Value::Number(value as i64))));
            }
            _ => (),
        }

        loop {
            let mut counter = from_raw(h).counter;
            match tick {
                Lazy::Defer(node, ast_, cont) => {
                    if counter % PREEMPTION == 0 {
                        from_raw(h).registers = tick;
                        from_raw(h).counter = counter + 1;
                        return Ok(from_raw(h).arena.ast(ASTNode::AST(AST::Yield(Context::Nil))));
                    } else {
                        tick = try!({
                            from_raw(h).edge = Message::Nop;
                            let a = match ast_ {
                                &ASTNode::AST(AST::Yield(..)) => ret,
                                x => x,
                            };
                            from_raw(h).counter = counter + 1;
                            from_raw(h).handle_defer(node, a, cont)
                        })
                    }
                }
                Lazy::Start => break,
                Lazy::Continuation(node, ast, cont) => {
                    let ast_ = from_raw(h).arena.ast(ASTNode::AST(AST::Yield(Context::Intercore(&from_raw(h).edge))));
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
            ast: format!("{:?}", self.arena.nil()),
        })
    }

    pub fn gc(&self) -> usize {
        self.env.clean() + self.arena.clean()
    }

    fn handle_defer(&'a mut self,
                    node: otree::NodeId,
                    a: &'a ASTNode<'a>,
                    cont: &'a Cont<'a>)
                    -> Result<Lazy<'a>, Error> {
        let h = into_raw(self);
        match a {
            &ASTNode::AST(AST::Assign(name, body)) => {
                Ok(Lazy::Defer(node, body, from_raw(h).arena.cont(Cont::Assign(name, cont))))
            }
            &ASTNode::AST(AST::Cond(val, left, right)) => {
                Ok(Lazy::Defer(node,
                               val,
                               from_raw(h).arena.cont(Cont::Cond(left, right, cont))))
            }
            &ASTNode::AST(AST::List(x)) => from_raw(h).defer_dict(node, x, cont), // so far list are treated the same as dicts
            &ASTNode::AST(AST::Dict(x)) => from_raw(h).defer_dict(node, x, cont),
            &ASTNode::AST(AST::Call(c, a)) => {
                // println!("Defer call: {:?} {:?}", c, a);
                Ok(Lazy::Defer(node, a, from_raw(h).arena.cont(Cont::Call(c, cont))))
            }
            &ASTNode::AST(AST::Verb(ref verb, left, right)) => {
                // println!("Defer Verb: {:?} {:?}", left, right);
                match (left, right) {
                    (&ASTNode::AST(AST::Value(_)), _) => {
                        Ok(Lazy::Defer(node,
                                       right,
                                       from_raw(h).arena.cont(Cont::Verb(verb.clone(), left, 0, cont))))
                    }
                    (_, &ASTNode::AST(AST::Value(_))) => {
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
            &ASTNode::AST(AST::NameInt(name)) => {
                let l = from_raw(h).lookup(node, name, &from_raw(h).env);
                match l {
                    Ok((v, f)) => from_raw(h).run_cont(f, v, cont),
                    Err(x) => Err(x),
                }
            }
            &ASTNode::AST(AST::Lambda(_, x, y)) => {
                from_raw(h).run_cont(node,
                                     from_raw(h).arena.ast(ASTNode::AST(AST::Lambda(Some(node), x, y))),
                                     cont)
            }
            x => from_raw(h).run_cont(node, x, cont),
        }
    }

    fn lookup(&'a mut self,
              node: otree::NodeId,
              name: u16,
              env: &'a env::Environment<'a>)
              -> Result<(&'a ASTNode<'a>, otree::NodeId), Error> {
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
                        node: otree::NodeId,
                        fun: &'a ASTNode<'a>,
                        args: &'a ASTNode<'a>,
                        cont: &'a Cont<'a>)
                        -> Result<Lazy<'a>, Error> {
        // println!("Eval Fun: {:?}", fun);
        let h = into_raw(self);
        match fun {
            &ASTNode::AST(AST::Lambda(closure, names, body)) => {
                self.run_cont(if closure == None {
                                  node
                              } else {
                                  closure.unwrap()
                              },
                              body,
                              from_raw(h).arena.cont(Cont::Func(names, args, body, cont)))
            }
            &ASTNode::AST(AST::NameInt(s)) => {
                // println!("{:?}", s);
                let v = from_raw(h).lookup(node, s, &from_raw(h).env);
                match v {
                    Ok((c, f)) => {
                        match c {
                            &ASTNode::AST(AST::NameInt(n)) if n < from_raw(h).arena.builtins => {
                                eval_context(f,
                                             from_raw(h),
                                             internals(from_raw(h), n, args, &from_raw(h).arena),
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

    pub fn evaluate_expr(&'a self,
                         node: otree::NodeId,
                         exprs: &'a ASTNode<'a>,
                         cont: &'a Cont<'a>)
                         -> Result<Lazy<'a>, Error> {

        Ok(Lazy::Defer(node,
                       self.arena.any(),
                       self.arena.cont(Cont::Expressions(exprs, None, cont))))
    }

    pub fn defer_dict(&'a self,
                      node: otree::NodeId,
                      dict: &'a ASTNode<'a>,
                      cont: &'a Cont<'a>)
                      -> Result<Lazy<'a>, Error> {

        // println!("defer Dict: {:?}", dict);
        match dict {
            &ASTNode::VecAST(ref v) => {
                // create new accumulator and start calculating dict values
                Ok(Lazy::Defer(node,
                               dict,
                               self.arena.cont(Cont::Dict(ASTAcc::new(), v.as_slice().iter(), cont))))
            }
            x => Ok(Lazy::Defer(node, x, cont)),
        }
    }

    pub fn run_cont(&'a mut self,
                    node: otree::NodeId,
                    val: &'a ASTNode<'a>,
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
                    &ASTNode::AST(AST::Dict(v)) => c = from_raw(h).evaluate_fun(node, callee, v, cont),
                    x => c = from_raw(h).evaluate_fun(node, callee, x, cont),
                };
                c
            }
            &Cont::Func(ref names, ref args, body, cont) => {
                // println!("cont_func names={:?} args={:?}", names, args);
                let f = from_raw(h).env.new_child(node);
                let mut partial: Vec<ASTNode> = Vec::new(); // vector of unfilled/empty names

                for (k, v) in names.into_iter().zip(args.into_iter()) {
                    // println!("cont_func name={:?} arg={:?}", k, v);
                    match v {
                        &ASTNode::AST(AST::Any) => partial.push(k.clone()),
                        _ => {
                            from_raw(h).env.define(ast::extract_name(k), v);
                        }
                    };
                }
                if partial.len() == 0 {
                    // println!("run_cont func: val={:?}", val);
                    from_raw(h).evaluate_expr(f, val, cont)
                } else {
                    Ok(Lazy::Defer(f,
                                   from_raw(h)
                                       .arena
                                       .ast(ASTNode::AST(AST::Lambda(Some(f),
                                                                     from_raw(h)
                                                                         .arena
                                                                         .ast(ASTNode::VecAST(partial)),
                                                                     body))),
                                   cont))
                }
            }
            &Cont::Cond(if_expr, else_expr, cont) => {
                match val {
                    &ASTNode::AST(AST::Value(Value::Number(0))) => Ok(Lazy::Defer(node, else_expr, cont)),
                    &ASTNode::AST(AST::Value(_)) => Ok(Lazy::Defer(node, if_expr, cont)),
                    x => {
                        Ok(Lazy::Defer(node,
                                       x,
                                       from_raw(h).arena.cont(Cont::Cond(if_expr, else_expr, cont))))
                    }
                }
            }
            &Cont::Assign(name, cont) => {
                match name {
                    &ASTNode::AST(AST::NameInt(s)) => {
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
            &Cont::DictComplete(ref acc, ref rest, idx, cont) => {
                // println!("run_cont dict_complete: acc={} idx={} val={} #### cont: {:?}\n", acc, idx, val, cont);
                acc.set(idx, val.clone()); // TODO Get rid of clone!!!

                Ok(Lazy::Defer(node,
                               val,
                               from_raw(h).arena.cont(Cont::Dict(acc.clone(), rest.clone(), cont))))
            }
            &Cont::Dict(ref acc, ref rest, cont) => {
                // println!("run_cont dict: acc={} #### cont: {:?}\n", acc, cont);

                match val {
                    &ASTNode::AST(_) => {
                        acc.push(val);
                    }
                    _ => {}
                }

                let mut r = rest.clone();
                match r.next() {
                    Some(v) => {
                        // println!("run_cont dict next: x={} #### cont: {:?}\n", v, cont);
                        match v {
                            &ASTNode::AST(AST::Dict(ref astv)) |
                            &ASTNode::AST(AST::List(ref astv)) // so far list are treated the same as dicts
                                => {
                                //println!("run_cont dict vec: val={} #### cont: {:?}\n", val, cont);
                                let idx = acc.push( from_raw(h).arena.any()); // allocate empty place for vector result
                                 from_raw(h).defer_dict(node, astv,
                                                 from_raw(h).arena.cont(Cont::DictComplete(acc.clone(), r, idx, cont)))
                            }
                            x => {
                                //println!("run_cont dict defer: x={} #### cont: {:?}\n", x, cont);
                                Ok(Lazy::Defer(node,
                                               x,
                                                from_raw(h).arena.cont(Cont::Dict(acc.clone(), r, cont))))
                            }
                        }
                    }
                    _ => {
                        // entire vector calculated, time to move calculation on
                        from_raw(h).run_cont(node,
                                             from_raw(h).arena.ast(ASTNode::VecAST(acc.disown())),
                                             cont)
                    }
                }
            }
            &Cont::Verb(ref verb, right, swap, cont) => {
                // println!("Cont Verb: {:?}", val);
                match (right, val) {
                    (&ASTNode::AST(AST::Value(_)), &ASTNode::AST(AST::Value(_))) => {
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
                    (x, &ASTNode::AST(AST::Value(Value::Nil))) => Ok(Lazy::Defer(node, x, cont)),
                    (x, y) => {
                        Ok(Lazy::Defer(node,
                                       x,
                                       from_raw(h)
                                           .arena
                                           .cont(Cont::Verb(verb.clone(), y, 0, cont))))
                    }
                }
            }

            &Cont::Expressions(ast, ref rest, cont) => {
                // println!("run_cont expr: ast={:?} #### cont: {:?}\n", ast, cont);

                match ast {
                    &ASTNode::AST(ref x) => {
                        // println!("run_cont expr ast: #### cont: {:?}\n", cont);
                        Ok(Lazy::Defer(node, ast, cont))
                    }
                    &ASTNode::VecAST(ref v) => {
                        match *rest {
                            Some(ref x) => {
                                let mut e = x.clone();
                                match e.next() {
                                    Some(n) => {
                                        // println!("run_cont expr vec n={:?} #### cont: {:?}\n", n, cont);
                                        Ok(Lazy::Defer(node,
                                                       n,
                                                       from_raw(h).arena.cont(Cont::Expressions(ast, Some(e), cont))))
                                    }
                                    _ => from_raw(h).run_cont(node, val, cont),
                                }
                            }
                            _ => {
                                // println!("run_cont expr vec end: #### cont: {:?}\n", cont);
                                Ok(Lazy::Defer(node,
                                               ast,
                                               from_raw(h).arena.cont(Cont::Expressions(ast, Some(v.iter()), cont))))
                            }
                        }
                    }
                }
            }
            _ => Ok(Lazy::Return(val)),
        }
    }
}
