
use iterators::runner::api::*;
use iterators::iterators::IndexedParallelIterator;
use iterators::len::*;
use iterators::runner::thread_pool::get_registry;

pub trait ProducerCallback<ITEM> {
    type Output;
    fn callback<P>(self, producer: P) -> Self::Output
        where P: Producer<Item=ITEM>;
}

pub trait Producer: IntoIterator + Send + Sized {
    fn weighted(&self) -> bool { false }
    fn cost(&mut self, len: usize) -> f64;
    fn split_at(self, index: usize) -> (Self, Self);
}

pub trait Consumer<Item>: Send + Sized {
    type Folder: Folder<Item, Result=Self::Result>;
    type Reducer: Reducer<Self::Result>;
    type Result: Send;

    fn weighted(&self) -> bool { false }
    fn cost(&mut self, producer_cost: f64) -> f64;
    fn split_at(self, index: usize) -> (Self, Self, Self::Reducer);
    fn into_folder(self) -> Self::Folder;
    fn full(&self) -> bool { false }
}

pub trait Folder<Item> {
    type Result;
    fn consume(self, item: Item) -> Self;
    fn complete(self) -> Self::Result;
    fn full(&self) -> bool { false }
}

pub trait Reducer<Result> {
    fn reduce(self, left: Result, right: Result) -> Result;
}

pub trait UnindexedConsumer<ITEM>: Consumer<ITEM> {
    fn split_off(&self) -> Self;
    fn to_reducer(&self) -> Self::Reducer;
}

#[derive(Clone, Copy)]
enum Splitter {
    Cost(f64),
    Thief(usize, usize),
}

impl Splitter {
    #[inline]
    fn thief_id() -> usize {
        // The actual `ID` value is irrelevant.  We're just using its TLS
        // address as a unique thread key, faster than a real thread-id call.
        thread_local!{ static ID: bool = false }
        ID.with(|id| id as *const bool as usize )
    }

    #[inline]
    fn new_thief() -> Splitter {
        Splitter::Thief(Splitter::thief_id(), get_registry().num_threads())
    }

    #[inline]
    fn try(&mut self) -> bool {
        match *self {
            Splitter::Cost(ref mut cost) => {
                if *cost > THRESHOLD {
                    *cost /= 2.0;
                    true
                } else { false }
            },

            Splitter::Thief(ref mut origin, ref mut splits) => {
                let id = Splitter::thief_id();
                if *origin != id {
                    *origin = id;
                    *splits = get_registry().num_threads();
                    true
                } else if *splits > 0 {
                    *splits /= 2;
                    true
                } else { false }
            }
        }
    }
}

pub fn bridge<PAR_ITER,C>(mut par_iter: PAR_ITER,
                             consumer: C)
                             -> C::Result
    where PAR_ITER: IndexedParallelIterator, C: Consumer<PAR_ITER::Item>
{
    let len = par_iter.len();
    return par_iter.with_producer(Callback { len: len,
                                             consumer: consumer, });

    struct Callback<C> {
        len: usize,
        consumer: C,
    }

    impl<C, ITEM> ProducerCallback<ITEM> for Callback<C>
        where C: Consumer<ITEM>
    {
        type Output = C::Result;
        fn callback<P>(mut self, mut producer: P) -> C::Result
            where P: Producer<Item=ITEM>
        {
            let splitter = if producer.weighted() || self.consumer.weighted() {
                let producer_cost = producer.cost(self.len);
                let cost = self.consumer.cost(producer_cost);
                Splitter::Cost(cost)
            } else {
                Splitter::new_thief()
            };
            bridge_producer_consumer(self.len, splitter, producer, self.consumer)
        }
    }
}

fn bridge_producer_consumer<P,C>(len: usize,
                                 mut splitter: Splitter,
                                 producer: P,
                                 consumer: C)
                                 -> C::Result
    where P: Producer, C: Consumer<P::Item>
{
    if consumer.full() {
        consumer.into_folder().complete()
    } else if len > 1 && splitter.try() {
        let mid = len / 2;
        let (left_producer, right_producer) = producer.split_at(mid);
        let (left_consumer, right_consumer, reducer) = consumer.split_at(mid);
        let (left_result, right_result) =
            join(move || bridge_producer_consumer(mid, splitter,
                                                  left_producer, left_consumer),
                 move || bridge_producer_consumer(len - mid, splitter,
                                                  right_producer, right_consumer));
        reducer.reduce(left_result, right_result)
    } else {
        let mut folder = consumer.into_folder();
        for item in producer {
            folder = folder.consume(item);
            if folder.full() { break }
        }
        folder.complete()
    }
}

pub struct NoopReducer;

impl Reducer<()> for NoopReducer {
    fn reduce(self, _left: (), _right: ()) { }
}
