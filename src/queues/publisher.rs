
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::cmp::min;
use std::usize::MAX;
use super::ring::RingBuffer;
use std::ffi::CString;
use std::intrinsics;
use std::fmt::Formatter;
use std::fmt::Debug;
use std::fmt;

type Sequence = usize;

#[derive(Debug)]
pub struct Cursor {
    padding0: [u64; 3],
    fail_items: Cell<u64>,
    fail_opers: Cell<u64>,
    succ_items: Cell<u64>,
    succ_opers: Cell<u64>,
    sequence: AtomicUsize,
    padding1: [u64; 7],
    cache: Cell<Sequence>,
    padding2: [u64; 7],
}

impl Cursor {

    pub fn new(value: Sequence) -> Self {
        Cursor {
            padding0: [0; 3],
            fail_items: Cell::new(0),
            fail_opers: Cell::new(0),
            succ_items: Cell::new(0),
            succ_opers: Cell::new(0),
            sequence: AtomicUsize::new(value),
            padding1: [0; 7],
            cache: Cell::new(0),
            padding2: [0; 7],
        }
    }

    #[inline]
    pub fn load(&self) -> Sequence {
        self.sequence.load(Ordering::Acquire)
    }

    #[inline]
    pub fn store(&self, seq: Sequence) {
        self.sequence.store(seq, Ordering::Release);
    }

    #[inline]
    pub fn get_cache(&self) -> Sequence {
        self.cache.get()
    }

    #[inline]
    pub fn set_cache(&self, seq: Sequence) {
        self.cache.set(seq);
    }
}

pub struct UncheckedUnsafeArc<T> {
    arc: Arc<UnsafeCell<T>>,
    data: *mut T,
}

impl<T: Send> UncheckedUnsafeArc<T> {

    fn new(data: T) -> UncheckedUnsafeArc<T> {
        let arc = Arc::new(UnsafeCell::new(data));
        let data = arc.get();
        UncheckedUnsafeArc {
            arc: arc,
            data: data,
        }
    }

    #[inline]
    unsafe fn get<'s>(&'s self) -> &'s mut T {
        &mut *self.data
    }

    #[inline]
    unsafe fn get_immut<'s>(&'s self) -> &'s T {
        &*self.data
    }
}

impl<T: Send> Clone for UncheckedUnsafeArc<T> {

    fn clone(&self) -> UncheckedUnsafeArc<T> {    
        UncheckedUnsafeArc {
            arc: self.arc.clone(),
            data: self.data,
        }        
    }
}


pub struct Publisher<T> {
    ring: Arc<RingBuffer<T>>,
    next_seq_cache: Cell<Sequence>,
    cursors: UncheckedUnsafeArc<Vec<Cursor>>,
}

impl<T> Publisher<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let mut cursors = vec![];
        cursors.push(Cursor::new(0));

        Publisher {
            ring: Arc::new(RingBuffer::with_capacity(cap)),
            next_seq_cache: Cell::new(0),
            cursors: UncheckedUnsafeArc::new(cursors),
        }
    }

    pub fn with_mirror(name: CString, cap: usize) -> Self {
        let mut cursors = vec![];
        cursors.push(Cursor::new(0));

        Publisher {
            ring: Arc::new(RingBuffer::with_mirror(name, cap).unwrap()),
            next_seq_cache: Cell::new(0),
            cursors: UncheckedUnsafeArc::new(cursors),
        }
    }

    pub fn subscribe(&mut self) -> Subscriber<T> {
        let head = self.head().load();
        unsafe { self.cursors.get().push(Cursor::new(head));}
        let token = self.cursors().len() - 1 ;
        Subscriber::<T>::new(self.ring.clone(), self.cursors.clone(), token)
    }

    pub fn next(&self) -> Option<&mut T> {
        self.next_n(1).map(|vs| &mut vs[0])
    }

    pub fn next_n(&self, n: usize) -> Option<&mut [T]> {
        let head = self.head();
        let cursors = self.cursors();
        let delta = n as Sequence;
        let curr_seq = head.load();
        let next_seq = curr_seq + delta;
        let cap = self.ring.cap();

        if head.get_cache() + cap < next_seq {
            let mut min_tail = MAX;
            for tail in cursors.iter().skip(1) {
                min_tail = min(min_tail, tail.load());
                head.set_cache(min_tail);
                if min_tail + cap < next_seq {
                    return None;
                }
            }
        }
        self.next_seq_cache.set(next_seq);
        let slice = unsafe { self.ring.get_slice_mut(curr_seq, n) };
        Some(slice)
    }

    pub fn commit(&self) {
        self.head().store(self.next_seq_cache.get());
    }

    #[inline]
    fn head(&self) -> &Cursor {
        unsafe { self.cursors().get_unchecked(0) }
    }

    #[inline]
    fn cursors(&self) -> &[Cursor] {
        unsafe { self.cursors.get_immut() }
    }

}

