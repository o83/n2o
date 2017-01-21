#![feature(test)]
extern crate test;
extern crate kernel;

use kernel::commands::*;
use kernel::commands::ast::*;
use kernel::streams::interpreter::*;
use std::cell::UnsafeCell;
use kernel::handle;
use kernel::reactors::task::Context;

#[test]
pub fn k_ariph() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"1+2".to_string());
    assert_eq!(*code,
               AST::Verb(Verb::Plus,
                         &AST::Value(Value::Number(1)),
                         &AST::Value(Value::Number(2))));

    let code = h.borrow_mut().parse(&"1+2*4".to_string());
    assert_eq!(*code,
               AST::Verb(Verb::Plus,
                         &AST::Value(Value::Number(1)),
                         &AST::Verb(Verb::Times,
                                    &AST::Value(Value::Number(2)),
                                    &AST::Value(Value::Number(4)))));
}

#[test]
pub fn k_list() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"(1;\"2\";3;4)".to_string());
    assert_eq!(*code,
               AST::List(&AST::Cons(&AST::Value(Value::Number(1)),
                                    &AST::Cons(&AST::Value(Value::SequenceInt(0)),
                                               &AST::Cons(&AST::Value(Value::Number(3)),
                                                          &AST::Value(Value::Number(4)))))));
}

#[test]
pub fn k_symbols() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"`a`b`c;`1`1`1".to_string());
    assert_eq!(*code,
               AST::Cons(&AST::Call(&AST::Value(Value::SymbolInt(0)),
                                    &AST::Call(&AST::Value(Value::SymbolInt(1)),
                                               &AST::Value(Value::SymbolInt(2)))),
                         &AST::Call(&AST::Value(Value::SymbolInt(3)),
                                    &AST::Call(&AST::Value(Value::Number(1)),
                                               &AST::Call(&AST::Value(Value::SymbolInt(3)),
                                                          &AST::Call(&AST::Value(Value::Number(1)),
                                                                     &AST::Call(&AST::Value(Value::SymbolInt(3)),
                                                                                &AST::Value(Value::Number(1)))))))));
}

#[test]
pub fn k_assign() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"a:b:c:1".to_string());
    assert_eq!(*code,
               AST::Assign(&AST::NameInt(0),
                           &AST::Assign(&AST::NameInt(1),
                                        &AST::Assign(&AST::NameInt(2), &AST::Value(Value::Number(1))))));
}

#[test]
pub fn k_anyargs1() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"[;]".to_string());
    assert_eq!(*code,
               AST::Dict(&AST::Cons(&AST::Any, &AST::Cons(&&AST::Any, &&AST::Nil))));
}

#[test]
pub fn k_anyargs2() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"[;;]".to_string());
    assert_eq!(*code,
               AST::Dict(&AST::Cons(&AST::Any,
                                    &AST::Cons(&AST::Any, &AST::Cons(&AST::Any, &AST::Nil)))));
}

#[test]
pub fn k_anyargs3() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"[;;3]".to_string());
    assert_eq!(*code,
               AST::Dict(&AST::Cons(&AST::Any,
                                    &AST::Cons(&AST::Any, &AST::Value(Value::Number(3))))));
}

#[test]
pub fn k_anyargs4() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"[1;;]".to_string());
    assert_eq!(*code,
               AST::Dict(&AST::Cons(&AST::Value(Value::Number(1)),
                                    &AST::Cons(&AST::Any, &AST::Cons(&AST::Any, &AST::Nil)))));
}

#[test]
pub fn k_vecconst1() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"(1;2;3)".to_string());
    let a: Vec<i64> = vec![1, 2, 3];
    assert_eq!(*code, AST::Value(Value::VecInt(a)));
}

#[test]
pub fn k_plus() {
    let uc = UnsafeCell::new(Interpreter::new().unwrap());
    let i1: &mut Interpreter = unsafe { &mut *uc.get() };
    let i2: &mut Interpreter = unsafe { &mut *uc.get() };
    let code = i1.parse(&"2+5".to_string());
    assert_eq!(format!("{}", i2.run(code, Context::Nil).unwrap()), "7");
}

#[test]
pub fn k_func() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"{x*2}[(1;2;3)]".to_string());
    assert_eq!(format!("{:?}", code),
               "Call(Lambda(None, NameInt(0), Verb(Times, NameInt(0), Value(Number(2)))), \
                Value(VecInt([1, 2, 3])))");
}

#[test]
pub fn k_adverb() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"{x+2}/(1;2;3)".to_string());
    assert_eq!(format!("{:?}", code),
               "Adverb(Over, Lambda(None, NameInt(0), Verb(Plus, NameInt(0), Value(Number(2)))), \
                Value(VecInt([1, 2, 3])))");
}


