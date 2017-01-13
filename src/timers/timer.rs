
// Timer Stream by Nikolas

use io::event::*;
use io::ready::*;
use io::token::*;
use io::options::*;
use io::registration::*;
use io::poll::*;

use timers::slab;
use std::{self, cmp, error, fmt, u64, usize, iter, thread, io};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::cell::UnsafeCell;

const NANOS_PER_MILLI: u32 = 1_000_000;
const MILLIS_PER_SEC: u64 = 1_000;

pub fn millis(duration: Duration) -> u64 {
    let millis = (duration.subsec_nanos() + NANOS_PER_MILLI - 1) / NANOS_PER_MILLI;
    duration.as_secs().saturating_mul(MILLIS_PER_SEC).saturating_add(millis as u64)
}

pub struct LazyCell<T> {
    inner: UnsafeCell<Option<T>>,
}

impl<T> LazyCell<T> {
    pub fn new() -> LazyCell<T> {
        LazyCell { inner: UnsafeCell::new(None) }
    }

    pub fn fill(&self, t: T) -> std::result::Result<(), T> {
        let mut slot = unsafe { &mut *self.inner.get() };
        if slot.is_some() {
            return Err(t);
        }
        *slot = Some(t);

        Ok(())
    }

    pub fn filled(&self) -> bool {
        self.borrow().is_some()
    }

    pub fn borrow(&self) -> Option<&T> {
        unsafe { &*self.inner.get() }.as_ref()
    }

    pub fn into_inner(self) -> Option<T> {
        unsafe { self.inner.into_inner() }
    }
}

const NONE: usize = 0;
const LOCK: usize = 1;
const SOME: usize = 2;

pub struct AtomicLazyCell<T> {
    inner: UnsafeCell<Option<T>>,
    state: AtomicUsize,
}

impl<T> AtomicLazyCell<T> {
    pub fn new() -> AtomicLazyCell<T> {
        AtomicLazyCell {
            inner: UnsafeCell::new(None),
            state: AtomicUsize::new(NONE),
        }
    }

    pub fn fill(&self, t: T) -> std::result::Result<(), T> {
        if NONE != self.state.compare_and_swap(NONE, LOCK, Ordering::Acquire) {
            return Err(t);
        }

        unsafe { *self.inner.get() = Some(t) };

        if LOCK != self.state.compare_and_swap(LOCK, SOME, Ordering::Release) {
            panic!("unable to release lock");
        }

        Ok(())
    }

    pub fn filled(&self) -> bool {
        self.state.load(Ordering::Acquire) == SOME
    }

    pub fn borrow(&self) -> Option<&T> {
        match self.state.load(Ordering::Acquire) {
            SOME => unsafe { &*self.inner.get() }.as_ref(),
            _ => None,
        }
    }

    pub fn into_inner(self) -> Option<T> {
        unsafe { self.inner.into_inner() }
    }
}

unsafe impl<T: Sync> Sync for AtomicLazyCell<T> {}
unsafe impl<T: Send> Send for AtomicLazyCell<T> {}

use self::TimerErrorKind::TimerOverflow;

pub struct Timer<T> {
    tick_ms: u64,
    entries: Slab<Entry<T>>,
    wheel: Vec<WheelEntry>,
    start: Instant,
    tick: Tick,
    next: Token,
    mask: u64,
    inner: LazyCell<Inner>,
}

pub struct Builder {
    tick: Duration,
    num_slots: usize,
    capacity: usize,
}

#[derive(Clone, Debug)]
pub struct Timeout {
    token: Token,
    tick: u64,
}

struct Inner {
    registration: Registration,
    set_readiness: SetReadiness,
    wakeup_state: WakeupState,
    wakeup_thread: thread::JoinHandle<()>,
}

#[derive(Copy, Clone, Debug)]
struct WheelEntry {
    next_tick: Tick,
    head: Token,
}

struct Entry<T> {
    state: T,
    links: EntryLinks,
}

#[derive(Copy, Clone)]
struct EntryLinks {
    tick: Tick,
    prev: Token,
    next: Token,
}

type Tick = u64;

