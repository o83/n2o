
use commands::ast::{Error, AST, Atom, Arena, Value};
use streams::otree;
use streams::interpreter::{Interpreter, Lazy, Cont};
use intercore::message::{Pub, Sub, Message, Spawn};
use reactors::task::Context;
use handle::{into_raw, from_raw};

// The InterCore messages + Buildins are being handled in Interpreter

pub fn internals<'a>(i: &'a mut Interpreter<'a>, f_id: u16, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    match f_id {
        0 => print(i, args, arena),
        1 => publisher(i, args, arena),
        2 => subscriber(i, args, arena),
        3 => send(i, args, arena),
        4 => receive(i, args, arena),
        5 => spawn(i, args, arena),
        6 => Context::Nil,
        _ => panic!("unknown internal func"),
    }
}

pub fn eval_context<'a>(f: otree::NodeId,
                        i: &'a mut Interpreter<'a>,
                        x: Context<'a>,
                        cont: &'a Cont<'a>)
                        -> Result<Lazy<'a>, Error> {

    let h = into_raw(i);
    match x {
        Context::Nil => {
            from_raw(h).run_cont(f,
                                 from_raw(h).arena.ast(AST::Atom(Atom::Yield(Context::Nil))),
                                 from_raw(h).arena.cont(Cont::Yield(cont)))
        }
        Context::Intercore(message) => {
            from_raw(h).run_cont(f,
                                 from_raw(h).arena.ast(AST::Atom(Atom::Yield(Context::Intercore(&from_raw(h).edge)))),
                                 from_raw(h).arena.cont(Cont::Intercore(message.clone(), cont)))
        }
        Context::Node(ref ast) => from_raw(h).run_cont(f, ast, cont),

        _ => panic!("TODO"),
    }
}


pub fn print<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("Print Args: {}", args);
    Context::Node(args)
}

pub fn spawn<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("Spawn Args: {:?}", args);
    let (core, txt) = (0, "a:1".to_string());

    let (core, txt) = match args {
        &AST::Vector(ref v) if v.len() == 2 => {
            match (&v[0], &v[1]) {
                (&AST::Atom(Atom::Value(Value::Number(c))), &AST::Atom(Atom::Value(Value::SequenceInt(n)))) => {
                    (c, "test".to_string())
                }
                _ => (0, "".to_string()),
            }
        }
        _ => (0, "".to_string()),
    };

    i.edge = Message::Spawn(Spawn {
        from: 0,
        to: core as usize,
        txt: txt,
    });
    Context::Intercore(&i.edge)
}

pub fn publisher<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("Pub Args: {:?}", args);

    let (core, cap) = match args {
        &AST::Vector(ref v) if v.len() == 2 => {
            match (&v[0], &v[1]) {
                (&AST::Atom(Atom::Value(Value::Number(core))), &AST::Atom(Atom::Value(Value::Number(cap)))) => {
                    (core as usize, cap as usize)
                }
                _ => panic!("oops!"),
            }
        }
        _ => panic!("oops!"),
    };

    i.edge = Message::Pub(Pub {
        from: 0,
        task_id: 0,
        to: core,
        name: "".to_string(),
        cap: cap,
    });
    Context::Intercore(&i.edge)
}

pub fn subscriber<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("Sub Args: {:?}", args);

    let (core, pub_id) = match args {
        &AST::Vector(ref v) if v.len() == 2 => {
            match (&v[0], &v[1]) {
                (&AST::Atom(Atom::Value(Value::Number(core))), &AST::Atom(Atom::Value(Value::Number(pub_id)))) => {
                    (core as usize, pub_id as usize)
                }
                _ => panic!("oops!"),
            }
        }
        _ => panic!("oops!"),
    };

    i.edge = Message::Sub(Sub {
        from: i.task_id,
        task_id: i.task_id,
        to: core,
        pub_id: pub_id,
    });
    Context::Intercore(&i.edge)
}

pub fn send<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("Send Args: {:?}", args);

    let (val, pub_id) = match args {
        &AST::Vector(ref v) if v.len() == 2 => {
            match (&v[0], &v[1]) {
                (&AST::Atom(Atom::Value(Value::Number(pub_id))), &AST::Atom(Atom::Value(Value::Number(val)))) => {
                    (val, pub_id)
                }
                _ => panic!("oops!"),
            }
        }
        _ => panic!("oops!"),
    };

    let mut p = i.queues.publishers().get(pub_id as usize).expect(&format!("Wrong publisher id: {}", pub_id));
    if let Some(slot) = p.next() {
        *slot = val;
        p.commit();
    }
    Context::Node(arena.nil())
}

pub fn receive<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("Receive Args: {:?}", args);
    match args {
        &AST::Atom(Atom::Value(Value::Number(sub_id))) => {
            let mut s = i.queues.subscribers().get(sub_id as usize).expect(&format!("Wrong subscriber id: {}", sub_id));
            if let Some(slot) = s.recv() {
                let res = *slot;
                s.commit();
                return Context::Node(arena.ast(AST::Atom(Atom::Value(Value::Number(res as i64)))));
            }
        }
        _ => panic!("oops!"),
    }

    Context::Node(arena.nil())
}
