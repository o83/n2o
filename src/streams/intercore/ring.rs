use super::ctx::Ctx;
use commands::ast::{AST, Arena};
use commands::ast::Value;
use queues::publisher::Publisher;
use reactors::task::Context;

pub fn pub_<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("publishers {:?}", args);
    let cap = match args {
        &AST::Value(Value::Number(n)) => n,
        _ => 1024,
    } as usize;
    Context::Node(arena.ast(AST::Value(Value::Number(13))))
}

pub fn sub_<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("subscribers {:?}", args);
    Context::Node(arena.ast(AST::Value(Value::Number(14))))
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
    Context::Node(arena.nil())
}

pub fn rcv_<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    let mut res = 0u64;
    println!("RECV {:?}", args);
    match args {
        &AST::Value(Value::Number(n)) => {}
        _ => panic!("oops!"),
    }
    Context::Node(arena.ast(AST::Value(Value::Number(res as i64))))
}
