#![feature(test)]
extern crate test;
extern crate kernel;

use test::Bencher;
use kernel::commands::*;
use kernel::commands::ast::*;
use kernel::streams::interpreter::*;

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
//fn k_plus(b: &mut Bencher) {
//    b.iter(|| ast::eval(AST::Verb(Verb::Plus, AST::Number(2).boxed(), AST::Number(3).boxed())));
//}

#[bench]
fn parse4(b: &mut Bencher) {
    b.iter(|| {
        command::parse_Mex("();[];{};(());[[]];{{}};()();1 2 3;(1 2 3);[1 2 \
                            3];[a[b[c[d]]]];(a(b(c(d))));{a{b{c{d}}}};");
    })
}


#[bench]
fn fac(b: &mut Bencher) {
    let mut i = Interpreter::new().unwrap();
    let code = command::parse_Mex("fac:{$[x=1;1;x*fac[x-1]]}").unwrap();
    i.run(code).unwrap();
    let f = command::parse_Mex("fac[5]").unwrap();

    b.iter(|| {
        i.run(f.clone()).unwrap();
    })
}
