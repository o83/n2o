use commands::ast::AST;

#[derive(Debug)]
pub enum Poll<T, E> {
    Yield(T),
    Err(E),
    End,
}

#[derive(Debug)]
pub enum Error {
    RuntimeError,
    WrongContext,
}

#[derive(Debug)]
pub enum Context<'a> {
    Cont(usize),
    Node(&'a AST<'a>),
    Nil,
}

pub trait Task<'a> {
    fn init(&'a mut self);
    fn poll(&'a mut self, c: Context<'a>) -> Poll<Context<'a>, Error>;
}