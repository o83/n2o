use std::cmp;
use std::thread;
use bounded_spsc_queue as queue;
use num_cpus;

use util::hash::{HashMapFNV, Hasher};
use types::Symbol;
use orderbook::Book;
use orderbook::event::{Event, HasHeader};

pub enum Op {
    Quit,
    Apply(Symbol, Event),
    Observe(Symbol, Box<Fn(&Book) + Send>),
}

type ConsumerQ = queue::Consumer<Op>;
type ProducerQ = queue::Producer<Op>;

pub struct Dispatcher {
    chunk: usize,
    executors: Vec<ProducerQ>,
}

#[derive(Debug, Clone)]
pub struct SymbolSlice(pub Vec<Symbol>);

#[inline]
fn ncores() -> usize {
    // reserve 1 core
    num_cpus::get() - 1
}

impl Dispatcher {
    pub fn new(symbols: SymbolSlice) -> Self {
        Self::with_capacity(symbols, 4096)
    }

    pub fn with_capacity(symbols: SymbolSlice, max: usize) -> Self {
        let mut executors = Vec::with_capacity(ncores());
        for syms in symbols.into_iter() {
            let (p, c) = queue::make(max);
            Executor::start(syms, c);
            executors.push(p);
        }
        Dispatcher {
            chunk: cmp::max(symbols.0.len() / ncores(), 1),
            executors: executors,
        }
    }

    fn executor(&self, sym: Symbol) -> &ProducerQ {
        // symbols start from 1
        &self.executors[(sym.value() - 1) as usize / self.chunk]
    }

    pub fn observe(&self, sym: Symbol, observer: Box<Fn(&Book) + Send>) {
        let q = self.executor(sym);
        q.push(Op::Observe(sym, observer))
    }

    pub fn dispatch(&self, e: Event) {
        let sym = e.header().symbol;
            let q = self.executor(sym);
            q.push(Op::Apply(sym, e))
    }
}

pub struct Executor {
    books: HashMapFNV<Symbol, Book>,
}

impl Executor {
    fn start<'a>(symbols: &'a [Symbol], q: ConsumerQ) {
        let mut books = HashMapFNV::with_capacity_and_hasher(symbols.len(), Hasher::default());
        for sym in symbols {
            books.insert(*sym, Book::new());
            ()
        }
        let mut exec = Executor { books: books };
        thread::spawn(move || {
            loop {
                match q.pop() {
                    Op::Quit => break,
                    Op::Observe(sym, observe) => {
                        if let Some(book) = exec.books.get(&sym) {
                            observe(book)
                        }
                    } 
                    Op::Apply(sym, msg) => exec.update(sym, msg),
                }
            }
        });
        ()
    }

    fn update(&mut self, sym: Symbol, e: Event) {
        if let Some(book) = self.books.get_mut(&sym) {
            match e {
                Event::Add(v) => book.add(&v),
                Event::Cancel(v) => book.cancel(&v),
                Event::Delete(v) => book.delete(&v),
                Event::Change(v) => book.change(&v),
                Event::ExecSize(v) => book.execsize(&v),
                // XXX What do we do with the execution price?
                Event::Exec(v) => book.exec(&v),
                Event::Replace(v) => book.replace(&v),
            }
        }
    }
}

pub struct SymbolSliceIter<'a> {
    symbols: &'a SymbolSlice,
    len: usize,
    rem: usize,
    chunk: usize,
    pos: usize,
}

impl<'a> Iterator for SymbolSliceIter<'a> {
    type Item = &'a [Symbol];

    fn next(&mut self) -> Option<&'a [Symbol]> {
        if self.pos >= self.len {
            return None;
        }
        let pos = self.pos;
        self.pos += self.chunk;
        if self.rem > 0 {
            self.pos += 1;
            self.rem -= 1
        }
        Some(&self.symbols.0[pos..self.pos])
    }
}

impl<'a> IntoIterator for &'a SymbolSlice {
    type Item = &'a [Symbol];
    type IntoIter = SymbolSliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let cores = ncores();
        let len = self.0.len();
        SymbolSliceIter {
            symbols: self,
            len: len,
            chunk: len / cores,
            rem: len % cores,
            pos: 0,
        }
    }
}

#[cfg(test)]
mod symbol_slice_iter {
    use quickcheck::{Arbitrary, Gen};
    use types::Symbol;
    use super::{SymbolSlice, ncores};

    impl Arbitrary for SymbolSlice {
        fn arbitrary<G: Gen>(g: &mut G) -> SymbolSlice {
            let mut v: Vec<Symbol> = Arbitrary::arbitrary(g);
            // symbols start from 1
            v.retain(|&i| i.value() != 0);
            SymbolSlice(v)
        }
    }

    quickcheck! {
    fn spread_load_evenly(symbols: SymbolSlice) -> bool {
        let chunks = symbols.into_iter().collect::<Vec<_>>();
        let len = symbols.0.len();
        let cores = ncores();
        let chunk_size = len / cores;
        let rem = len % cores;
        let match_cores = chunks.len() == cores;
        if len < cores {
            let less = chunks.len() <= cores;
            let all_one = chunks.iter().all(|&c| c.len() == 1);
            return less && all_one;
        } else if len % cores == 0 {
            let same_size = chunks.iter().all(|&c| c.len() == chunk_size);
            return same_size && match_cores;
        } else {
            let total = chunks.iter().fold(0, |sum, &c| sum + c.len());
            let ngreater = chunks.iter().fold(0, |sum, &c| {
                if c.len() == chunk_size + 1 {
                    sum + 1
                } else {
                    sum
                }
            });
            match_cores && total == len && ngreater == rem
        }
    }
    }
}
