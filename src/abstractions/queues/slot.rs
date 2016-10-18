
#![allow(dead_code)] // imported in a few places

use std::prelude::v1::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use abstractions::queues::lock::Lock;

pub struct Slot<T> {
    state: AtomicUsize,
    slot: Lock<Option<T>>,
    on_full: Lock<Option<Box<FnBox<T>>>>,
    on_empty: Lock<Option<(Box<FnBox2<T>>, Option<T>)>>,
}

#[derive(Debug, PartialEq)]
pub struct TryProduceError<T>(T);
#[derive(Debug, PartialEq)]
pub struct TryConsumeError(());
#[derive(Debug, PartialEq)]
pub struct OnFullError(());
#[derive(Debug, PartialEq)]
pub struct OnEmptyError(());
#[derive(Clone, Copy)]
pub struct Token(usize);

struct State(usize);

const DATA: usize = 1 << 0;
const ON_FULL: usize = 1 << 1;
const ON_EMPTY: usize = 1 << 2;
const STATE_BITS: usize = 3;
const STATE_MASK: usize = (1 << STATE_BITS) - 1;

fn _is_send<T: Send>() {}
fn _is_sync<T: Send>() {}

fn _assert() {
    _is_send::<Slot<i32>>();
    _is_sync::<Slot<u32>>();
}

impl<T> Slot<T> {
    pub fn new(val: Option<T>) -> Slot<T> {
        Slot {
            state: AtomicUsize::new(if val.is_some() {DATA} else {0}),
            slot: Lock::new(val),
            on_full: Lock::new(None),
            on_empty: Lock::new(None),
        }
    }

    pub fn try_produce(&self, t: T) -> Result<(), TryProduceError<T>> {
        let mut state = State(self.state.load(Ordering::SeqCst));
        assert!(!state.flag(ON_EMPTY));
        if state.flag(DATA) {
            return Err(TryProduceError(t))
        }

        let mut slot = self.slot.try_lock().expect("interference with consumer?");
        assert!(slot.is_none());
        *slot = Some(t);
        drop(slot);

        loop {
            assert!(!state.flag(ON_EMPTY));
            let new_state = state.set_flag(DATA, true).set_flag(ON_FULL, false);
            let old = self.state.compare_and_swap(state.0,
                                                  new_state.0,
                                                  Ordering::SeqCst);
            if old == state.0 {
                break
            }
            state.0 = old;
        }

        if state.flag(ON_FULL) {
            let cb = self.on_full.try_lock().expect("interference2")
                                 .take().expect("ON_FULL but no callback");
            cb.call_box(self);
        }
        Ok(())
    }

    pub fn on_empty<F>(&self, item: Option<T>, f: F) -> Token
        where F: FnOnce(&Slot<T>, Option<T>) + Send + 'static
    {
        let mut state = State(self.state.load(Ordering::SeqCst));
        assert!(!state.flag(ON_EMPTY));
        if !state.flag(DATA) {
            f(self, item);
            return Token(0)
        }
        assert!(!state.flag(ON_FULL));

        let mut slot = self.on_empty.try_lock().expect("on_empty interference");
        assert!(slot.is_none());
        *slot = Some((Box::new(f), item));
        drop(slot);

        loop {
            assert!(state.flag(DATA));
            assert!(!state.flag(ON_FULL));
            assert!(!state.flag(ON_EMPTY));
            let new_state = state.set_flag(ON_EMPTY, true)
                                 .set_token(state.token() + 1);
            let old = self.state.compare_and_swap(state.0,
                                                  new_state.0,
                                                  Ordering::SeqCst);

            if old == state.0 {
                return Token(new_state.token())
            }
            state.0 = old;

            if !state.flag(DATA) {
                let cb = self.on_empty.try_lock().expect("on_empty interference2")
                                      .take().expect("on_empty not empty??");
                let (cb, item) = cb;
                cb.call_box(self, item);
                return Token(0)
            }
        }
    }