pub struct Subscriber<T> {
    ring: Arc<RingBuffer<T>>,
    token: usize,
    next_seq_cache: Cell<Sequence>,
    cursors: UncheckedUnsafeArc<Vec<Cursor>>,
}

impl<T> Debug for Subscriber<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let tail = self.tail(self.token);
        write!(f, "Subscriber {{ token: {}, fail_opers: {}, fail_items: {}, seq: {} }}", 
            self.token, 
            tail.fail_opers.get(),
            tail.fail_items.get(),
            self.next_seq_cache.get())
    }
}

unsafe impl<T: Send> Send for Subscriber<T> {}
unsafe impl<T: Send> Send for Publisher<T> {}

impl<T> !Sync for Subscriber<T> {}
impl<T> !Sync for Publisher<T> {}


impl<T> Clone for Subscriber<T> {
    #[inline]
    fn clone(&self) -> Subscriber<T> {
        let tail_seq = self.tail(self.token).load();
        unsafe { self.cursors.get().push(Cursor::new(tail_seq));}
        let token = self.cursors().len() - 1 ;
        Subscriber::<T>::new(self.ring.clone(), self.cursors.clone(), token)
    }
}

impl<T> Subscriber<T> {
    pub fn new(ring: Arc<RingBuffer<T>>,
               cursors: UncheckedUnsafeArc<Vec<Cursor>>,
               token: usize)
               -> Self {
        Subscriber::<T> {
            ring: ring,
            token: token,
            next_seq_cache: Cell::new(0),
            cursors: cursors,
        }
    }

    pub fn recv(&self) -> Option<&T> {
      self.recv_n(1).map(|vs| &vs[0])
    }

    pub fn recv_n(&self, n: usize) -> Option<&[T]> {

        let tail = self.tail(self.token);
        let head = self.tail(0);

        let tail_seq = tail.load();
        let delta = n as Sequence;
        let next_seq = tail_seq + delta;

        if next_seq > tail.get_cache() {
            tail.set_cache(head.load());
            if next_seq > tail.get_cache() {
                //tail.fail_items.set(tail.fail_items.get() + n as u64);
                //tail.fail_opers.set(tail.fail_opers.get() + 1 as u64);
                return None;
            }
        }

        self.next_seq_cache.set(next_seq);
        let slice = unsafe { self.ring.get_slice(tail_seq, n) };
        Some(slice)
    }

    pub fn recv_all(&self) -> Option<&[T]> {

        let tail = self.tail(self.token);
        let head = self.tail(0);

        let tail_seq = tail.load();
        let head_seq = head.load();

        if tail_seq >= head_seq {
            //tail.fail_items.set(tail.fail_items.get() + 1 as u64);
            //tail.fail_opers.set(tail.fail_opers.get() + 1 as u64);
            return None;
        } else {
            self.next_seq_cache.set(head_seq);
            let slice = unsafe { self.ring.get_slice(tail_seq, head_seq - tail_seq) };
            Some(slice)
        }
    }

    pub fn commit(&self) {
        self.tail(self.token).store(self.next_seq_cache.get());
    }

    #[inline]
    fn tail(&self, token: usize) -> &Cursor {
        unsafe { self.cursors().get_unchecked(token) }
    }

