use commands::ast::AST;
use streams::intercore::api::Message;

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

#[derive(Debug)]
pub enum Context<'a> {
    Cont(usize, Message),
    Node(&'a AST<'a>),
    IntercoreNode(Message),
    Nil,
}

pub trait Task<'a> {
    fn init(&'a mut self, input: Option<&'a str>);
    fn exec(&'a mut self, input: Option<&'a str>);
    fn poll(&'a mut self, c: Context<'a>) -> Poll<Context<'a>, Error>;
    fn finalize(&'a mut self);
}