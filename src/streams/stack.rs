
#[derive(Debug, PartialEq)]
pub enum Error {
    Capacity,
    InvalidOperation,
}
#[derive(Debug)]
pub struct Stack<T> {
    frames: Vec<usize>,
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn with_capacity(cap: usize, max_frames: usize) -> Stack<T> {
        Stack {
            frames: Vec::with_capacity(max_frames),
            items: Vec::with_capacity(cap),
        }
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.items.capacity() == self.items.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    pub fn push_frame(&mut self) -> Result<(), Error> {
        if self.is_full() {
            Err(Error::Capacity)
        } else {
            self.frames.push(self.items.len());
            Ok(())
        }
    }

    pub fn pop_frame(&mut self) -> Result<(), Error> {
        if self.items.is_empty() {
            Err(Error::InvalidOperation)
        } else {
            let frame_size = self.items.len() - *self.frames.last().unwrap();
            for i in 0..frame_size {
                self.items.pop();
            }
            self.frames.pop();
            Ok(())
        }
    }

    pub fn add_item(&mut self, item: T) -> Result<(), Error> {
        if self.is_full() {
            Err(Error::Capacity)
        } else {
            self.items.push(item);
            Ok(())
        }
    }

    // find_item(|item| (*item).key == 14)
    pub fn find_item<'a, F>(&'a self, mut f: F) -> Option<&T>
        where F: FnMut(&'a T) -> bool
    {
        for i in self.items.iter().rev() {
            if f(i) {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frames_stack() {
        let mut stack: Stack<u64> = Stack::with_capacity(128, 4);

        assert!(stack.capacity() == 128);
        assert!(stack.len() == 0);
        assert!(stack.num_frames() == 0);

        stack.push_frame();
        assert!(stack.capacity() == 128);
        assert!(stack.len() == 0);

        stack.add_item(41);
        assert!(stack.len() == 1);
        assert!(stack.num_frames() == 1);

        stack.push_frame();
        stack.add_item(42);
        stack.add_item(43);
        stack.add_item(44);
        assert!(stack.capacity() == 128);
        assert!(stack.len() == 4);
        assert!(stack.num_frames() == 2);
        assert_eq!(stack.find_item(|it| *it == 43), Some(&43));

        stack.pop_frame();
        assert_eq!(stack.find_item(|it| *it == 43), None);
        assert_eq!(stack.find_item(|it| *it == 41), Some(&41));
        assert!(stack.num_frames() == 1);
        assert!(stack.len() == 1);

        assert_eq!(stack.pop_frame(), Ok(()));
        assert_eq!(stack.pop_frame(), Err(Error::InvalidOperation));
    }
}