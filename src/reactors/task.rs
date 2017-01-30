use commands::ast::ASTNode;
use intercore::message::Message;
use reactors::scheduler::Scheduler;

#[derive(Debug,Clone,Copy)]
pub struct TaskId(pub usize, pub usize);

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Termination {
    Recursive,
    Corecursive,
}

#[derive(Debug)]
pub struct T3<T>(pub T, pub Termination);

#[derive(Debug,Clone)]
pub enum Poll<T, E> {
    Yield(T),
    End(T),
    Err(E),
    Infinite,
}

#[derive(Debug,Clone)]
pub enum Error {
    RuntimeError,
    WrongContext,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Context<'a> {
    Cont(usize, &'a Message),
    Node(&'a ASTNode<'a>),
    NodeAck(usize, usize),
    Intercore(&'a Message),
    Init(usize),
    Nil,
}

pub trait Task<'a> {
    fn init(&'a mut self, input: Option<&'a str>, task_id: usize);
    fn exec(&'a mut self, input: Option<&'a str>);
    fn poll(&'a mut self, c: Context<'a>, sched: &'a Scheduler<'a>) -> Poll<Context<'a>, Error>;
    fn finalize(&'a mut self);
}