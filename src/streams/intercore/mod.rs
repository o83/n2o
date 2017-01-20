pub mod ctx;
pub mod ring;
pub mod api;
use commands::ast::{AST, Arena};
use streams::intercore::ring::{pub_, sub_, snd_, rcv_};
use reactors::task::Context;

pub fn internals<'a>(f_id: u16, args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    match f_id {
        0 => print(args, arena),
        1 => pub_(args, arena),
        2 => sub_(args, arena),
        3 => snd_(args, arena),
        4 => rcv_(args, arena),
        5 => spawn_(args, arena), // args should include Host
        6 => select_(args, arena),
        _ => panic!("unknown internal func"),
    }
}

pub fn print<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("{:?}", args);
    Context::Node(arena.nil())
}

pub fn spawn_<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    println!("{:?}", args);
    Context::Node(arena.nil())
}

pub fn select_<'a>(args: &'a AST<'a>, arena: &'a Arena<'a>) -> Context<'a> {
    Context::Node(arena.nil())
}
