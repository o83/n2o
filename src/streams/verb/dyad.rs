use core::iter::{empty, once, repeat};

struct DyadIterator<L, R, F> {
    l: L,
    r: R,
    f: F,
}

impl<L, R, A, B, C, F> Iterator for DyadIterator<L, R, F>
    where L: Iterator<Item = A>,
          R: Iterator<Item = B>,
          F: FnMut(A, B) -> C
{
    type Item = C;

    fn next(&mut self) -> Option<C> {
        match (self.l.next(), self.r.next()) {
            (Some(a), Some(b)) => Some((self.f)(a, b)),
            _ => None,
        }
    }
}

fn apply<L, R, A, B, C, F>(lhs: L, rhs: R, f: F) -> DyadIterator<L, R, F>
    where F: Fn(A, B) -> C,
          L: Iterator<Item = A>,
          R: Iterator<Item = B>
{
    DyadIterator {
        l: lhs,
        r: rhs,
        f: f,
    }
}

// let res1: Vec<_> = apply(once(1), once(2), Add::add).collect();
// let res2: Vec<_> = apply(vec![1, 2, 3, 4].into_iter(), repeat(2), Mul::mul).collect();
// let res3: Vec<_> = apply(repeat(1), vec![1, 2, 3, 4].into_iter(), Add::add).collect();
// let res4: Vec<_> = apply(vec![1, 2, 3, 4].into_iter(), vec![1, 2, 3, 4].into_iter(), Add::add).collect();