const TICK_MAX: Tick = u64::MAX;

type WakeupState = Arc<AtomicUsize>;
type Slab<T> = slab::Slab<T, Token>;

pub type Result<T> = std::result::Result<T, TimerError>;
pub type TimerResult<T> = Result<T>;


#[derive(Debug)]
pub struct TimerError {
    kind: TimerErrorKind,
    desc: &'static str,
}

#[derive(Debug)]
pub enum TimerErrorKind {
    TimerOverflow,
}

const TERMINATE_THREAD: usize = 0;
const EMPTY: Token = Token(usize::MAX);

impl Builder {
    pub fn tick_duration(mut self, duration: Duration) -> Builder {
        self.tick = duration;
        self
    }

    pub fn num_slots(mut self, num_slots: usize) -> Builder {
        self.num_slots = num_slots;
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Builder {
        self.capacity = capacity;
        self
    }

    pub fn build<T>(self) -> Timer<T> {
        Timer::new(millis(self.tick),
                   self.num_slots,
                   self.capacity,
                   Instant::now())
    }
}

impl Default for Builder {
    fn default() -> Builder {
        Builder {
            tick: Duration::from_millis(100),
            num_slots: 256,
            capacity: 65_536,
        }
    }
}

impl<T> Timer<T> {
    fn new(tick_ms: u64, num_slots: usize, capacity: usize, start: Instant) -> Timer<T> {
        let num_slots = num_slots.next_power_of_two();
        let capacity = capacity.next_power_of_two();
        let mask = (num_slots as u64) - 1;
        let wheel = iter::repeat(WheelEntry {
                next_tick: TICK_MAX,
                head: EMPTY,
            })
            .take(num_slots)
            .collect();

        Timer {
            tick_ms: tick_ms,
            entries: Slab::with_capacity(capacity),
            wheel: wheel,
            start: start,
            tick: 0,
            next: EMPTY,
            mask: mask,
            inner: LazyCell::new(),
        }
    }

    pub fn set_timeout(&mut self, delay_from_now: Duration, state: T) -> Result<Timeout> {
        let delay_from_start = self.start.elapsed() + delay_from_now;
        self.set_timeout_at(delay_from_start, state)
    }

    fn set_timeout_at(&mut self, delay_from_start: Duration, state: T) -> Result<Timeout> {
        let mut tick = duration_to_tick(delay_from_start, self.tick_ms);
        trace!("setting timeout; delay={:?}; tick={:?}; current-tick={:?}",
               delay_from_start,
               tick,
               self.tick);

        if tick <= self.tick {
            tick = self.tick + 1;
        }

        self.insert(tick, state)
    }

    fn insert(&mut self, tick: Tick, state: T) -> Result<Timeout> {
        let slot = (tick & self.mask) as usize;
        let curr = self.wheel[slot];

        let token = try!(self.entries
            .insert(Entry::new(state, tick, curr.head))
            .map_err(|_| TimerError::overflow()));

        if curr.head != EMPTY {
            self.entries[curr.head].links.prev = token;
        }

        self.wheel[slot] = WheelEntry {
            next_tick: cmp::min(tick, curr.next_tick),
            head: token,
        };

        self.schedule_readiness(tick);

        trace!("inserted timout; slot={}; token={:?}", slot, token);

        Ok(Timeout {
            token: token,
            tick: tick,
        })
    }

    pub fn cancel_timeout(&mut self, timeout: &Timeout) -> Option<T> {
        let links = match self.entries.get(timeout.token) {
            Some(e) => e.links,
            None => return None,
        };

        if links.tick != timeout.tick {
            return None;
        }

        self.unlink(&links, timeout.token);
        self.entries.remove(timeout.token).map(|e| e.state)
    }

    pub fn poll(&mut self) -> Option<T> {
        let target_tick = current_tick(self.start, self.tick_ms);
        self.poll_to(target_tick)
    }

