use std::cell::UnsafeCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

pub struct UnparkMutex<D> {
    status: AtomicUsize,
    inner: UnsafeCell<Option<D>>,
}

unsafe impl<D: Send> Send for UnparkMutex<D> {}
unsafe impl<D: Send> Sync for UnparkMutex<D> {}

const WAITING: usize = 0;       // --> POLLING
const POLLING: usize = 1;       // --> WAITING, REPOLL, or COMPLETE
const REPOLL: usize = 2;        // --> POLLING
const COMPLETE: usize = 3;      // No transitions out

impl<D> UnparkMutex<D> {
    pub fn new() -> UnparkMutex<D> {
        UnparkMutex {
            status: AtomicUsize::new(WAITING),
            inner: UnsafeCell::new(None),
        }
    }

    pub fn notify(&self) -> Result<D, ()> {
        let mut status = self.status.load(SeqCst);
        loop {
            match status {
                WAITING => {
                    match self.status.compare_exchange(WAITING, POLLING,
                                                       SeqCst, SeqCst) {
                        Ok(_) => {
                            let data = unsafe {
                                (*self.inner.get()).take().unwrap()
                            };
                            return Ok(data);
                        }
                        Err(cur) => status = cur,
                    }
                }

                POLLING => {
                    match self.status.compare_exchange(POLLING, REPOLL,
                                                       SeqCst, SeqCst) {
                        Ok(_) => return Err(()),
                        Err(cur) => status = cur,
                    }
                }

                _ => return Err(()),
            }
        }
    }

    pub unsafe fn start_poll(&self) {
        self.status.store(POLLING, SeqCst);
    }

    pub unsafe fn wait(&self, data: D) -> Result<(), D> {
        *self.inner.get() = Some(data);

        match self.status.compare_exchange(POLLING, WAITING, SeqCst, SeqCst) {
            // no unparks came in while we were running
            Ok(_) => Ok(()),

            Err(status) => {
                assert_eq!(status, REPOLL);
                self.status.store(POLLING, SeqCst);
                Err((*self.inner.get()).take().unwrap())
            }
        }
    }

    pub unsafe fn complete(&self) {
        self.status.store(COMPLETE, SeqCst);
    }
}
