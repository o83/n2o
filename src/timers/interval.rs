
use std::io;
use std::time::{Duration, Instant};

use abstractions::poll::{Poll, Async};
use abstractions::streams::stream::Stream;

use reactors::tokio::sched::{Remote, Handle};
use reactors::tokio::timeout::TimeoutToken;

pub struct Interval {
    token: TimeoutToken,
    next: Instant,
    interval: Duration,
    handle: Remote,
}

impl Interval {
    pub fn new(dur: Duration, handle: &Handle) -> io::Result<Interval> {
        Interval::new_at(Instant::now() + dur, dur, handle)
    }

    pub fn new_at(at: Instant, dur: Duration, handle: &Handle) -> io::Result<Interval> {
        Ok(Interval {
            token: try!(TimeoutToken::new(at, &handle)),
            next: at,
            interval: dur,
            handle: handle.remote().clone(),
        })
    }
}

impl Stream for Interval {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<()>, io::Error> {
        // TODO: is this fast enough?
        let now = Instant::now();
        if self.next <= now {
            self.next = next_interval(self.next, now, self.interval);
            self.token.reset_timeout(self.next, &self.handle);
            Ok(Async::Ready(Some(())))
        } else {
            self.token.update_timeout(&self.handle);
            Ok(Async::NotReady)
        }
    }
}

impl Drop for Interval {
    fn drop(&mut self) {
        self.token.cancel_timeout(&self.handle);
    }
}

fn duration_to_nanos(dur: Duration) -> Option<u64> {
    dur.as_secs()
        .checked_mul(1_000_000_000)
        .and_then(|v| v.checked_add(dur.subsec_nanos() as u64))
}

fn next_interval(prev: Instant, now: Instant, interval: Duration) -> Instant {
    let new = prev + interval;
    if new > now {
        return new;
    } else {
        let spent_ns = duration_to_nanos(now.duration_since(prev))
            .expect("interval should be expired");
        let interval_ns = duration_to_nanos(interval)
            .expect("interval is less that 427 thousand years");
        let mult = spent_ns / interval_ns + 1;
        assert!(mult < (1 << 32),
                "can't skip more than 4 billion intervals of {:?} (trying to skip {})",
                interval,
                mult);
        return prev + interval * (mult as u32);
    }
}

#[cfg(test)]
mod test {
    use std::time::{Instant, Duration};
    use super::next_interval;

    struct Timeline(Instant);

    impl Timeline {
        fn new() -> Timeline {
            Timeline(Instant::now())
        }
        fn at(&self, millis: u64) -> Instant {
            self.0 + Duration::from_millis(millis)
        }
        fn at_ns(&self, sec: u64, nanos: u32) -> Instant {
            self.0 + Duration::new(sec, nanos)
        }
    }

    fn dur(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn norm_next() {
        let tm = Timeline::new();
        assert_eq!(next_interval(tm.at(1), tm.at(2), dur(10)), tm.at(11));
        assert_eq!(next_interval(tm.at(7777), tm.at(7788), dur(100)),
                   tm.at(7877));
        assert_eq!(next_interval(tm.at(1), tm.at(1000), dur(2100)), tm.at(2101));
    }

    #[test]
    fn fast_forward() {
        let tm = Timeline::new();
        assert_eq!(next_interval(tm.at(1), tm.at(1000), dur(10)), tm.at(1001));
        assert_eq!(next_interval(tm.at(7777), tm.at(8888), dur(100)),
                   tm.at(8977));
        assert_eq!(next_interval(tm.at(1), tm.at(10000), dur(2100)),
                   tm.at(10501));
    }

    #[test]
    #[should_panic(expected = "can't skip more than 4 billion intervals")]
    fn large_skip() {
        let tm = Timeline::new();
        assert_eq!(next_interval(tm.at_ns(0, 1), tm.at_ns(25, 0), Duration::new(0, 2)),
                   tm.at_ns(25, 1));
    }

}