    fn poll_to(&mut self, target_tick: Tick) -> Option<T> {
        trace!("tick_to; target_tick={}; current_tick={}",
               target_tick,
               self.tick);

        while self.tick <= target_tick {
            let curr = self.next;

            trace!("ticking; curr={:?}", curr);

            if curr == EMPTY {
                self.tick += 1;

                let slot = self.slot_for(self.tick);
                self.next = self.wheel[slot].head;

                if self.next == EMPTY {
                    self.wheel[slot].next_tick = TICK_MAX;
                }
            } else {
                let slot = self.slot_for(self.tick);

                if curr == self.wheel[slot].head {
                    self.wheel[slot].next_tick = TICK_MAX;
                }

                let links = self.entries[curr].links;

                if links.tick <= self.tick {
                    trace!("triggering; token={:?}", curr);
                    self.unlink(&links, curr);
                    return self.entries
                        .remove(curr)
                        .map(|e| e.state);
                } else {
                    let next_tick = self.wheel[slot].next_tick;
                    self.wheel[slot].next_tick = cmp::min(next_tick, links.tick);
                    self.next = links.next;
                }
            }
        }

        if let Some(inner) = self.inner.borrow() {
            trace!("unsetting readiness");
            let _ = inner.set_readiness.set_readiness(Ready::none());

            if let Some(tick) = self.next_tick() {
                self.schedule_readiness(tick);
            }
        }

        None
    }

    fn unlink(&mut self, links: &EntryLinks, token: Token) {
        trace!("unlinking timeout; slot={}; token={:?}",
               self.slot_for(links.tick),
               token);

        if links.prev == EMPTY {
            let slot = self.slot_for(links.tick);
            self.wheel[slot].head = links.next;
        } else {
            self.entries[links.prev].links.next = links.next;
        }

        if links.next != EMPTY {
            self.entries[links.next].links.prev = links.prev;

            if token == self.next {
                self.next = links.next;
            }
        } else if token == self.next {
            self.next = EMPTY;
        }
    }

    fn schedule_readiness(&self, tick: Tick) {
        if let Some(inner) = self.inner.borrow() {
            let mut curr = inner.wakeup_state.load(Ordering::Acquire);

            loop {
                if curr as Tick <= tick {
                    return;
                }

                trace!("advancing the wakeup time; target={}; curr={}", tick, curr);
                let actual = inner.wakeup_state
                    .compare_and_swap(curr, tick as usize, Ordering::Release);

                if actual == curr {
                    inner.wakeup_thread.thread().unpark();
                    return;
                }

                curr = actual;
            }
        }
    }

    fn next_tick(&self) -> Option<Tick> {
        if self.next != EMPTY {
            let slot = self.slot_for(self.entries[self.next].links.tick);

            if self.wheel[slot].next_tick == self.tick {
                return Some(self.tick);
            }
        }

        self.wheel.iter().map(|e| e.next_tick).min()
    }

    fn slot_for(&self, tick: Tick) -> usize {
        (self.mask & tick) as usize
    }
}

impl<T> Default for Timer<T> {
    fn default() -> Timer<T> {
        Builder::default().build()
    }
}

impl<T> Evented for Timer<T> {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        if self.inner.borrow().is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "timer already registered"));
        }

        let (registration, set_readiness) = Registration::new(poll, token, interest, opts);
        let wakeup_state = Arc::new(AtomicUsize::new(usize::MAX));
        let thread_handle = spawn_wakeup_thread(wakeup_state.clone(),
                                                set_readiness.clone(),
                                                self.start,
                                                self.tick_ms);

        self.inner
            .fill(Inner {
                registration: registration,
                set_readiness: set_readiness,
                wakeup_state: wakeup_state,
                wakeup_thread: thread_handle,
            })
            .ok()
            .expect("timer already registered");

        if let Some(next_tick) = self.next_tick() {
            self.schedule_readiness(next_tick);
        }

        Ok(())
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        match self.inner.borrow() {
            Some(inner) => inner.registration.update(poll, token, interest, opts),
            None => Err(io::Error::new(io::ErrorKind::Other, "receiver not registered")),
        }
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        match self.inner.borrow() {
            Some(inner) => inner.registration.deregister(poll),
            None => Err(io::Error::new(io::ErrorKind::Other, "receiver not registered")),
        }
    }
}