    #[inline]
    fn cursors(&self) -> &[Cursor] {
        unsafe { self.cursors.get_immut() }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::u64;
    use std::sync::mpsc::channel;

    #[test]
    fn test_publisher_cursor() {
        let mut c = Cursor::new(42);
        assert_eq!(c.load(), 42);
        assert_eq!(c.get_cache(), 0);
        c.set_cache(42);
        assert_eq!(c.get_cache(), 42);
        c.store(43);
        assert_eq!(c.load(), 43);
    }

    #[test]
    fn test_publisher_next() {
        let mut publisher: Publisher<u64> = Publisher::with_capacity(8);
        let subscriber = publisher.subscribe();
        
        for i in 0..8 {
            match publisher.next() {
                Some(v) => {
                    *v = i as u64;
                    publisher.commit();
                },
                None => {}
            }
        }

        match publisher.next() {
            Some(_) => assert!(false, "Queue should not have accepted another write!"),
            None => {}
        }
    }

    #[test]
    fn test_publisher_next_n() {
        let mut publisher: Publisher<u64> = Publisher::with_capacity(8);
        let subscriber = publisher.subscribe();
        
        for i in 0..4 {
            match publisher.next_n(2) {
                Some(vs) => {
                    vs[0] = 2*i as u64;
                    vs[1] = 2*i + 1 as u64;
                    publisher.commit();
                },
                None => {}
            }
        }

        match publisher.next_n(2) {
            Some(_) => assert!(false, "Queue should not have accepted another write!"),
            None => {}
        }
    }
    

    #[test]
    fn test_publisher_recv() {
        let mut publisher: Publisher<u64> = Publisher::with_capacity(8);
        let subscriber = publisher.subscribe();

        match subscriber.recv() {
            Some(_) => { assert!(false, "Queue was empty but a value was read!")},
            None => {}
        } 
        
        match publisher.next() {
            Some(v) => {
                        *v = 42u64;
                        publisher.commit();
                        },
            None => {}
        }

        match subscriber.recv() {
            Some(v) => {
                assert!(*v == 42);
                subscriber.commit();
            },
            None => assert!(false, "Queue was not empty but recv() returned nothing!")
        }

        match subscriber.recv() {
            Some(_) => {
                assert!(false, "Queue was empty but a value was read!")
            },
            None => {}
        }

    }

    #[test]
    fn test_publisher_recv_n() {
        let mut publisher: Publisher<u64> = Publisher::with_capacity(8);
        let subscriber = publisher.subscribe();
        
        for i in 0..4 {
            match publisher.next_n(2) {
                Some(vs) => {
                    vs[0] = 2*i as u64;
                    vs[1] = 2*i + 1 as u64;
                    publisher.commit();
                },
                None => {}
            }
        }

        for i in 0..4 {
            match subscriber.recv_n(2) {
                Some(vs) => {
                    assert!(vs[0] == 2*i as u64);
                    assert!(vs[1] == 2*i + 1 as u64);
                    subscriber.commit();
                },
                None => assert!(false, "Queue was not empty but recv() returned nothing!")
            }
        }
    }

    #[test]
    fn test_publisher_recv_all() {
        let mut publisher: Publisher<u64> = Publisher::with_capacity(8);
        let subscriber = publisher.subscribe();
        
        for i in 0..4 {
            match publisher.next_n(2) {
                Some(vs) => {
                    vs[0] = 2*i as u64;
                    vs[1] = 2*i + 1 as u64;
                    publisher.commit();
                },
                None => {}
            }
        }

        match subscriber.recv_all() {
            Some(vs) => {
                assert_eq!(vs, &[0u64, 1, 2, 3, 4, 5, 6, 7]);
                subscriber.commit();
            },
            None => assert!(false, "Queue was not empty but recv_all() returned nothing!")
        }

        match subscriber.recv() {
            Some(_) => {
                assert!(false, "Queue was empty but a value was read!")
            },
            None => {}
        }
    }

    #[test]
    fn test_publisher_one2one() {
        let mut publisher: Publisher<u64> = Publisher::with_capacity(8);
        let subscriber = publisher.subscribe();
        
        thread::spawn(move|| {        
            for i in 0..4 {
                loop {
                    match publisher.next_n(2) {
                        Some(vs) => {
                            vs[0] = 2*i as u64;
                            vs[1] = 2*i + 1 as u64;
                            publisher.commit();
                            break;
                        },
                        None => {}
                    }
                }
            }
        });

        for i in 0..4 {
            loop {
                match subscriber.recv_n(2) {
                    Some(vs) => {
                        assert!(vs[0] == 2*i as u64);
                        assert!(vs[1] == 2*i + 1 as u64);
                        subscriber.commit();
                        break;
                    },
                    None => {}
                }
            }
        }
    }

    #[test]
    fn test_publisher_one2n() {
        let mut publisher: Publisher<u64> = Publisher::with_capacity(8);
        let (tx, rx) = channel::<u64>();
        for t in 0..4 {
            let subscriber = publisher.subscribe();
            let tx_c = tx.clone();       
            thread::spawn(move|| {
                let mut expected = 0u64; 
                'outer: loop {
                    'inner: loop {
                        match subscriber.recv() {
                            Some(v) => {
                                if *v == u64::MAX {
                                    let _ = tx_c.send(*v);
                                    subscriber.commit();
                                    break 'outer;
                                } 
                                assert!(*v == expected);
                                expected += 1;
                                subscriber.commit();
                                break 'inner;
                            },
                            None => {}
                        }
                    }
                }
            });            
        }

        for i in 0..8 {
            loop {
                match publisher.next() {
                    Some(v) => {
                        *v = i as u64;
                        publisher.commit();
                        break;
                    },
                    None => {}
                }
            }
        }

       loop {
            match publisher.next() {
                Some(v) => {
                    *v = u64::MAX;
                    publisher.commit();
                    break;
                },
                None => {}
            }
        }

        for i in 0..4 {
            let _ = rx.recv(); //wait for readers
        }
    }

}
