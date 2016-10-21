use alloc::heap::{allocate, deallocate};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::usize;
use std::sync::Arc;
use std::cell::Cell;
use core::{mem, ptr};
use core::mem::transmute;

pub struct Consumer<T> { buffer: Arc<Buffer<T>> }
pub struct Producer<T> { buffer: Arc<Buffer<T>> }

#[repr(C)]
pub struct Buffer<T> {
    buffer:         *mut T,
    capacity:       usize,
    allocated_size: usize,
    _padding1:      [u64; 1],

    head:           AtomicUsize,
    shadow_tail:    Cell<usize>,
    _padding2:      [u64; 2],

    tail:           AtomicUsize,
    shadow_head:    Cell<usize>,
    _padding3:      [u64; 2],
}

unsafe impl<T: Sync> Sync for Buffer<T> { }
unsafe impl<T: Send> Send for Consumer<T> { }
unsafe impl<T: Send> Send for Producer<T> { }

impl<T> !Sync for Consumer<T> {}
impl<T> !Sync for Producer<T> {}

impl<T> Buffer<T> {

    pub fn try_pop(&self) -> Option<T> {
        let current_head = self.head.load(Ordering::Relaxed);

        if current_head == self.shadow_tail.get() {
            self.shadow_tail.set(self.tail.load(Ordering::Acquire));
            if current_head == self.shadow_tail.get() {
                return None;
            }
        }

        let v = unsafe { ptr::read(self.load(current_head)) };
        self.head.store(current_head.wrapping_add(1), Ordering::Release);
        Some(v)
    }

    pub fn pop(&self) -> T {
        loop {
            match self.try_pop()  {
                None => {},
                Some(v) => return v
            }
        }
    }

    pub fn try_push(&self, v: T) -> Option<T> {
        let current_tail = self.tail.load(Ordering::Relaxed);

        if self.shadow_head.get() + self.capacity <= current_tail {
            self.shadow_head.set(self.head.load(Ordering::Relaxed));
            if self.shadow_head.get() + self.capacity <= current_tail {
                return Some(v);
            }
        }

        unsafe { self.store(current_tail, v); }
        self.tail.store(current_tail.wrapping_add(1), Ordering::Release);
        None
    }

    pub fn push(&self, v: T) {
        let mut t = v;
        loop {
            match self.try_push(t) {
                Some(rv) => t = rv,
                None => return
            }
        }
    }

    #[inline]
    unsafe fn load(&self, pos: usize) -> &T {
        transmute(self.buffer.offset((pos & (self.allocated_size - 1)) as isize))
    }

    #[inline]
    unsafe fn store(&self, pos: usize, v: T) {
        let end = self.buffer.offset((pos & (self.allocated_size - 1)) as isize);
        ptr::write(&mut *end, v);
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {

        loop {
            match self.try_pop() {
                Some(_) => {},  // Got a value, keep poppin!
                None => break   // All done, deallocate mem now
            }
        }

        unsafe {
            deallocate(self.buffer as *mut u8,
                self.allocated_size * mem::size_of::<T>(),
                mem::align_of::<T>());
        }
    }
}

pub fn create<T>(capacity: usize) -> (Producer<T>, Consumer<T>) {

    let ptr = unsafe { allocate_buffer(capacity) };

    let arc = Arc::new(Buffer{
        buffer: ptr,
        capacity: capacity,
        allocated_size: capacity.next_power_of_two(),
        _padding1:      [0; 1],

        head:           AtomicUsize::new(0),
        shadow_tail:    Cell::new(0),
        _padding2:      [0; 2],

        tail:           AtomicUsize::new(0),
        shadow_head:    Cell::new(0),
        _padding3:      [0; 2],
    });

    (Producer { buffer: arc.clone() }, Consumer { buffer: arc.clone() })
}

unsafe fn allocate_buffer<T>(capacity: usize) -> *mut T {
    let adjusted_size = capacity.next_power_of_two();
    let size = adjusted_size.checked_mul(mem::size_of::<T>()).expect("capacity overflow");
    let ptr = allocate(size, mem::align_of::<T>()) as *mut T;
    if ptr.is_null() { ::alloc::oom() }
    ptr
}

impl<T> Producer<T> {