impl fmt::Debug for Inner {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Inner")
            .field("registration", &self.registration)
            .field("wakeup_state", &self.wakeup_state.load(Ordering::Relaxed))
            .finish()
    }
}

fn spawn_wakeup_thread(state: WakeupState,
                       set_readiness: SetReadiness,
                       start: Instant,
                       tick_ms: u64)
                       -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut sleep_until_tick = state.load(Ordering::Acquire) as Tick;

        loop {
            if sleep_until_tick == TERMINATE_THREAD as Tick {
                return;
            }

            let now_tick = current_tick(start, tick_ms);

            trace!("wakeup thread: sleep_until_tick={:?}; now_tick={:?}",
                   sleep_until_tick,
                   now_tick);

            if now_tick < sleep_until_tick {
                let sleep_duration = tick_ms.checked_mul(sleep_until_tick - now_tick)
                    .unwrap_or(u64::MAX);
                trace!("sleeping; tick_ms={}; now_tick={}; sleep_until_tick={}; duration={:?}",
                       tick_ms,
                       now_tick,
                       sleep_until_tick,
                       sleep_duration);
                thread::park_timeout(Duration::from_millis(sleep_duration));
                sleep_until_tick = state.load(Ordering::Acquire) as Tick;
            } else {
                let actual = state.compare_and_swap(sleep_until_tick as usize, usize::MAX, Ordering::AcqRel) as Tick;

                if actual == sleep_until_tick {
                    trace!("setting readiness from wakeup thread");
                    let _ = set_readiness.set_readiness(Ready::readable());
                    sleep_until_tick = usize::MAX as Tick;
                } else {
                    sleep_until_tick = actual as Tick;
                }
            }
        }
    })
}

fn duration_to_tick(elapsed: Duration, tick_ms: u64) -> Tick {
    let elapsed_ms = millis(elapsed);
    elapsed_ms.saturating_add(tick_ms / 2) / tick_ms
}

fn current_tick(start: Instant, tick_ms: u64) -> Tick {
    duration_to_tick(start.elapsed(), tick_ms)
}

impl<T> Entry<T> {
    fn new(state: T, tick: u64, next: Token) -> Entry<T> {
        Entry {
            state: state,
            links: EntryLinks {
                tick: tick,
                prev: EMPTY,
                next: next,
            },
        }
    }
}

impl fmt::Display for TimerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {}", self.kind, self.desc)
    }
}

impl TimerError {
    fn overflow() -> TimerError {
        TimerError {
            kind: TimerOverflow,
            desc: "too many timer entries",
        }
    }
}

impl error::Error for TimerError {
    fn description(&self) -> &str {
        self.desc
    }
}