    pub fn try_consume(&self) -> Result<T, TryConsumeError> {
        // The implementation of this method is basically the same as
        // `try_produce` above, it's just the opposite of all the operations.
        let mut state = State(self.state.load(Ordering::SeqCst));
        assert!(!state.flag(ON_FULL));
        if !state.flag(DATA) {
            return Err(TryConsumeError(()))
        }
        let mut slot = self.slot.try_lock().expect("interference with producer?");
        let val = slot.take().expect("DATA but not data");
        drop(slot);

        loop {
            assert!(!state.flag(ON_FULL));
            let new_state = state.set_flag(DATA, false).set_flag(ON_EMPTY, false);
            let old = self.state.compare_and_swap(state.0,
                                                  new_state.0,
                                                  Ordering::SeqCst);
            if old == state.0 {
                break
            }
            state.0 = old;
        }
        assert!(!state.flag(ON_FULL));
        if state.flag(ON_EMPTY) {
            let cb = self.on_empty.try_lock().expect("interference3")
                                  .take().expect("ON_EMPTY but no callback");
            let (cb, item) = cb;
            cb.call_box(self, item);
        }
        Ok(val)
    }

    pub fn on_full<F>(&self, f: F) -> Token
        where F: FnOnce(&Slot<T>) + Send + 'static
    {
        let mut state = State(self.state.load(Ordering::SeqCst));
        assert!(!state.flag(ON_FULL));
        if state.flag(DATA) {
            f(self);
            return Token(0)
        }
        assert!(!state.flag(ON_EMPTY));

        let mut slot = self.on_full.try_lock().expect("on_full interference");
        assert!(slot.is_none());
        *slot = Some(Box::new(f));
        drop(slot);

        loop {
            assert!(!state.flag(DATA));
            assert!(!state.flag(ON_EMPTY));
            assert!(!state.flag(ON_FULL));
            let new_state = state.set_flag(ON_FULL, true)
                                 .set_token(state.token() + 1);
            let old = self.state.compare_and_swap(state.0,
                                                  new_state.0,
                                                  Ordering::SeqCst);
            if old == state.0 {
                return Token(new_state.token())
            }
            state.0 = old;

            if state.flag(DATA) {
                let cb = self.on_full.try_lock().expect("on_full interference2")
                                      .take().expect("on_full not full??");
                cb.call_box(self);
                return Token(0)
            }
        }
    }

    pub fn cancel(&self, token: Token) {
        // Tokens with a value of "0" are sentinels which don't actually do
        // anything.
        let token = token.0;
        if token == 0 {
            return
        }

        let mut state = State(self.state.load(Ordering::SeqCst));
        loop {
            if state.token() != token {
                return
            }

            let new_state = if state.flag(ON_FULL) {
                assert!(!state.flag(ON_EMPTY));
                state.set_flag(ON_FULL, false)
            } else if state.flag(ON_EMPTY) {
                assert!(!state.flag(ON_FULL));
                state.set_flag(ON_EMPTY, false)
            } else {
                return
            };
            let old = self.state.compare_and_swap(state.0,
                                                  new_state.0,
                                                  Ordering::SeqCst);
            if old == state.0 {
                break
            }
            state.0 = old;
        }

        if state.flag(ON_FULL) {
            let cb = self.on_full.try_lock().expect("on_full interference3")
                                 .take().expect("on_full not full??");
            drop(cb);
        } else {
            let cb = self.on_empty.try_lock().expect("on_empty interference3")
                                  .take().expect("on_empty not empty??");
            drop(cb);
        }
    }
}

impl<T> TryProduceError<T> {
    /// Extracts the value that was attempted to be produced.
    pub fn into_inner(self) -> T {
        self.0
    }
}

trait FnBox<T>: Send {
    fn call_box(self: Box<Self>, other: &Slot<T>);
}

