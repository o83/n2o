#![feature(test)]
extern crate test;
extern crate kernel;

use test::Bencher;
use kernel::commands::*;
use kernel::commands::ast::*;
use kernel::streams::interpreter::*;
use kernel::streams::stack::Stack;
use std::cell::UnsafeCell;

#[bench]
fn empty(b: &mut Bencher) {
    b.iter(|| 1)
}

#[bench]
fn parse1(b: &mut Bencher) {
    let mut i = Interpreter::new().unwrap();
    let eval = &"1*2+3".to_string();
    b.iter(|| {
        i.parse(eval);
    })
}

#[bench]
fn parse2(b: &mut Bencher) {
    let mut i = Interpreter::new().unwrap();
    let eval = &"+/{x*y}[(a;b;c;d;e);(2;6;2;1;3)]".to_string();
    b.iter(|| {
        i.parse(eval);
    })
}

// #[bench]
// #fn k_plus(b: &mut Bencher) {
// #b.iter(|| ast::eval(AST::Verb(Verb::Plus, AST::Number(2).boxed(), AST::Number(3).boxed())));
//

#[bench]
fn parse4(b: &mut Bencher) {
    let mut i = Interpreter::new().unwrap();
    let eval = &"();[];{};(());[[]];{{}};()();1 2 3;(1 2 3);[1 2 3];[a[b[c[d]]]];(a(b(c(d))));{a{b{c{d}}}};"
        .to_string();
    b.iter(|| {
        i.parse(eval);
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
fn fac_rec<'a>(b: &'a mut Bencher) {
    let uc = UnsafeCell::new(Interpreter::new().unwrap());
    let se1: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let se2: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let se3: &mut Interpreter<'a> = unsafe { &mut *uc.get() };

    let eval = &"fac:{$[x=1;1;x*fac[x-1]]}".to_string();
    let mut code = se1.parse(eval);
    se2.run(code).unwrap();
    let f = se3.parse(&"fac[5]".to_string());
    b.iter(|| {
        let se4: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        let se5: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        se4.run(f);
        se5.gc();
    })
}

#[bench]
fn fac_tail<'a>(b: &'a mut Bencher) {
    let uc = UnsafeCell::new(Interpreter::new().unwrap());
    let se1: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let se2: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let se3: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let se4: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let eval = &"fac:{[a;b]$[a=1;b;fac[a-1;a*b]]}".to_string();
    let code = se1.parse(eval);
    se2.run(code).unwrap();
    let f = se3.parse(&"fac[4;5]".to_string());
    let code = se4.parse(eval);
    b.iter(|| {
        let se5: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        let se6: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        se5.run(f);
        se6.gc();
    })
}

#[bench]
fn fac_mul<'a>(b: &'a mut Bencher) {
    let hdl = handle();
    let f = hdl.borrow().parse(&"2*3*4*5".to_string());
    b.iter(|| {
        hdl.borrow_mut().run(f);
        hdl.borrow_mut().gc();
    })
}

#[bench]
fn akkerman_k<'a>(b: &'a mut Bencher) {
    let uc = UnsafeCell::new(Interpreter::new().unwrap());
    let se1: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let se2: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let se3: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
    let akk = se1.parse(&"f:{[x;y]$[0=x;1+y;$[0=y;f[x-1;1];f[x-1;f[x;y-1]]]]}".to_string());
    se2.run(akk).unwrap();
    let call = se3.parse(&"f[3;4]".to_string());
    b.iter(|| {
        let se4: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        let se5: &mut Interpreter<'a> = unsafe { &mut *uc.get() };
        se4.run(call);
        se5.gc();
    })
}

fn ack(m: isize, n: isize) -> isize {
    if m == 0 {
        n + 1
    } else if n == 0 {
        ack(m - 1, 1)
    } else {
        ack(m - 1, ack(m, n - 1))
    }
}

#[bench]
fn akkerman_rust(b: &mut Bencher) {
    b.iter(|| ack(3, 4))
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
