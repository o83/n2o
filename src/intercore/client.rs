
use commands::ast::{self, Error, AST, Arena, Value};
use streams::otree;
use streams::interpreter::{Interpreter, Lazy, Cont};
use intercore::message::{Pub, Sub, Spawn, Message};
use intercore::bus::Ctx;
use reactors::task::Context;
use queues::publisher::Publisher;
use handle::{self, into_raw, from_raw};

// The InterCore messages are being sent fron client in Interpreter

pub fn internals<'a>(i: &'a mut Interpreter<'a>,f_id: u16,  args: &'a AST<'a>, arena: &'a Arena<'a>, task_id: usize) -> Context<'a> {
    match f_id {
        0 => Context::Nil,
        1 => create_publisher(i, args, arena, task_id),
        2 => create_subscriber(i, args, arena, task_id),
        3 => snd(args, arena),
        4 => rcv(args, arena),
        5 => Context::Nil,
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
                       from_raw(h).arena.ast(AST::Yield(Context::Intercore(&from_raw(h).ctx))),
                       from_raw(h).arena.cont(Cont::Intercore(message.clone(), cont)))
        }
        Context::NodeAck(task_id, x) => from_raw(h).run_cont(f, from_raw(h).arena.ast(AST::Value(Value::Number(x as i64))), cont),
        _ => panic!("TODO"),
    }
}

pub fn create_publisher<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>, task_id: usize) -> Context<'a> {
    println!("publishers {:?}", args);
    let cap = match args {
        &AST::Value(Value::Number(n)) => n,
        _ => 1024,
    } as usize;
    i.ctx = Message::Pub(Pub {
        from: task_id,
        task_id: task_id,
        to: 0,
        name: "".to_string(),
        cap: cap,
    });
    Context::Intercore(&i.ctx)
}

pub fn create_subscriber<'a>(i: &'a mut Interpreter<'a>, args: &'a AST<'a>, arena: &'a Arena<'a>, task_id: usize) -> Context<'a> {
    let p = match args {
        &AST::Value(Value::Number(n)) => n,
        _ => 0,
    } as usize;
    i.ctx = Message::Sub(Sub {
        from: task_id,
        task_id: task_id,
        to: 0,
        pub_id: p,
    });
    Context::Intercore(&i.ctx)
}

pub fn snd<'a>(args: &AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    // println!("SND {:?}", args);
    match args {
        &AST::Cons(&AST::Value(Value::Number(val)), tail) => {
            match tail {
                &AST::Cons(&AST::Value(Value::Number(cursor_id)), tail) => {}
                _ => panic!("oops!"),
            }
        }
        _ => panic!("oops!"),
    }
    Context::Nil
}

pub fn rcv<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    let mut res: usize = 0;
    println!("RECV {:?}", args);
    match args {
        &AST::Value(Value::Number(n)) => {}
        _ => panic!("oops!"),
    }
    Context::NodeAck(0, res)
}
