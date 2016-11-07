#![allow(dead_code)]

use std::f64;
use std::ops::Fn;
use iterators::chain::ChainIter;
use iterators::collect::collect_into;
use iterators::enumerate::Enumerate;
use iterators::filter::Filter;
use iterators::filter_map::FilterMap;
use iterators::flat_map::FlatMap;
use iterators::map::{Map, MapFn, MapCloned, MapInspect};
use iterators::reduce::{reduce, ReduceOp, SumOp, MulOp, MinOp, MaxOp,
                   ReduceWithIdentityOp, SUM, MUL, MIN, MAX};
use iterators::internal::*;
use iterators::weight::Weight;
use iterators::zip::ZipIter;
use iterators::*;

pub trait IntoParallelIterator {
    type Iter: ParallelIterator<Item=Self::Item>;
    type Item: Send;

    fn into_par_iter(self) -> Self::Iter;
}

pub trait IntoParallelRefIterator<'data> {
    type Iter: ParallelIterator<Item=Self::Item>;
    type Item: Send + 'data;

    fn par_iter(&'data self) -> Self::Iter;
}

impl<'data, I: 'data + ?Sized> IntoParallelRefIterator<'data> for I
    where &'data I: IntoParallelIterator
{
    type Iter = <&'data I as IntoParallelIterator>::Iter;
    type Item = <&'data I as IntoParallelIterator>::Item;

    fn par_iter(&'data self) -> Self::Iter {
        self.into_par_iter()
    }
}

pub trait IntoParallelRefMutIterator<'data> {
    type Iter: ParallelIterator<Item=Self::Item>;
    type Item: Send + 'data;

    fn par_iter_mut(&'data mut self) -> Self::Iter;
}

impl<'data, I: 'data + ?Sized> IntoParallelRefMutIterator<'data> for I
    where &'data mut I: IntoParallelIterator
{
    type Iter = <&'data mut I as IntoParallelIterator>::Iter;
    type Item = <&'data mut I as IntoParallelIterator>::Item;

    fn par_iter_mut(&'data mut self) -> Self::Iter {
        self.into_par_iter()
    }
}

pub trait ToParallelChunks<'data> {
    type Iter: ParallelIterator<Item=&'data [Self::Item]>;
    type Item: Sync + 'data;
    fn par_chunks(&'data self, size: usize) -> Self::Iter;
}

pub trait ToParallelChunksMut<'data> {
    type Iter: ParallelIterator<Item=&'data mut [Self::Item]>;
    type Item: Send + 'data;
    fn par_chunks_mut(&'data mut self, size: usize) -> Self::Iter;
}

/// The `ParallelIterator` interface.
pub trait ParallelIterator: Sized {
    type Item: Send;
    fn weight(self, scale: f64) -> Weight<Self> {
        Weight::new(self, scale)
    }

    fn weight_max(self) -> Weight<Self> {
        self.weight(f64::INFINITY)
    }

    fn for_each<OP>(self, op: OP)
        where OP: Fn(Self::Item) + Sync
    {
        for_each::for_each(self, &op)
    }

    fn count(self) -> usize {
        self.map(|_| 1).sum()
    }

    fn map<MAP_OP,R>(self, map_op: MAP_OP) -> Map<Self, MapFn<MAP_OP>>
        where MAP_OP: Fn(Self::Item) -> R + Sync
    {
        Map::new(self, MapFn(map_op))
    }

    fn cloned<'a, T>(self) -> Map<Self, MapCloned>
        where T: 'a + Clone, Self: ParallelIterator<Item=&'a T>
    {
        Map::new(self, MapCloned)
    }

    fn inspect<INSPECT_OP>(self, inspect_op: INSPECT_OP) -> Map<Self, MapInspect<INSPECT_OP>>
        where INSPECT_OP: Fn(&Self::Item) + Sync
    {
        Map::new(self, MapInspect(inspect_op))
    }

    fn filter<FILTER_OP>(self, filter_op: FILTER_OP) -> Filter<Self, FILTER_OP>
        where FILTER_OP: Fn(&Self::Item) -> bool + Sync
    {
        Filter::new(self, filter_op)
    }

    fn filter_map<FILTER_OP,R>(self, filter_op: FILTER_OP) -> FilterMap<Self, FILTER_OP>
        where FILTER_OP: Fn(Self::Item) -> Option<R> + Sync
    {
        FilterMap::new(self, filter_op)
    }

    fn flat_map<MAP_OP,PI>(self, map_op: MAP_OP) -> FlatMap<Self, MAP_OP>
        where MAP_OP: Fn(Self::Item) -> PI + Sync, PI: IntoParallelIterator
    {
        FlatMap::new(self, map_op)
    }