    pub fn push(&self, v: T) {
        (*self.buffer).push(v);
    }

    pub fn try_push(&self, v: T) -> Option<T> {
        (*self.buffer).try_push(v)
    }

    pub fn capacity(&self) -> usize {
        (*self.buffer).capacity
    }

    pub fn size(&self) -> usize {
        (*self.buffer).tail.load(Ordering::Acquire) - (*self.buffer).head.load(Ordering::Acquire)
    }

    pub fn free_space(&self) -> usize {
        self.capacity() - self.size()
    }

}

impl<T> Consumer<T> {

    pub fn pop(&self) -> T {
        (*self.buffer).pop()
    }

    pub fn try_pop(&self) -> Option<T> {
        (*self.buffer).try_pop()
    }

    pub fn capacity(&self) -> usize {
        (*self.buffer).capacity
    }

    pub fn size(&self) -> usize {
        (*self.buffer).tail.load(Ordering::Acquire) - (*self.buffer).head.load(Ordering::Acquire)
    }

}



#[cfg(test)]
mod tests {

    use abstractions::queues::link;
    use std::thread;

    #[test]
    fn test_producer_push() {
        let (p, _) = link::create(10);

        for i in 0..9 {
            p.push(i);
            assert!(p.capacity() == 10);
            assert!(p.size() == i + 1);
        }
    }

    #[test]
    fn test_consumer_pop() {
        let (p, c) = link::create(10);

        for i in 0..9 {
            p.push(i);
            assert!(p.capacity() == 10);
            assert!(p.size() == i + 1);
        }

        for i in 0..9 {
            assert!(c.size() == 9 - i);
            let t = c.pop();
            assert!(c.capacity() == 10);
            assert!(c.size() == 9 - i - 1);
            assert!(t == i);
        }
    }

    #[test]
    fn test_try_push() {
        let (p, _) = link::create(10);

        for i in 0..10 {
            p.push(i);
            assert!(p.capacity() == 10);
            assert!(p.size() == i + 1);
        }

        match p.try_push(10) {
            Some(v) => {
                assert!(v == 10);
            },
            None => assert!(false, "Queue should not have accepted another write!")
        }
    }

    #[test]
    fn test_try_poll() {
        let (p, c) = link::create(10);

        match c.try_pop() {
            Some(_) => {
                assert!(false, "Queue was empty but a value was read!")
            },
            None => {}
        }

        p.push(123);

        match c.try_pop() {
            Some(v) => assert!(v == 123),
            None => assert!(false, "Queue was not empty but poll() returned nothing!")
        }

        match c.try_pop() {
            Some(_) => {
                assert!(false, "Queue was empty but a value was read!")
            },
            None => {}
        }
    }

    #[test]
    fn test_threaded() {
        let (p, c) = link::create(500);

        thread::spawn(move|| {
            for i in 0..100000 {
                p.push(i);
            }
        });

        for i in 0..100000 {
            let t = c.pop();
            assert!(t == i);
        }
    }

    #[cfg(feature = "benchmark")]
    fn bench_chan(b: &mut Bencher) {
        let (tx, rx) = sync_channel::<u8>(500);
        b.iter(|| {
            tx.send(1);
            rx.recv().unwrap()
        });
    }

    #[cfg(feature = "benchmark")]
    fn bench_chan_threaded(b: &mut Bencher) {
        let (tx, rx) = sync_channel::<u8>(500);
        let flag = AtomicBool::new(false);
        let arc_flag = Arc::new(flag);

        let flag_clone = arc_flag.clone();
        thread::spawn(move|| {
            while flag_clone.load(Ordering::Acquire) == false {
                // Try to do as much work as possible without checking the atomic
                for _ in 0..400 {
                    rx.recv().unwrap();
                }
            }
        });

        b.iter(|| {
            tx.send(1)
        });

        let flag_clone = arc_flag.clone();
        flag_clone.store(true, Ordering::Release);

        // We have to loop a minimum of 400 times to guarantee the other thread shuts down
        for _ in 0..400 {
            tx.send(1);
        }
    }

