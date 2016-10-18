#[macro_use]
extern crate kernel;

use kernel::abstractions::futures::future;
use kernel::abstractions::futures::failed::failed;
use kernel::abstractions::futures::finished::finished;
use kernel::abstractions::futures::future::Future;
use kernel::abstractions::poll::{Poll};
use kernel::abstractions::queues::channel;
use kernel::abstractions::queues::channel::{Receiver};
use kernel::abstractions::streams::stream::*;
use kernel::abstractions::streams::iter::*;

mod support;
use support::*;


fn list() -> Receiver<i32, u32> {
    let (tx, rx) = channel::create();
    tx.send(Ok(1))
      .and_then(|tx| tx.send(Ok(2)))
      .and_then(|tx| tx.send(Ok(3)))
      .forget();
    rx
}

fn err_list() -> Receiver<i32, u32> {
    let (tx, rx) = channel::create();
    tx.send(Ok(1))
      .and_then(|tx| tx.send(Ok(2)))
      .and_then(|tx| tx.send(Err(3)))
      .forget();
    rx
}

#[test]
fn map() {
    assert_done(|| list().map(|a| a + 1).collect(), Ok(vec![2, 3, 4]));
}

#[test]
fn map_err() {
    assert_done(|| err_list().map_err(|a| a + 1).collect(), Err(4));
}

#[test]
fn fold() {
    assert_done(|| list().fold(0, |a, b| finished::<i32, u32>(a + b)), Ok(6));
    assert_done(|| err_list().fold(0, |a, b| finished::<i32, u32>(a + b)), Err(3));
}

#[test]
fn filter() {
    assert_done(|| list().filter(|a| *a % 2 == 0).collect(), Ok(vec![2]));
}

#[test]
fn filter_map() {
    assert_done(|| list().filter_map(|x| {
        if x % 2 == 0 {
            Some(x + 10)
        } else {
            None
        }
    }).collect(), Ok(vec![12]));
}

#[test]
fn and_then() {
    assert_done(|| list().and_then(|a| Ok(a + 1)).collect(), Ok(vec![2, 3, 4]));
    assert_done(|| list().and_then(|a| failed::<i32, u32>(a as u32)).collect(),
                Err(1));
}

#[test]
fn then() {
    assert_done(|| list().then(|a| a.map(|e| e + 1)).collect(), Ok(vec![2, 3, 4]));

}

#[test]
fn or_else() {
    assert_done(|| err_list().or_else(|a| {
        finished::<i32, u32>(a as i32)
    }).collect(), Ok(vec![1, 2, 3]));
}

#[test]
fn flatten() {
    assert_done(|| list().map(|_| list()).flatten().collect(),
                Ok(vec![1, 2, 3, 1, 2, 3, 1, 2, 3]));

}

#[test]
fn skip() {
    assert_done(|| list().skip(2).collect(), Ok(vec![3]));
}

#[test]
fn skip_passes_errors_through() {
    let mut s = iter(vec![Err(1), Err(2), Ok(3), Ok(4), Ok(5)])
        .skip(1)
        .wait();
    assert_eq!(s.next(), Some(Err(1)));
    assert_eq!(s.next(), Some(Err(2)));
    assert_eq!(s.next(), Some(Ok(4)));
    assert_eq!(s.next(), Some(Ok(5)));
    assert_eq!(s.next(), None);
}

#[test]
fn take() {
    assert_done(|| list().take(2).collect(), Ok(vec![1, 2]));
}

#[test]
fn take_passes_errors_through() {
    let mut s = iter(vec![Err(1), Err(2), Ok(3), Ok(4), Err(4)])
        .take(1)
        .wait();
    assert_eq!(s.next(), Some(Err(1)));
    assert_eq!(s.next(), Some(Err(2)));
    assert_eq!(s.next(), Some(Ok(3)));
    assert_eq!(s.next(), None);

    let mut s = iter(vec![Ok(1), Err(2)]).take(1).wait();
    assert_eq!(s.next(), Some(Ok(1)));
    assert_eq!(s.next(), None);
}



#[test]
fn wait() {
    assert_eq!(list().wait().collect::<Result<Vec<_>, _>>(),
               Ok(vec![1, 2, 3]));
}
