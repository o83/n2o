pub mod ctx;
pub mod ring;
use commands::ast::AST;
use streams::intercore::ctx::{Ctx, Ctxs};
use streams::intercore::ring::{pub_, sub_, snd_, rcv_};

pub fn internals<'a>(f_id: u16, args: &'a AST<'a>, ctx: &Ctx<u64>) -> AST<'a> {
    match f_id {
        0 => print(args, ctx),
        1 => pub_(args, ctx),
        2 => sub_(args, ctx),
        3 => snd_(args, ctx),
        4 => rcv_(args, ctx),
        _ => panic!("unknown internal func"),
    }
}

pub fn print<'a>(args: &'a AST<'a>, ctx: &Ctx<u64>) -> AST<'a> {
    println!("{:?}", args);
    AST::Nil
}