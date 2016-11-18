
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::cmp::min;
use std::usize::MAX;
use super::ring::RingBuffer;
use std::ffi::CString;

type Sequence = usize;

#[derive(Debug)]
pub struct Cursor {
    _padding0: [u64; 7],
    sequence: AtomicUsize,
    _padding1: [u64; 7],
    cache: Cell<Sequence>,
    _padding2: [u64; 7],
}

impl Cursor {
    pub fn new(value: Sequence) -> Self {
        Cursor {
            _padding0: [0; 7],
            sequence: AtomicUsize::new(value as usize),
            _padding1: [0; 7],
            cache: Cell::new(0usize),
            _padding2: [0; 7],
        }
    }
    #[inline]
    pub fn get_seq(&self) -> Sequence {
        self.sequence.load(Ordering::Relaxed) as Sequence
    }
    #[inline]
    pub fn set_seq(&self, seq: Sequence) {
        self.sequence.store(seq as usize, Ordering::Release);
    }
    #[inline]
    pub fn get_cache(&self) -> Sequence {
        self.cache.get() as Sequence
    }
    #[inline]
    pub fn set_cache(&self, seq: Sequence) {
        self.cache.set(seq as usize);
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
    unsafe fn get<'s>(&'s mut self) -> &'s mut T {
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


pub struct Enso<T> {
    ring: Arc<RingBuffer<T>>,
    _padding1: [u64; 7],
    next_seq_cache: Cell<Sequence>,
    _padding2: [u64; 7],
    cursors: UncheckedUnsafeArc<Vec<Cursor>>,
}

impl<T> Enso<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let mut cursors = vec![];
        cursors.push(Cursor::new(0));

        Enso {
            ring: Arc::new(RingBuffer::with_capacity(cap)),
            _padding1: [0; 7],
            next_seq_cache: Cell::new(0),
            _padding2: [0; 7],
            cursors: UncheckedUnsafeArc::new(cursors),
        }
    }

    pub fn with_mirror(name: CString, cap: usize) -> Self {
        let mut cursors = vec![];
        cursors.push(Cursor::new(0));

        Enso {
            ring: Arc::new(RingBuffer::with_mirror(name, cap).unwrap()),
            _padding1: [0; 7],
            next_seq_cache: Cell::new(0),
            _padding2: [0; 7],
            cursors: UncheckedUnsafeArc::new(cursors),
        }
    }

    pub fn new_consumer(&mut self) -> Consumer<T> {
        unsafe {
            self.cursors.get().push(Cursor::new(0));
        }
        let token = unsafe { self.cursors.get_immut().len() - 1 };
        Consumer::<T>::new(self.ring.clone(), self.cursors.clone(), token)
    }

    pub fn next(&self) -> Option<&mut T> {
        self.next_n(1).map(|vs| &mut vs[0])
    }

    pub fn next_n(&self, n: usize) -> Option<&mut [T]> {
        let ref cursors = unsafe { self.cursors.get_immut().as_slice() };
        let ref prod_cursor = cursors[0];
        let delta = n as Sequence;
        let current_pos = prod_cursor.get_seq();
        let next_seq = current_pos + delta;
        let cap = self.ring.cap();

        if prod_cursor.get_cache() + cap < next_seq {
            let mut min_cons = MAX;
            for cons in cursors.iter().skip(1) {
                min_cons = min(min_cons, cons.get_seq());
                prod_cursor.set_cache(min_cons);
                if min_cons + cap < next_seq {
                    return None;
                }
            }
        }
        self.next_seq_cache.set(next_seq);
        let slice = unsafe { self.ring.get_slice_mut(current_pos, n) };
        Some(slice)
    }

    pub fn flush(&self) {
        let ref cursors = unsafe { self.cursors.get_immut().as_slice() };
        let ref prod_cursor = cursors[0];
        prod_cursor.set_seq(self.next_seq_cache.get());
    }
}

pub struct Consumer<T> {
    ring: Arc<RingBuffer<T>>,
    token: usize,
    next_seq_cache: Cell<Sequence>,
    cursors: UncheckedUnsafeArc<Vec<Cursor>>,
}

unsafe impl<T: Send> Send for Consumer<T> {}
unsafe impl<T: Send> Send for Enso<T> {}

impl<T> !Sync for Consumer<T> {}
impl<T> !Sync for Enso<T> {}


