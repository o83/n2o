
use commands::ast::{self, Error, AST, Arena, Value};
use streams::otree;
use streams::interpreter::{Interpreter, Lazy, Cont};
use intercore::message::{Pub, Sub, Spawn, Message};
use intercore::bus::Ctx;
use reactors::task::Context;
use queues::publisher::Publisher;

// The InterCore messages are being sent fron client in Interpreter

pub fn internals<'a>(f_id: u16, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    match f_id {
        0 => Context::Intercore(Message::Print(format!("args {:?}", args))),
        1 => Context::Intercore(Message::Pub(Pub { from: 0, to: 0, task_id: 0, name: "".to_string(), cap: 8 })),
        2 => Context::Intercore(Message::Sub(Sub { from: 0, to: 0, task_id: 0, pub_id: 0})),
        3 => snd_(args, arena), // Context::Node
        4 => rcv_(args, arena), // Context::Node + Context::Nil
        5 => Context::Intercore(Message::Spawn(Spawn { from: 0, to: 0, txt: "".to_string() })),
        6 => Context::Intercore(Message::Select("".to_string(),80)),
        _ => panic!("unknown internal func"),
    }
}

pub fn handle_context<'a>(f: &'a otree::Node<'a>,
                          i: &'a Interpreter<'a>,
                          x: Context<'a>,
                          cont: &'a Cont<'a>)
                          -> Result<Lazy<'a>, Error> {
    match x {
        Context::Nil => i.run_cont(f, i.arena.yield_(), i.arena.cont(Cont::Yield(cont))),
        Context::Node(&AST::Yield) => i.run_cont(f, i.arena.yield_(), i.arena.cont(Cont::Yield(cont))),
        Context::Intercore(message) => i.run_cont(f, i.arena.yield_(), i.arena.cont(Cont::Intercore(cont))),
        Context::NodeAck(x) => i.run_cont(f, i.arena.ast(x), cont),
        _ => panic!("TODO"),
    }
}

pub fn pub_<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("publishers {:?}", args);
    let cap = match args {
        &AST::Value(Value::Number(n)) => n,
        _ => 1024,
    } as usize;
    Context::NodeAck(AST::Value(Value::Number(13)))
}

pub fn sub_<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("subscribers {:?}", args);
    Context::NodeAck(AST::Value(Value::Number(14)))
}

pub fn snd_<'a>(args: &AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
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

pub fn rcv_<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    let mut res = 0u64;
    println!("RECV {:?}", args);
    match args {
        &AST::Value(Value::Number(n)) => {}
        _ => panic!("oops!"),
    }
    Context::NodeAck(AST::Value(Value::Number(res as i64)))
}