#[test]
pub fn k_reduce() {
    let mut i = Interpreter::new().unwrap();
    let ref mut code = i.parse(&"+/{x*y}[(1;3;4;5;6);(2;6;2;1;3)]".to_string());
    assert_eq!(format!("{:?}", code),
               "Adverb(Over, Verb(Plus, Nil, Nil), Call(Lambda(None, NameInt(0), Verb(Times, NameInt(0), \
                NameInt(1))), Dict(Cons(Value(VecInt([1, 3, 4, 5, 6])), Value(VecInt([2, 6, 2, 1, 3]))))))");
}

#[test]
pub fn k_repl() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"y:3;add:{[x]y};f:{[x]add x};f 1".to_string());
}

#[test]
pub fn k_nested_dict() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"a:10;[1;2;[a+a;[4+a;3];2];5]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "[1 2 [20 [14 3] 2] 5]");
}


#[test]
pub fn k_repl2() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"xo:{1};z:{[x]xo x};d:{[x]z x};e:{[x]d x};e[3]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "1");
}

#[test]
pub fn k_factorial() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"fac:{$[x=0;1;x*fac[x-1]]};fac 20".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "2432902008176640000");
}

#[test]
pub fn k_tail_factorial() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"x:5;fac:{[a;b]$[a=1;b;fac[a-1;a*b]]};fac[x-1;x]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "120");
}

#[test]
pub fn k_cond() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"a:{[x;y]$[x y;20;10]};a[{x};10]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "20");
}

#[test]
pub fn k_cond2() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"a:{[x;y]$[x y;20;10]};a[{x};0]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "10");

}

#[test]
pub fn k_14() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"f:{a:9};a:14;k:{[x] a}; k 3".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "14");
}


#[test]
pub fn k_multiargs2() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"b:2;a:3;fac:{[x;y]x*y};fac[b*a;a+1]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "24");
}

#[test]
pub fn k_multiargs() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"xa:9;f:{[x;y;z]x+y*z};f[1;xa+11;3]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "61");
}

#[test]
pub fn k_repl1() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"y:3;addy:{y};f:{[g;y]g y};f[addy;1]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "3");
}

#[test]
pub fn k_tensor() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"g:1;b:1;[[g;g*b;1;0];[g*b;g;180;0];[0;0;270;0];[0;0;0;1]]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "[[1 1 1 0] [1 1 180 0] [0 0 270 0] [0 0 0 1]]");
}

#[test]
pub fn k_tensor1() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"a:10;[[[a;2;3];[1;[a;4];3]];[1;2]]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "[[[10 2 3] [1 [10 4] 3]] [1 2]]");
}

#[test]
pub fn k_tensor2() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"a:10;[[[a;2;3];[[a;4];[3;0]]];[1;2]]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "[[[10 2 3] [[10 4] [3 0]]] [1 2]]");
}

#[test]
pub fn k_application_order() {
    let h = handle::new(Interpreter::new().unwrap());
    let code1 = h.borrow_mut().parse(&"a:10;print:{x+1};print[a * 10]".to_string());
    let code2 = h.borrow_mut().parse(&"a:10;print:{x+1};print a * 10".to_string());
    assert_eq!(format!("{}",
                       h.borrow_mut().run(code1, Context::Nil).unwrap() == h.borrow_mut().run(code2, Context::Nil).unwrap()),
               "true");
}

#[test]
pub fn k_akkerman() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"f:{[x;y]$[0=x;1+y;$[0=y;f[x-1;1];f[x-1;f[x;y-1]]]]};f[3;4]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "125");
}

#[test]
pub fn k_tensor3() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"a:10;[[[[a;2;3];[[a;4];[3;0]]];[1;2]];1]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "[[[[10 2 3] [[10 4] [3 0]]] [1 2]] 1]");
}

#[test]
pub fn k_pubsub() {
    let h = handle::new(Interpreter::new().unwrap());
    h.borrow_mut().define_primitives();
    let code = h.borrow_mut()
        .parse(&"p0: pub 8; s1: sub 0; s2: sub 0; snd[p0;41]; snd[p0;42]; [rcv s1; rcv s2; rcv s1; rcv s2]"
            .to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "[41 41 42 42]");
}

#[test]
pub fn k_partial1() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"aa:{[x;y]x+y};bb:aa[;2];bb 3".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "5");
}

#[test]
pub fn k_partial2() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"aa:{[x;y;z]x+y+z};bb:aa[;;];bb[1;2;3]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()), "6");
}

#[test]
pub fn k_vecop_va() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"(1;2;3)+1".to_string());
    let a: Vec<i64> = vec![2, 3, 4];
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "#i[2;3;4]");
}

#[test]
pub fn k_vecop_vv() {
    let h = handle::new(Interpreter::new().unwrap());
    let code = h.borrow_mut().parse(&"(1;2;3)+(1;2;3)".to_string());
    let a: Vec<i64> = vec![2, 4, 6];
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil).unwrap()),
               "#i[2;4;6]");
}