    #[cfg(feature = "benchmark")]
    fn bench_chan_threaded2(b: &mut Bencher) {
        let (tx, rx) = sync_channel::<u8>(500);
        let flag = AtomicBool::new(false);
        let arc_flag = Arc::new(flag);
        let flag_clone = arc_flag.clone();
        thread::spawn(move|| {
            while flag_clone.load(Ordering::Acquire) == false {
                // Try to do as much work as possible without checking the atomic
                for _ in 0..400 {
                    tx.send(1);
                }
            }
        });

        b.iter(|| {
            rx.recv().unwrap()
        });

        let flag_clone = arc_flag.clone();
        flag_clone.store(true, Ordering::Release);

        // We have to loop a minimum of 400 times to guarantee the other thread shuts down
        for _ in 0..400 {
            rx.try_recv();
        }
    }

    #[cfg(feature = "benchmark")]
    fn bench_spsc(b: &mut Bencher) {
        let (p, c) = super::make(500);

        b.iter(|| {
            p.push(1);
            c.pop()
        });
    }

    #[cfg(feature = "benchmark")]
    fn bench_spsc_threaded(b: &mut Bencher) {
        let (p, c) = link::create(500);
        let flag = AtomicBool::new(false);
        let arc_flag = Arc::new(flag);
        let flag_clone = arc_flag.clone();
        thread::spawn(move|| {
            while flag_clone.load(Ordering::Acquire) == false {

                // Try to do as much work as possible without checking the atomic
                for _ in 0..400 {
                    c.pop();
                }
            }
        });

        b.iter(|| {
            p.push(1)
        });

        let flag_clone = arc_flag.clone();
        flag_clone.store(true, Ordering::Release);

        // We have to loop a minimum of 400 times to guarantee the other thread shuts down
        for _ in 0..400 {
            p.try_push(1);
        }
    }

    #[cfg(feature = "benchmark")]
    fn bench_spsc_threaded2(b: &mut Bencher) {
        let (p, c) = link::create(500);

        let flag = AtomicBool::new(false);
        let arc_flag = Arc::new(flag);

        let flag_clone = arc_flag.clone();
        thread::spawn(move|| {
            while flag_clone.load(Ordering::Acquire) == false {

                // Try to do as much work as possible without checking the atomic
                for _ in 0..400 {
                    p.push(1);
                }
            }
        });

        b.iter(|| {
            c.pop()
        });

        let flag_clone = arc_flag.clone();
        flag_clone.store(true, Ordering::Release);

        // We have to loop a minimum of 400 times to guarantee the other thread shuts down
        for _ in 0..400 {
            c.try_pop();
        }
    }

    #[cfg(feature = "benchmark")]
    #[test]
    fn bench_spsc_throughput() {
        let iterations: i64 = 2i64.pow(14);

        let (p, c) = link::create(iterations as usize);

        let start = PreciseTime::now();
        for i in 0..iterations as usize {
            p.push(i);
        }
        let t = c.pop();
        assert!(t == 0);
        let end = PreciseTime::now();
        let throughput = (iterations as f64 / (start.to(end)).num_nanoseconds().unwrap() as f64) * 1000000000f64;
        println!("Spsc Throughput: {}/s -- (iterations: {} in {} ns)",
            throughput,
            iterations,
            (start.to(end)).num_nanoseconds().unwrap());


    }

    #[cfg(feature = "benchmark")]
    #[test]
    fn bench_chan_throughput() {
        let iterations: i64 = 2i64.pow(14);

        let (tx, rx) = sync_channel(iterations as usize);

        let start = PreciseTime::now();
        for i in 0..iterations as usize {
            tx.send(i);
        }
        let t = rx.recv().unwrap();
        assert!(t == 0);
        let end = PreciseTime::now();
        let throughput = (iterations as f64 / (start.to(end)).num_nanoseconds().unwrap() as f64) * 1000000000f64;
        println!("Chan Throughput: {}/s -- (iterations: {} in {} ns)",
            throughput,
            iterations,
            (start.to(end)).num_nanoseconds().unwrap());


    }

}
