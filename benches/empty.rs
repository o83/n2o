#![feature(test)]
extern crate test;
extern crate kernel;

use test::Bencher;
use kernel::commands::*;
use kernel::commands::ast::*;

#[bench]
fn empty(b: &mut Bencher) {
    b.iter(|| 1)
}

#[bench]
fn parse(b: &mut Bencher) {
    b.iter(|| { command::parse_Expr("+/{x*y}[(a;b;c;d;e);(2;6;2;1;3)]"); })
}

#[bench]
fn parse2(b: &mut Bencher) {
    b.iter(|| { command::parse_Expr("1+2"); })
}
