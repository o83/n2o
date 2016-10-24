
use std::rc::Rc;
use std::io::{Result, Error};
use network::message::Message;
use network::endpoint::Context;
use network::tcp::pipe::{Event, AsyncPipeStub};
use mio::Ready;

pub struct Dead;

impl<S: AsyncPipeStub + 'static> PipeState<S> for Dead {
    fn name(&self) -> &'static str {
        "Dead"
    }
    fn enter(&self, ctx: &mut Context) {
        ctx.raise(Event::Closed);
    }
    fn open(self: Box<Self>, _: &mut Context) -> Box<PipeState<S>> {
        self
    }
    fn close(self: Box<Self>, _: &mut Context) -> Box<PipeState<S>> {
        self
    }
    fn send(self: Box<Self>, _: &mut Context, _: Rc<Message>) -> Box<PipeState<S>> {
        self
    }
    fn recv(self: Box<Self>, _: &mut Context) -> Box<PipeState<S>> {
        self
    }
    fn ready(self: Box<Self>, _: &mut Context, _: Ready) -> Box<PipeState<S>> {
        self
    }
}

pub trait PipeState<S: AsyncPipeStub + 'static> {
    fn name(&self) -> &'static str;
    fn open(self: Box<Self>, _: &mut Context) -> Box<PipeState<S>> {
        box Dead
    }
    fn close(self: Box<Self>, _: &mut Context) -> Box<PipeState<S>> {
        box Dead
    }
    fn send(self: Box<Self>, _: &mut Context, _: Rc<Message>) -> Box<PipeState<S>> {
        box Dead
    }
    fn recv(self: Box<Self>, _: &mut Context) -> Box<PipeState<S>> {
        box Dead
    }
    fn error(self: Box<Self>, ctx: &mut Context, err: Error) -> Box<PipeState<S>> {
        ctx.raise(Event::Error(err));

        box Dead
    }
    fn ready(self: Box<Self>, _: &mut Context, _: Ready) -> Box<PipeState<S>> {
        box Dead
    }
    fn enter(&self, _: &mut Context) {}
    fn leave(&self, _: &mut Context) {}
}

pub fn transition<F, T, S>(old_state: Box<F>, ctx: &mut Context) -> Box<T>
    where F: PipeState<S>,
          F: Into<T>,
          T: PipeState<S>,
          S: AsyncPipeStub + 'static
{
    old_state.leave(ctx);
    let new_state = Into::into(*old_state);
    new_state.enter(ctx);
    box new_state
}

pub fn transition_if_ok<F, T, S>(f: Box<F>, ctx: &mut Context, res: Result<()>) -> Box<PipeState<S>>
    where F: PipeState<S>,
          F: Into<T>,
          T: PipeState<S> + 'static,
          S: AsyncPipeStub + 'static
{
    match res {
        Ok(..) => transition::<F, T, S>(f, ctx),
        Err(e) => f.error(ctx, e),
    }
}

pub fn no_transition_if_ok<F, S>(f: Box<F>, ctx: &mut Context, res: Result<()>) -> Box<PipeState<S>>
    where F: PipeState<S> + 'static,
          S: AsyncPipeStub + 'static
{
    match res {
        Ok(..) => f,
        Err(e) => f.error(ctx, e),
    }
}