impl fmt::Display for TimerErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TimerOverflow => write!(fmt, "TimerOverflow"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    pub fn test_timeout_next_tick() {
        let mut t = timer();
        let mut tick;

        t.set_timeout_at(Duration::from_millis(100), "a").unwrap();

        tick = ms_to_tick(&t, 50);
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 100);
        assert_eq!(Some("a"), t.poll_to(tick));
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 150);
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 200);
        assert_eq!(None, t.poll_to(tick));

        assert_eq!(count(&t), 0);
    }

    #[test]
    pub fn test_clearing_timeout() {
        let mut t = timer();
        let mut tick;

        let to = t.set_timeout_at(Duration::from_millis(100), "a").unwrap();
        assert_eq!("a", t.cancel_timeout(&to).unwrap());

        tick = ms_to_tick(&t, 100);
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 200);
        assert_eq!(None, t.poll_to(tick));

        assert_eq!(count(&t), 0);
    }

    #[test]
    pub fn test_multiple_timeouts_same_tick() {
        let mut t = timer();
        let mut tick;

        t.set_timeout_at(Duration::from_millis(100), "a").unwrap();
        t.set_timeout_at(Duration::from_millis(100), "b").unwrap();

        let mut rcv = vec![];

        tick = ms_to_tick(&t, 100);
        rcv.push(t.poll_to(tick).unwrap());
        rcv.push(t.poll_to(tick).unwrap());

        assert_eq!(None, t.poll_to(tick));

        rcv.sort();
        assert!(rcv == ["a", "b"], "actual={:?}", rcv);

        tick = ms_to_tick(&t, 200);
        assert_eq!(None, t.poll_to(tick));

        assert_eq!(count(&t), 0);
    }

    #[test]
    pub fn test_multiple_timeouts_diff_tick() {
        let mut t = timer();
        let mut tick;

        t.set_timeout_at(Duration::from_millis(110), "a").unwrap();
        t.set_timeout_at(Duration::from_millis(220), "b").unwrap();
        t.set_timeout_at(Duration::from_millis(230), "c").unwrap();
        t.set_timeout_at(Duration::from_millis(440), "d").unwrap();
        t.set_timeout_at(Duration::from_millis(560), "e").unwrap();

        tick = ms_to_tick(&t, 100);
        assert_eq!(Some("a"), t.poll_to(tick));
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 200);
        assert_eq!(Some("c"), t.poll_to(tick));
        assert_eq!(Some("b"), t.poll_to(tick));
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 300);
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 400);
        assert_eq!(Some("d"), t.poll_to(tick));
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 500);
        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 600);
        assert_eq!(Some("e"), t.poll_to(tick));
        assert_eq!(None, t.poll_to(tick));
    }

    #[test]
    pub fn test_catching_up() {
        let mut t = timer();

        t.set_timeout_at(Duration::from_millis(110), "a").unwrap();
        t.set_timeout_at(Duration::from_millis(220), "b").unwrap();
        t.set_timeout_at(Duration::from_millis(230), "c").unwrap();
        t.set_timeout_at(Duration::from_millis(440), "d").unwrap();

        let tick = ms_to_tick(&t, 600);
        assert_eq!(Some("a"), t.poll_to(tick));
        assert_eq!(Some("c"), t.poll_to(tick));
        assert_eq!(Some("b"), t.poll_to(tick));
        assert_eq!(Some("d"), t.poll_to(tick));
        assert_eq!(None, t.poll_to(tick));
    }

    #[test]
    pub fn test_timeout_hash_collision() {
        let mut t = timer();
        let mut tick;

        t.set_timeout_at(Duration::from_millis(100), "a").unwrap();
        t.set_timeout_at(Duration::from_millis(100 + TICK * SLOTS as u64), "b").unwrap();

        tick = ms_to_tick(&t, 100);
        assert_eq!(Some("a"), t.poll_to(tick));
        assert_eq!(1, count(&t));

        tick = ms_to_tick(&t, 200);
        assert_eq!(None, t.poll_to(tick));
        assert_eq!(1, count(&t));

        tick = ms_to_tick(&t, 100 + TICK * SLOTS as u64);
        assert_eq!(Some("b"), t.poll_to(tick));
        assert_eq!(0, count(&t));
    }

    #[test]
    pub fn test_clearing_timeout_between_triggers() {
        let mut t = timer();
        let mut tick;

        let a = t.set_timeout_at(Duration::from_millis(100), "a").unwrap();
        let _ = t.set_timeout_at(Duration::from_millis(100), "b").unwrap();
        let _ = t.set_timeout_at(Duration::from_millis(200), "c").unwrap();

        tick = ms_to_tick(&t, 100);
        assert_eq!(Some("b"), t.poll_to(tick));
        assert_eq!(2, count(&t));

        t.cancel_timeout(&a);
        assert_eq!(1, count(&t));

        assert_eq!(None, t.poll_to(tick));

        tick = ms_to_tick(&t, 200);
        assert_eq!(Some("c"), t.poll_to(tick));
        assert_eq!(0, count(&t));
    }

    const TICK: u64 = 100;
    const SLOTS: usize = 16;
    const CAPACITY: usize = 32;

    fn count<T>(timer: &Timer<T>) -> usize {
        timer.entries.len()
    }

    fn timer() -> Timer<&'static str> {
        Timer::new(TICK, SLOTS, CAPACITY, Instant::now())
    }

    fn ms_to_tick<T>(timer: &Timer<T>, ms: u64) -> u64 {
        ms / timer.tick_ms
    }

}
