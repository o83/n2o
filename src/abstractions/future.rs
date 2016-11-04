// #
//
// future.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

pub enum Async<T> {
    Ready(T),
    NotReady,
}

pub type Poll<T, E> = Result<Async<T>, E>;

pub trait Future {
    type Item;
    type Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error>;
}
