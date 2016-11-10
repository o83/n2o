use alloc::raw_vec::RawVec;
use core::ptr;
use core::mem::transmute;
use std::slice::{from_raw_parts, from_raw_parts_mut};

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

unsafe impl<T: Sync> Sync for RingBuffer<T> {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer() {
        let mut ring: RingBuffer<u64> = RingBuffer::with_capacity(3);
        assert!(ring.cap() == 4);

        unsafe {
            for i in 0..4 {
                ring.store(i, i as u64);
            }
            for i in 0..4 {
                let v = ring.get(i);
                assert!(i == (*v) as usize);
            }

            let arr = ring.get_slice(0, 4);
            assert!(arr[0] == 0);
            assert!(arr[3] == 3);

            let mut s = ring.get_slice_mut(0, 4);
            s[2] = 42;
            assert!(arr[2] == 42);
        }
    }

}
