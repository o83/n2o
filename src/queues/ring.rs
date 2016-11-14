use alloc::raw_vec::RawVec;
use core::ptr;
use core::mem::transmute;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::fmt;

#[repr(C)]
pub struct RingBuffer<T> {
    buffer: RawVec<T>,
    mask: usize,
}

impl<T> RingBuffer<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let adjusted = cap.next_power_of_two();
        RingBuffer {
            buffer: RawVec::with_capacity(adjusted),
            mask: adjusted - 1,
        }
    }

    pub fn from_raw_parts(ptr: *mut T, cap: usize) -> Self {
        RingBuffer {
            buffer: unsafe { RawVec::from_raw_parts(ptr, cap) },
            mask: cap - 1,
        }
    }

    #[inline]
    pub fn cap(&self) -> usize {
        self.buffer.cap()
    }

    #[inline]
    pub unsafe fn get(&self, pos: usize) -> &T {
        transmute(self.buffer.ptr().offset((pos & self.mask) as isize))
    }

    #[inline]
    pub unsafe fn get_slice(&self, pos: usize, len: usize) -> &[T] {
        transmute(from_raw_parts(self.buffer.ptr().offset((pos & self.mask) as isize), len))
    }

    #[inline]
    pub unsafe fn get_slice_mut(&self, pos: usize, len: usize) -> &mut [T] {
        transmute(from_raw_parts_mut(self.buffer.ptr().offset((pos & self.mask) as isize), len))
    }

    #[inline]
    pub unsafe fn take(&self, pos: usize) -> T {
        ptr::read(self.buffer.ptr().offset((pos & self.mask) as isize))
    }

    #[inline]
    pub unsafe fn store(&mut self, pos: usize, value: T) {
        ptr::write(self.buffer.ptr().offset((pos & self.mask) as isize), value);
    }
}

// unsafe impl<T: Sync> Sync for RingBuffer<T> {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_cap() {
        let mut ring: RingBuffer<u64> = RingBuffer::with_capacity(3);
        assert!(ring.cap() == 4);
    }

    #[test]
    fn test_ring_buffer_pow2_cap() {
        let mut ring: RingBuffer<u64> = RingBuffer::with_capacity(8);
        assert!(ring.cap() == 8);
    }

    #[test]
    fn test_ring_buffer_take() {
        let mut ring: RingBuffer<u64> = RingBuffer::with_capacity(4);
        unsafe {
            ring.store(0, 42u64);
            let v1 = ring.take(0);
            let v2 = ring.take(8);
            assert!(v1 == v2);
        }
    }

    #[test]
    fn test_ring_buffer_get() {
        let mut ring: RingBuffer<u64> = RingBuffer::with_capacity(4);
        unsafe {
            ring.store(0, 42u64);
            let v1 = ring.get(0);
            let v2 = ring.get(8);
            assert!(*v1 == *v2);
        }
    }

    #[test]
    fn test_ring_buffer_get_slice() {
        let mut ring: RingBuffer<u64> = RingBuffer::with_capacity(4);
        unsafe {
            ring.store(1, 42u64);
            ring.store(2, 43u64);
            ring.store(3, 44u64);

            let s1 = ring.get_slice(1, 3);
            let s2 = ring.get_slice(9, 3);
            assert_eq!(s1, &[42, 43, 44u64]);
            assert_eq!(s2, &[42, 43, 44u64]);
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn test_ring_buffer_get_slice_mut() {
        let mut ring: RingBuffer<u64> = RingBuffer::with_capacity(4);
        unsafe {
            ring.store(1, 42u64);
            ring.store(2, 43u64);
            ring.store(3, 44u64);

            let s1 = ring.get_slice_mut(1, 3);
            let s2 = ring.get_slice(9, 3);
            s1[0] = 45u64;
            assert_eq!(s1, &[45, 43, 44u64]);
            assert_eq!(s2, &[45, 43, 44u64]);
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn test_ring_buffer_from_raw_parts() {
        use std::ptr;
        use std::mem;
        let mut arr = vec![0u64, 1, 2, 3];
        let mut ring: RingBuffer<u64> = RingBuffer::from_raw_parts(arr.as_mut_ptr(), arr.len());
        unsafe {
            mem::forget(arr);

            ring.store(1, 42u64);
            ring.store(2, 43u64);
            ring.store(3, 44u64);

            let s1 = ring.get_slice_mut(1, 3);
            let s2 = ring.get_slice(9, 3);

            s1[0] = 45u64;

            assert_eq!(s1, &[45, 43, 44u64]);
            assert_eq!(s2, &[45, 43, 44u64]);
            assert_eq!(s1, s2);
        }
    }

}
