// #
//
// tele.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//
extern crate kernel;

use kernel::io::poll::*;
use kernel::io::tele::Tele;
use kernel::io::token::Token;

#[test]
fn tele_test() {
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut t = Tele::new(Token::from(10500));
    println!("REGISTER: {:?}", t.register(&mut poll));
}
