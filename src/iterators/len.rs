#[derive(Copy, Clone)]
pub struct ParallelLen {
    pub maximal_len: usize,
    pub cost: f64,
    pub sparse: bool,
}

impl ParallelLen {
    pub fn left_cost(&self, mid: usize) -> ParallelLen {
        ParallelLen { maximal_len: mid,
                      cost: self.cost / 2.0,
                      sparse: self.sparse }
    }

    pub fn right_cost(&self, mid: usize) -> ParallelLen {
        ParallelLen { maximal_len: self.maximal_len - mid,
                      cost: self.cost / 2.0,
                      sparse: self.sparse }
    }
}

pub const THRESHOLD: f64 = 10. * 1024.0;
pub const FUNC_ADJUSTMENT: f64 = 1.05;
