
use commands::ast::{Error, AST, Arena, Value};
use streams::otree;
use streams::interpreter::{Interpreter, Lazy, Cont};
use intercore::message::{Pub, Sub, Message, Spawn};
use reactors::task::Context;
use handle::{into_raw, from_raw};

// The InterCore messages are being sent fron client in Interpreter

pub fn internals<'a>(i: &'a mut Interpreter<'a>,
                     f_id: u16,
                     args: &'a AST<'a>,
                     arena: &'a Arena<'a>)
                     -> Context<'a> {
    match f_id {
        0 => { println!("print: {}", args); Context::Node(args) },
        1 => create_publisher(i, args, arena),
        2 => create_subscriber(i, args, arena),
        3 => snd(i, args, arena),
        4 => rcv(i, args, arena),
        5 => spawn(i, args, arena),
        6 => Context::Nil,
        _ => panic!("unknown internal func"),
    }
}

pub fn handle_context<'a>(f: &'a otree::Node<'a>,
                          i: &'a mut Interpreter<'a>,
                          x: Context<'a>,
                          cont: &'a Cont<'a>)
                          -> Result<Lazy<'a>, Error> {

    let h = into_raw(i);
    match x {
        Context::Nil => {
            from_raw(h).run_cont(f,
                                 from_raw(h).arena.ast(AST::Yield(Context::Nil)),
                                 from_raw(h).arena.cont(Cont::Yield(cont)))
        }
        Context::Intercore(message) => {
            from_raw(h).run_cont(f,
                                 from_raw(h).arena.ast(AST::Yield(Context::Intercore(&from_raw(h).edge))),
                                 from_raw(h).arena.cont(Cont::Intercore(message.clone(), cont)))
        }
        Context::Node(ref ast) => from_raw(h).run_cont(f, ast, cont),

        _ => panic!("TODO"),
    }
}

pub fn spawn<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    let (core, txt) = match args {
        &AST::Cons(&AST::Value(Value::Number(c)), &AST::Cons(&AST::Value(Value::SequenceInt(n)), t)) => {
            (c, "test".to_string())
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

pub fn create_publisher<'a>(i: &'a mut Interpreter<'a>,
                            args: &'a AST<'a>,
                            arena: &'a Arena<'a>)
                            -> Context<'a> {
    let (core, cap) = match args {
        &AST::Cons(&AST::Value(Value::Number(cap)), tail) => {
            match tail {
                &AST::Cons(&AST::Value(Value::Number(core)), tail) => (core as usize, cap as usize),
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

pub fn create_subscriber<'a>(i: &'a mut Interpreter<'a>,
                             args: &'a AST<'a>,
                             arena: &'a Arena<'a>)
                             -> Context<'a> {
    println!("print: {:?}", args);
    let (core, pub_id) = match args {
        &AST::Cons(&AST::Value(Value::Number(pub_id)), tail) => {
            match tail {
                &AST::Cons(&AST::Value(Value::Number(core)), tail) => (core as usize, pub_id as usize),
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

pub fn snd<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("SND {:?}", args);
    let (val, pub_id) = match args {
        &AST::Cons(&AST::Value(Value::Number(val)), tail) => {
            match tail {
                &AST::Cons(&AST::Value(Value::Number(pub_id)), tail) => (val, pub_id),
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
    // else how can i signal NotReady?
    Context::Node(arena.nil())
}

pub fn rcv<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("RECV {:?}", args);
    match args {
        &AST::Value(Value::Number(sub_id)) => {
            let mut s = i.queues.subscribers().get(sub_id as usize).expect(&format!("Wrong subscriber id: {}", sub_id));
            if let Some(slot) = s.recv() {
                let res = *slot;
                s.commit();
                println!("subs recv {:?}", res);
                return Context::Node(arena.ast(AST::Value(Value::Number(res as i64))));
            }
        }
        _ => panic!("oops!"),
    }
    Context::Node(arena.nil())
}
