
use std::io;
use std::time::{Duration, Instant};
use abstractions::futures::future::Future;
use abstractions::poll::{Poll, Async};
use abstractions::tasks::task;
use reactors::tokio::sched::{Message, Handle, Remote};

pub struct Timeout {
    token: TimeoutToken,
    when: Instant,
    handle: Remote,
}

impl Timeout {
    pub fn new(dur: Duration, handle: &Handle) -> io::Result<Timeout> {
        Timeout::new_at(Instant::now() + dur, handle)
    }

    pub fn new_at(at: Instant, handle: &Handle) -> io::Result<Timeout> {
        Ok(Timeout {
            token: try!(TimeoutToken::new(at, &handle)),
            when: at,
            handle: handle.remote().clone(),
        })
    }
}

impl Future for Timeout {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {

        let now = Instant::now();
        if self.when <= now {
            Ok(Async::Ready(()))
        } else {
            self.token.update_timeout(&self.handle);
            Ok(Async::NotReady)
        }
    }
}

impl Drop for Timeout {
    fn drop(&mut self) {
        self.token.cancel_timeout(&self.handle);
    }
}

pub struct TimeoutToken {
    token: usize,
}

impl TimeoutToken {
    pub fn new(at: Instant, handle: &Handle) -> io::Result<TimeoutToken> {
        match handle.inner.upgrade() {
            Some(inner) => {
                let token = inner.borrow_mut().add_timeout(at);
                Ok(TimeoutToken { token: token })
            }
            None => Err(io::Error::new(io::ErrorKind::Other, "event loop gone")),
        }
    }


    pub fn update_timeout(&self, handle: &Remote) {
        handle.send(Message::UpdateTimeout(self.token, task::park()))
    }

    pub fn reset_timeout(&mut self, at: Instant, handle: &Remote) {
        handle.send(Message::ResetTimeout(self.token, at));
    }

    pub fn cancel_timeout(&self, handle: &Remote) {
        println!("cancel timeout {}", self.token);
        handle.send(Message::CancelTimeout(self.token))
    }
}