impl<T, F> FnBox<T> for F
    where F: FnOnce(&Slot<T>) + Send,
{
    fn call_box(self: Box<F>, other: &Slot<T>) {
        (*self)(other)
    }
}

trait FnBox2<T>: Send {
    fn call_box(self: Box<Self>, other: &Slot<T>, Option<T>);
}

impl<T, F> FnBox2<T> for F
    where F: FnOnce(&Slot<T>, Option<T>) + Send,
{
    fn call_box(self: Box<F>, other: &Slot<T>, item: Option<T>) {
        (*self)(other, item)
    }
}

impl State {
    fn flag(&self, f: usize) -> bool {
        self.0 & f != 0
    }

    fn set_flag(&self, f: usize, val: bool) -> State {
        State(if val {
            self.0 | f
        } else {
            self.0 & !f
        })
    }

    fn token(&self) -> usize {
        self.0 >> STATE_BITS
    }

    fn set_token(&self, gen: usize) -> State {
        State((gen << STATE_BITS) | (self.0 & STATE_MASK))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    use super::Slot;

    #[test]
    fn sequential() {
        let slot = Slot::new(Some(1));

        // We can consume once
        assert_eq!(slot.try_consume(), Ok(1));
        assert!(slot.try_consume().is_err());

        // Consume a production
        assert_eq!(slot.try_produce(2), Ok(()));
        assert_eq!(slot.try_consume(), Ok(2));

        // Can't produce twice
        assert_eq!(slot.try_produce(3), Ok(()));
        assert!(slot.try_produce(3).is_err());

        // on_full is run immediately if full
        let hit = Arc::new(AtomicUsize::new(0));
        let hit2 = hit.clone();
        slot.on_full(move |_s| {
            hit2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(hit.load(Ordering::SeqCst), 1);

        // on_full can be run twice, and we can consume in the callback
        let hit2 = hit.clone();
        slot.on_full(move |s| {
            hit2.fetch_add(1, Ordering::SeqCst);
            assert_eq!(s.try_consume(), Ok(3));
        });
        assert_eq!(hit.load(Ordering::SeqCst), 2);

        // Production can't run a previous callback
        assert_eq!(slot.try_produce(4), Ok(()));
        assert_eq!(hit.load(Ordering::SeqCst), 2);
        assert_eq!(slot.try_consume(), Ok(4));

        // Productions run new callbacks
        let hit2 = hit.clone();
        slot.on_full(move |s| {
            hit2.fetch_add(1, Ordering::SeqCst);
            assert_eq!(s.try_consume(), Ok(5));
        });
        assert_eq!(slot.try_produce(5), Ok(()));
        assert_eq!(hit.load(Ordering::SeqCst), 3);

        // on empty should fire immediately for an empty slot
        let hit2 = hit.clone();
        slot.on_empty(None, move |_, _| {
            hit2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(hit.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn channel() {
        const N: usize = 10000;

        struct Sender {
            slot: Arc<Slot<usize>>,
            hit: Arc<AtomicUsize>,
        }

        struct Receiver {
            slot: Arc<Slot<usize>>,
            hit: Arc<AtomicUsize>,
        }

        impl Sender {
            fn send(&self, val: usize) {
                if self.slot.try_produce(val).is_ok() {
                    return
                }
                let me = thread::current();
                self.hit.store(0, Ordering::SeqCst);
                let hit = self.hit.clone();
                self.slot.on_empty(None, move |_slot, _| {
                    hit.store(1, Ordering::SeqCst);
                    me.unpark();
                });
                while self.hit.load(Ordering::SeqCst) == 0 {
                    thread::park();
                }
                self.slot.try_produce(val).expect("can't produce after on_empty")
            }
        }

        impl Receiver {
            fn recv(&self) -> usize {
                if let Ok(i) = self.slot.try_consume() {
                    return i
                }

                let me = thread::current();
                self.hit.store(0, Ordering::SeqCst);
                let hit = self.hit.clone();
                self.slot.on_full(move |_slot| {
                    hit.store(1, Ordering::SeqCst);
                    me.unpark();
                });
                while self.hit.load(Ordering::SeqCst) == 0 {
                    thread::park();
                }
                self.slot.try_consume().expect("can't consume after on_full")
            }
        }

        let slot = Arc::new(Slot::new(None));
        let slot2 = slot.clone();

        let tx = Sender { slot: slot2, hit: Arc::new(AtomicUsize::new(0)) };
        let rx = Receiver { slot: slot, hit: Arc::new(AtomicUsize::new(0)) };

        let a = thread::spawn(move || {
            for i in 0..N {
                assert_eq!(rx.recv(), i);
            }
        });

        for i in 0..N {
            tx.send(i);
        }

        a.join().unwrap();
    }

    #[test]
    fn cancel() {
        let slot = Slot::new(None);
        let hits = Arc::new(AtomicUsize::new(0));

        let add = || {
            let hits = hits.clone();
            move |_: &Slot<u32>| { hits.fetch_add(1, Ordering::SeqCst); }
        };
        let add_empty = || {
            let hits = hits.clone();
            move |_: &Slot<u32>, _: Option<u32>| {
                hits.fetch_add(1, Ordering::SeqCst);
            }
        };

        // cancel on_full
        let n = hits.load(Ordering::SeqCst);
        assert_eq!(hits.load(Ordering::SeqCst), n);
        let token = slot.on_full(add());
        assert_eq!(hits.load(Ordering::SeqCst), n);
        slot.cancel(token);
        assert_eq!(hits.load(Ordering::SeqCst), n);
        assert!(slot.try_consume().is_err());
        assert!(slot.try_produce(1).is_ok());
        assert!(slot.try_consume().is_ok());
        assert_eq!(hits.load(Ordering::SeqCst), n);

        // cancel on_empty
        let n = hits.load(Ordering::SeqCst);
        assert_eq!(hits.load(Ordering::SeqCst), n);
        slot.try_produce(1).unwrap();
        let token = slot.on_empty(None, add_empty());
        assert_eq!(hits.load(Ordering::SeqCst), n);
        slot.cancel(token);
        assert_eq!(hits.load(Ordering::SeqCst), n);
        assert!(slot.try_produce(1).is_err());

        // cancel with no effect
        let n = hits.load(Ordering::SeqCst);
        assert_eq!(hits.load(Ordering::SeqCst), n);
        let token = slot.on_full(add());
        assert_eq!(hits.load(Ordering::SeqCst), n + 1);
        slot.cancel(token);
        assert_eq!(hits.load(Ordering::SeqCst), n + 1);
        assert!(slot.try_consume().is_ok());
        let token = slot.on_empty(None, add_empty());
        assert_eq!(hits.load(Ordering::SeqCst), n + 2);
        slot.cancel(token);
        assert_eq!(hits.load(Ordering::SeqCst), n + 2);

        // cancel old ones don't count
        let n = hits.load(Ordering::SeqCst);
        assert_eq!(hits.load(Ordering::SeqCst), n);
        let token1 = slot.on_full(add());
        assert_eq!(hits.load(Ordering::SeqCst), n);
        assert!(slot.try_produce(1).is_ok());
        assert_eq!(hits.load(Ordering::SeqCst), n + 1);
        assert!(slot.try_consume().is_ok());
        assert_eq!(hits.load(Ordering::SeqCst), n + 1);
        let token2 = slot.on_full(add());
        assert_eq!(hits.load(Ordering::SeqCst), n + 1);
        slot.cancel(token1);
        assert_eq!(hits.load(Ordering::SeqCst), n + 1);
        slot.cancel(token2);
        assert_eq!(hits.load(Ordering::SeqCst), n + 1);
    }
}