impl<T> Consumer<T> {
    pub fn new(ring: Arc<RingBuffer<T>>,
               cursors: UncheckedUnsafeArc<Vec<Cursor>>,
               token: usize)
               -> Self {
        Consumer::<T> {
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
        let ref cursors = unsafe { self.cursors.get_immut().as_slice() };

        let ref cons_cursor = cursors[self.token];
        let ref prod_cursor = cursors[0];

        let consumer_pos = cons_cursor.get_seq();
        let delta = n as Sequence;
        let next_seq = consumer_pos + delta;

        if next_seq > cons_cursor.get_cache() {
            cons_cursor.set_cache(prod_cursor.get_seq());
            if next_seq > cons_cursor.get_cache() {
                return None;
            }
        }

        self.next_seq_cache.set(next_seq);
        let slice = unsafe { self.ring.get_slice(consumer_pos, n) };
        Some(slice)
    }

    pub fn recv_all(&self) -> Option<&[T]> {
        let ref cursors = unsafe { self.cursors.get_immut().as_slice() };

        let ref cons_cursor = cursors[self.token];
        let ref prod_cursor = cursors[0];

        let consumer_pos = cons_cursor.get_seq();
        let producer_pos = prod_cursor.get_seq();

        if consumer_pos >= producer_pos {
            return None;
        } else {
            self.next_seq_cache.set(producer_pos);
            let slice = unsafe { self.ring.get_slice(consumer_pos, producer_pos - consumer_pos) };
            Some(slice)
        }
    }

    pub fn release(&self) {
        let ref cursors = unsafe { self.cursors.get_immut().as_slice() };
        let ref cons_cursor = cursors[self.token];
        cons_cursor.set_seq(self.next_seq_cache.get());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::u64;
    use std::sync::mpsc::channel;

    #[test]
    fn test_enso_cursor() {
        let mut c = Cursor::new(42);
        assert_eq!(c.get_seq(), 42);
        assert_eq!(c.get_cache(), 0);
        c.set_cache(42);
        assert_eq!(c.get_cache(), 42);
        c.set_seq(43);
        assert_eq!(c.get_seq(), 43);
    }

    #[test]
    fn test_enso_next() {
        let mut enso: Enso<u64> = Enso::with_capacity(8);
        let cons = enso.new_consumer();
        
        for i in 0..8 {
            match enso.next() {
                Some(v) => {
                    *v = i as u64;
                    enso.flush();
                },
                None => {}
            }
        }

        match enso.next() {
            Some(_) => assert!(false, "Queue should not have accepted another write!"),
            None => {}
        }
    }

    #[test]
    fn test_enso_next_n() {
        let mut enso: Enso<u64> = Enso::with_capacity(8);
        let cons = enso.new_consumer();
        
        for i in 0..4 {
            match enso.next_n(2) {
                Some(vs) => {
                    vs[0] = 2*i as u64;
                    vs[1] = 2*i + 1 as u64;
                    enso.flush();
                },
                None => {}
            }
        }

        match enso.next_n(2) {
            Some(_) => assert!(false, "Queue should not have accepted another write!"),
            None => {}
        }
    }
    

    #[test]
    fn test_enso_recv() {
        let mut enso: Enso<u64> = Enso::with_capacity(8);
        let cons = enso.new_consumer();

        match cons.recv() {
            Some(_) => { assert!(false, "Queue was empty but a value was read!")},
            None => {}
        } 
        
        match enso.next() {
            Some(v) => {
                        *v = 42u64;
                        enso.flush();
                        },
            None => {}
        }

        match cons.recv() {
            Some(v) => {
                assert!(*v == 42);
                cons.release();
            },
            None => assert!(false, "Queue was not empty but recv() returned nothing!")
        }

        match cons.recv() {
            Some(_) => {
                assert!(false, "Queue was empty but a value was read!")
            },
            None => {}
        }

    }

    #[test]
    fn test_enso_recv_n() {
        let mut enso: Enso<u64> = Enso::with_capacity(8);
        let cons = enso.new_consumer();
        
        for i in 0..4 {
            match enso.next_n(2) {
                Some(vs) => {
                    vs[0] = 2*i as u64;
                    vs[1] = 2*i + 1 as u64;
                    enso.flush();
                },
                None => {}
            }
        }

        for i in 0..4 {
            match cons.recv_n(2) {
                Some(vs) => {
                    assert!(vs[0] == 2*i as u64);
                    assert!(vs[1] == 2*i + 1 as u64);
                    cons.release();
                },
                None => assert!(false, "Queue was not empty but recv() returned nothing!")
            }
        }
    }

    #[test]
    fn test_enso_recv_all() {
        let mut enso: Enso<u64> = Enso::with_capacity(8);
        let cons = enso.new_consumer();
        
        for i in 0..4 {
            match enso.next_n(2) {
                Some(vs) => {
                    vs[0] = 2*i as u64;
                    vs[1] = 2*i + 1 as u64;
                    enso.flush();
                },
                None => {}
            }
        }

        match cons.recv_all() {
            Some(vs) => {
                assert_eq!(vs, &[0u64, 1, 2, 3, 4, 5, 6, 7]);
                cons.release();
            },
            None => assert!(false, "Queue was not empty but recv_all() returned nothing!")
        }

        match cons.recv() {
            Some(_) => {
                assert!(false, "Queue was empty but a value was read!")
            },
            None => {}
        }
    }

    #[test]
    fn test_enso_one2one() {
        let mut enso: Enso<u64> = Enso::with_capacity(8);
        let cons = enso.new_consumer();
        
        thread::spawn(move|| {        
            for i in 0..4 {
                loop {
                    match enso.next_n(2) {
                        Some(vs) => {
                            vs[0] = 2*i as u64;
                            vs[1] = 2*i + 1 as u64;
                            enso.flush();
                            break;
                        },
                        None => {}
                    }
                }
            }
        });

        for i in 0..4 {
            loop {
                match cons.recv_n(2) {
                    Some(vs) => {
                        assert!(vs[0] == 2*i as u64);
                        assert!(vs[1] == 2*i + 1 as u64);
                        cons.release();
                        break;
                    },
                    None => {}
                }
            }
        }
    }

    #[test]
    fn test_enso_one2n() {
        let mut enso: Enso<u64> = Enso::with_capacity(8);
        let (tx, rx) = channel::<u64>();
        for t in 0..4 {
            let cons = enso.new_consumer();
            let tx_c = tx.clone();       
            thread::spawn(move|| {
                let mut expected = 0u64; 
                'outer: loop {
                    'inner: loop {
                        match cons.recv() {
                            Some(v) => {
                                if *v == u64::MAX {
                                    let _ = tx_c.send(*v);
                                    cons.release();
                                    break 'outer;
                                } 
                                assert!(*v == expected);
                                expected += 1;
                                cons.release();
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
                match enso.next() {
                    Some(v) => {
                        *v = i as u64;
                        enso.flush();
                        break;
                    },
                    None => {}
                }
            }
        }

       loop {
            match enso.next() {
                Some(v) => {
                    *v = u64::MAX;
                    enso.flush();
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
