// Generic type holds all implemetors of Task trait.

use reactors::task::{self, Poll, Context, Task};
use reactors::cpstask::CpsTask;

pub enum Job<'a> {
    Cps(CpsTask<'a>),
}

impl<'a> Job<'a> {
    fn unwrap(&'a mut self) -> &'a mut Task<'a> {
        match *self {
            Job::Cps(ref mut c) => c,
        }
    }
}

impl<'a> Task<'a> for Job<'a> {
    fn init(&'a mut self, input: Option<&'a str>) {
        self.unwrap().init(input)
    }
    fn poll(&'a mut self, c: Context<'a>) -> Poll<Context<'a>, task::Error> {
        self.unwrap().poll(c)
    }
    fn finalize(&'a mut self) {
        self.unwrap().finalize()
    }
}