    fn reduce<OP,IDENTITY>(self, identity: IDENTITY, op: OP) -> Self::Item
        where OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
              IDENTITY: Fn() -> Self::Item + Sync,
    {
        reduce(self, &ReduceWithIdentityOp::new(&identity, &op))
    }

    fn reduce_with<OP>(self, op: OP) -> Option<Self::Item>
        where OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.map(Some)
            .reduce(|| None,
                    |opt_a, opt_b| match (opt_a, opt_b) {
                        (Some(a), Some(b)) => Some(op(a, b)),
                        (Some(v), None) | (None, Some(v)) => Some(v),
                        (None, None) => None,
                    })
    }

    fn fold<IDENTITY_ITEM,IDENTITY,FOLD_OP>(self,
                                            identity: IDENTITY,
                                            fold_op: FOLD_OP)
                                            -> fold::Fold<Self, IDENTITY, FOLD_OP>
        where FOLD_OP: Fn(IDENTITY_ITEM, Self::Item) -> IDENTITY_ITEM + Sync,
              IDENTITY: Fn() -> IDENTITY_ITEM + Sync,
              IDENTITY_ITEM: Send,
    {
        fold::fold(self, identity, fold_op)
    }

    fn sum(self) -> Self::Item
        where SumOp: ReduceOp<Self::Item>
    {
        reduce(self, SUM)
    }

    fn mul(self) -> Self::Item
        where MulOp: ReduceOp<Self::Item>
    {
        reduce(self, MUL)
    }

    fn min(self) -> Self::Item
        where MinOp: ReduceOp<Self::Item>
    {
        reduce(self, MIN)
    }

    fn max(self) -> Self::Item
        where MaxOp: ReduceOp<Self::Item>
    {
        reduce(self, MAX)
    }

    fn chain<CHAIN>(self, chain: CHAIN) -> ChainIter<Self, CHAIN::Iter>
        where CHAIN: IntoParallelIterator<Item=Self::Item>
    {
        ChainIter::new(self, chain.into_par_iter())
    }

    fn find_any<FIND_OP>(self, predicate: FIND_OP) -> Option<Self::Item>
        where FIND_OP: Fn(&Self::Item) -> bool + Sync
    {
        find::find(self, predicate)
    }

    #[doc(hidden)]
    #[deprecated(note = "parallel `find` does not search in order -- use `find_any`")]
    fn find<FIND_OP>(self, predicate: FIND_OP) -> Option<Self::Item>
        where FIND_OP: Fn(&Self::Item) -> bool + Sync
    {
        self.find_any(predicate)
    }

    fn any<ANY_OP>(self, predicate: ANY_OP) -> bool
        where ANY_OP: Fn(Self::Item) -> bool + Sync
    {
        self.map(predicate).find_any(|&p| p).is_some()
    }

    fn all<ALL_OP>(self, predicate: ALL_OP) -> bool
        where ALL_OP: Fn(Self::Item) -> bool + Sync
    {
        self.map(predicate).find_any(|&p| !p).is_none()
    }

    #[doc(hidden)]
    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>;
}

impl<T: ParallelIterator> IntoParallelIterator for T {
    type Iter = T;
    type Item = T::Item;

    fn into_par_iter(self) -> T {
        self
    }
}

pub trait BoundedParallelIterator: ParallelIterator {
    fn upper_bound(&mut self) -> usize;

    #[doc(hidden)]
    fn drive<'c, C: Consumer<Self::Item>>(self,
                                          consumer: C)
                                          -> C::Result;
}

pub trait ExactParallelIterator: BoundedParallelIterator {
    fn len(&mut self) -> usize;
    fn collect_into(self, target: &mut Vec<Self::Item>) {
        collect_into(self, target);
    }
}

pub trait IndexedParallelIterator: ExactParallelIterator {
    #[doc(hidden)]
    fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output;

    fn zip<ZIP_OP>(self, zip_op: ZIP_OP) -> ZipIter<Self, ZIP_OP::Iter>
        where ZIP_OP: IntoParallelIterator, ZIP_OP::Iter: IndexedParallelIterator
    {
        ZipIter::new(self, zip_op.into_par_iter())
    }

    fn enumerate(self) -> Enumerate<Self> {
        Enumerate::new(self)
    }

    fn position_any<POSITION_OP>(self, predicate: POSITION_OP) -> Option<usize>
        where POSITION_OP: Fn(Self::Item) -> bool + Sync
    {
        self.map(predicate).enumerate()
            .find_any(|&(_, p)| p)
            .map(|(i, _)| i)
    }

    #[doc(hidden)]
    #[deprecated(note = "parallel `position` does not search in order -- use `position_any`")]
    fn position<POSITION_OP>(self, predicate: POSITION_OP) -> Option<usize>
        where POSITION_OP: Fn(Self::Item) -> bool + Sync
    {
        self.position_any(predicate)
    }
}

