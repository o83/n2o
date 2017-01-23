use commands::ast::AST;
use intercore::message::Message;

#[derive(Debug)]
pub enum Poll<T, E> {
    Yield(T),
    End(T),
    Err(E),
}

#[derive(Debug)]
pub enum Error {
    RuntimeError,
    WrongContext,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Context<'a> {
    Cont(usize, &'a Message),
    Node(&'a AST<'a>),
    NodeAck(usize, usize),
    Intercore(&'a Message),
    Init(usize),
    Nil,
}

pub trait Task<'a> {
    fn init(&'a mut self, input: Option<&'a str>, task_id: usize);
    fn exec(&'a mut self, input: Option<&'a str>);
    fn poll(&'a mut self, c: Context<'a>) -> Poll<Context<'a>, Error>;
    fn finalize(&'a mut self);
}