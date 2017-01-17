use reactors::task::{Task, Context, Poll, Error};
use queues::publisher::{Publisher, Subscriber};
use streams::intercore::api::Message;
use std::ffi::CString;

pub struct IntercoreTask {
    id: usize,
    publisher: Publisher<Message>,
    subscribers: Vec<Subscriber<Message>>,
}

impl IntercoreTask {
    pub fn new(id: usize, capacity: usize) -> Self {
        IntercoreTask {
            id: id,
            publisher: Publisher::with_mirror(CString::new(format!("/ipc_{}", id)).unwrap(), capacity),
            subscribers: Vec::with_capacity(capacity),
        }
    }

    pub fn add_subscriber(&mut self, s: Subscriber<Message>) {
        self.subscribers.push(s);
    }
}

impl<'a> Task<'a> for IntercoreTask {
    fn init(&'a mut self, input: Option<&'a str>) {
        //
    }

    fn exec(&'a mut self, input: Option<&'a str>) {
        //
    }

    fn poll(&'a mut self, c: Context<'a>) -> Poll<Context<'a>, Error> {
        println!("POLL");
        Poll::Yield(Context::Nil)
    }

    fn finalize(&'a mut self) {
        //
    }
}