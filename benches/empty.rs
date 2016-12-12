#![feature(test)]
extern crate test;
extern crate kernel;

use test::Bencher;
use kernel::commands::*;
use kernel::commands::ast::*;
use kernel::streams::interpreter::*;
use kernel::streams::stack::*;

#[bench]
fn empty(b: &mut Bencher) {
    b.iter(|| 1)
}

#[bench]
fn parse1(b: &mut Bencher) {
    b.iter(|| {
        command::parse_Mex("1*2+3");
    })
}

#[bench]
fn parse2(b: &mut Bencher) {
    b.iter(|| {
        command::parse_Mex("+/{x*y}[(a;b;c;d;e);(2;6;2;1;3)]");
    })
}

//#[bench]
//#fn k_plus(b: &mut Bencher) {
//#b.iter(|| ast::eval(AST::Verb(Verb::Plus, AST::Number(2).boxed(), AST::Number(3).boxed())));
//

#[bench]
fn parse4(b: &mut Bencher) {
    b.iter(|| {
        command::parse_Mex("();[];{};(());[[]];{{}};()();1 2 3;(1 2 3);[1 2 \
                            3];[a[b[c[d]]]];(a(b(c(d))));{a{b{c{d}}}};");
    })
}

#[bench]
fn fac_rust(b: &mut Bencher) {
    let mut x: i64 = 0;
    let mut a: i64 = 5;
    b.iter(|| {
        x = factorial(a);
    });
}

#[inline]
fn factorial(value: i64) -> i64 {
    if value == 1 {
        1
    } else {
        return value * factorial(value - 1);
    }
}

#[bench]
fn fac(b: &mut Bencher) {
    let mut i = Interpreter::new().unwrap();
    let ref mut code = ast::parse(&"fac:{$[x=1;1;x*fac[x-1]]}".to_string());
    i.run(code).unwrap();
    let ref mut f = ast::parse(&"fac[5]".to_string());
    b.iter(|| {
        i.run(f);
    })
}

#[derive(Debug,PartialEq,Clone)]
struct Entry(u16, i64);

#[bench]
fn stack_batch(b: &mut Bencher) {
    let capacity = (!0 as u16) as usize;
    let mut stack: Stack<Entry> = Stack::with_capacity(capacity);
    let items = [Entry(9, 9), Entry(6, 6), Entry(7, 7)];
    b.iter(|| {
        stack.insert_many(&items);
    });
    stack.finalize();
}

#[bench]
fn stack_iter(b: &mut Bencher) {
    let capacity = (!0 as u16) as usize;
    let mut stack: Stack<Entry> = Stack::with_capacity(capacity);
    let items = [Entry(9, 9), Entry(6, 6), Entry(7, 7)];
    b.iter(|| {
        stack.insert_many_v2(&items);
    });
}

#[bench]
fn stack_iter_cloned(b: &mut Bencher) {
    let capacity = (!0 as u16) as usize;
    let mut stack: Stack<Entry> = Stack::with_capacity(capacity);
    let items = [Entry(9, 9), Entry(6, 6), Entry(7, 7)];
    b.iter(|| {
        stack.insert_many_v3(&items);
    });
}
