#![feature(test)]
extern crate test;
extern crate kernel;

use kernel::streams::stack::*;

#[derive(Debug,PartialEq)]
struct Entry(u16, i64);

#[test]
pub fn stack() {
    let capacity = (!0 as u16) as usize;
    let mut stack: Stack<Entry> = Stack::with_capacity(capacity);

    assert!(stack.capacity() == capacity);
    assert!(stack.len() == 0);
    assert!(stack.num_frames() == 0);

    stack.push_frame();
    assert!(stack.capacity() == capacity);
    assert!(stack.len() == 0);

    stack.insert(Entry(1, 1));
    assert!(stack.len() == 1);
    assert!(stack.num_frames() == 1);

    stack.push_frame();
    stack.insert(Entry(2, 2));
    stack.insert(Entry(3, 3));
    stack.insert(Entry(4, 4));
    assert!(stack.capacity() == capacity);
    assert!(stack.len() == 4);
    assert!(stack.num_frames() == 2);
    assert_eq!(stack.get(|it| (*it).0 == 4).unwrap(), &Entry(4, 4));

    stack.pop_frame();
    assert_eq!(stack.get(|it| (*it).0 == 3), None);
    assert_eq!(stack.get(|it| (*it).0 == 2), None);
    assert!(stack.num_frames() == 1);
    assert!(stack.len() == 1);

    // assert_eq!(stack.pop_frame(), Ok(()));
    // assert_eq!(stack.pop_frame(), Err(Error::InvalidOperation));

    stack.push_frame();
    let items = [Entry(9, 9), Entry(6, 6), Entry(7, 7)];
    stack.insert_many(&items);

    assert_eq!(stack.get(|it| (*it).0 == 6).unwrap(), &Entry(6, 6));
    assert_eq!(stack.get(|it| (*it).0 == 1).unwrap(), &Entry(1, 1));
    println!("STACK2: {:?}", stack);
    stack.push_frame();
    stack.insert(Entry(2, 2));
    assert!(stack.num_frames() == 3);
    assert_eq!(stack.get(|it| (*it).0 == 1).unwrap(), &Entry(1, 1));

}