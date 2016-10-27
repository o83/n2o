
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Sequence {
    value: Rc<Cell<usize>>
}

impl Sequence {
    pub fn new() -> Sequence {
        Sequence { value: Rc::new(Cell::new(0)) }
    }

    pub fn next(&self) -> usize {
        let id = self.value.get();

        self.value.set(id + 1);
        id
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Sequence::new()
    }
}
