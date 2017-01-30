#![feature(test)]
extern crate test;
extern crate kernel;

use kernel::commands::ast::*;
use kernel::streams::interpreter::*;
use kernel::reactors::task::{Termination, Context, Poll, Task};
use kernel::reactors::job::Job;
use kernel::reactors::cps::CpsTask;
use kernel::reactors::scheduler::Scheduler;
use kernel::handle::{self, into_raw, UnsafeShared, use_, from_raw};
use kernel::intercore::bus::Memory;
use kernel::intercore::message::Message;
use kernel::intercore::server::intercore;
use kernel::queues::publisher::{Publisher, Subscriber};

fn av<'a>(x: Value) -> ASTNode<'a> {
    ASTNode::AST(AST::Value(x))
}

#[test]
pub fn k_ariph() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());
    let code = h.borrow_mut().parse(&"1+2".to_string());

    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::Verb(Verb::Plus,
                                                            &av(Value::Number(1)),
                                                            &av(Value::Number(2))))]));

    let code = h.borrow_mut().parse(&"1+2*4".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::Verb(Verb::Plus,
                                                            &av(Value::Number(1)),
                                                            &ASTNode::AST(AST::Verb(Verb::Times,
                                                                                    &av(Value::Number(2)),
                                                                                    &av(Value::Number(4))))))]));
}

#[test]
pub fn k_list() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());
    let code = h.borrow_mut().parse(&"(1;\"2\";3;4.1111)".to_string());

    let v: Vec<ASTNode> =
        vec![av(Value::Number(1)), av(Value::SequenceInt(0)), av(Value::Number(3)), av(Value::Float(4.1111))];
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::List(&ASTNode::VecAST(v)))]));
}

#[test]
pub fn k_symbols() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());
    let code = h.borrow_mut().parse(&"`a`b`c;`1`1`1".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(
                   vec![
                       // symbols
                       ASTNode::AST(AST::Call(&av(Value::SymbolInt(0)),
                                              &ASTNode::AST(AST::Call(&av(Value::SymbolInt(1)),
                                                                      &av(Value::SymbolInt(2)))))), 

                       // values
                       ASTNode::AST(AST::Call(&av(Value::SymbolInt(3)),
                                              &ASTNode::AST(AST::Call(&av(Value::Number(1)),
                                                                      &ASTNode::AST(AST::Call(&av(Value::SymbolInt(3)),
                                                                                              &ASTNode::AST(AST::Call(&av(Value::Number(1)),
                                                                                                                      &ASTNode::AST(AST::Call(&av(Value::SymbolInt(3)), &av(Value::Number(1))))))))))))
                           
                   ]));
}

#[test]
pub fn k_assign() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());
    let code = h.borrow_mut().parse(&"a:b:c:1".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(
                   vec![ASTNode::AST(AST::Assign(&ASTNode::AST(AST::NameInt(0)),
                                                 &ASTNode::AST(AST::Assign(&ASTNode::AST(AST::NameInt(1)),
                                                                           &ASTNode::AST(AST::Assign(&ASTNode::AST(AST::NameInt(2)), &av(Value::Number(1))))))))]));
}

#[test]
pub fn k_anyargs0() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"[]".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::Dict(&ASTNode::VecAST(vec![ASTNode::AST(AST::Any)])))]));
}

#[test]
pub fn k_anyargs1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"[;]".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::Dict(&ASTNode::VecAST(vec![ASTNode::AST(AST::Any),
                                                                                  ASTNode::AST(AST::Any)])))]));
}

#[test]
pub fn k_anyargs2() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"[;;]".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::Dict(&ASTNode::VecAST(vec![ASTNode::AST(AST::Any),
                                                                                  ASTNode::AST(AST::Any),
                                                                                  ASTNode::AST(AST::Any)])))]));
}

#[test]
pub fn k_anyargs3() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"[;;3]".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::Dict(&ASTNode::VecAST(vec![ASTNode::AST(AST::Any),
                                                                                  ASTNode::AST(AST::Any),
                                                                                  av(Value::Number(3))])))]));
}

#[test]
pub fn k_anyargs4() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"[1;;]".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::Dict(&ASTNode::VecAST(vec![av(Value::Number(1)),
                                                                                  ASTNode::AST(AST::Any),
                                                                                  ASTNode::AST(AST::Any)])))]));
}

#[test]
pub fn k_vecconst1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"(1;2;3)".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::List(&av(Value::VecInt(vec![1, 2, 3]))))]));
}

#[test]
pub fn k_vecconst2() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"(1.0;2.0;3.0)".to_string());
    assert_eq!(code,
               &ASTNode::VecAST(vec![ASTNode::AST(AST::List(&av(Value::VecFloat(vec![1.0, 2.0, 3.0]))))]));
}

#[test]
pub fn k_plus() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"2+5+3".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "10");
}

#[test]
pub fn k_func() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"{x*2}[(1;2;3)]".to_string());
    assert_eq!(format!("{:?}", code),
               "VecAST([AST(Call(AST(Lambda(None, AST(NameInt(0)), VecAST([AST(Verb(Times, AST(NameInt(0)), \
                AST(Value(Number(2)))))]))), AST(Dict(VecAST([AST(List(AST(Value(VecInt([1, 2, 3])))))])))))])");
}

#[test]
pub fn k_adverb() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"{x+2}/(1;2;3)".to_string());
    assert_eq!(format!("{:?}", code),
               "VecAST([AST(Adverb(Over, AST(Lambda(None, AST(NameInt(0)), VecAST([AST(Verb(Plus, AST(NameInt(0)), \
                AST(Value(Number(2)))))]))), AST(List(AST(Value(VecInt([1, 2, 3])))))))])");
}


#[test]
pub fn k_reduce() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"+/{x*y}[(1;3;4;5;6);(2;6;2;1;3)]".to_string());
    assert_eq!(format!("{:?}", code),
               "VecAST([AST(Adverb(Over, AST(Verb(Plus, AST(Value(Nil)), AST(Value(Nil)))), \
                AST(Call(AST(Lambda(None, AST(NameInt(0)), VecAST([AST(Verb(Times, AST(NameInt(0)), \
                AST(NameInt(1))))]))), AST(Dict(VecAST([AST(List(AST(Value(VecInt([1, 3, 4, 5, 6]))))), \
                AST(List(AST(Value(VecInt([2, 6, 2, 1, 3])))))])))))))])");
}

#[test]
pub fn k_dict1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:10;[1;2;a;5]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[1;2;10;5]");
}

#[test]
pub fn k_dict2() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"[1;[\"2\";3];4;5]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[1;#a[0s;3];4;5]");
}

#[test]
pub fn k_dict3() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"[1;[\"2\";[\"3\";3]];4;5]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[1;#a[0s;#a[1s;3]];4;5]");
}

#[test]
pub fn k_nested_dict() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:10;[1;2;[a+a;[4+a;3];2];5]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[1;2;#a[20;#a[14;3];2];5]");
}

#[test]
pub fn k_list1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"(1;(\"2\";(\"3\";3));4;5)".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[1;#a[0s;#a[1s;3]];4;5]");
}

#[test]
pub fn k_nested_list() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:10;(1;2;(a+a;(4+a;3);2);5)".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[1;2;#a[20;#a[14;3];2];5]");
}

#[test]
pub fn k_expr1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"1;[\"2\";1]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[0s;1]");
}

#[test]
pub fn k_repl1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"xo:{[x;y]y};xo[1;[2;3]]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#i[2;3]");

    // test to avoid specializing vectors in function arguments
    let code = h.borrow_mut().parse(&"xo:{[x;y;z]y};xo[1;2;3]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "2");
}

#[test]
pub fn k_repl2() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"xo:{1};z:{[x]xo x};d:{[x]z x};e:{[x]d x};e[3]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "1");
}

#[test]
pub fn k_repl3() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"y:3;addy:{y};f:{[g;y]g y};f[addy;1]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "3");
}

#[test]
pub fn k_factorial() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"fac:{$[x=0;1;x*fac[x-1]]};fac 20".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "2432902008176640000");
}

#[test]
pub fn k_tail_factorial() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"x:5;fac:{[a;b]$[a=1;b;fac[a-1;a*b]]};fac[x-1;x]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "120");
}

#[test]
pub fn k_cond1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:{[x;y]x y};a[{x};10]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "10");
}

#[test]
pub fn k_cond2() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:{[x;y]$[x y;20;10]};a[{x};10]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "20");
}

#[test]
pub fn k_cond3() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:{[x;y]$[x y;20;10]};a[{x};0]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "10");

}

#[test]
pub fn k_14() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"f:{a:9};a:14;k:{[x] a}; k 3".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "14");
}

#[test]
pub fn k_multiargs() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"xa:9;f:{[x;y;z]x+y*z};f[1;xa+11;3]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "61");
}

#[test]
pub fn k_multiargs2() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"b:2;a:3;fac:{[x;y]x*y};fac[b*a;a+1]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "24");
}
#[test]
pub fn k_tensor0() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"g:1;b:1;[[g;g*b;1;0];[g*b;g;180;0];[0;0;270;0];[0;0;0;1]]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[#a[1;1;1;0];#a[1;1;180;0];#i[0;0;270;0];#i[0;0;270;0];#i[0;0;0;1];#i[0;0;0;1]]");
}

#[test]
pub fn k_tensor1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:10;[[[a;2;3];[1;[a;4];3]];[1;2]]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[#a[#a[10;2;3];#a[1;#a[10;4];3]];#i[1;2];#i[1;2]]");
}

#[test]
pub fn k_tensor2() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:10;[[[a;2;3];[[a;4];[3;0]]];[1;2]]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[#a[#a[10;2;3];#a[#a[10;4];#i[3;0];#i[3;0]]];#i[1;2];#i[1;2]]");
}

#[test]
pub fn k_tensor3() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"a:10;[[[[a;2;3];[[a;4];[3;0]]];[1;2]];1]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#a[#a[#a[#a[10;2;3];#a[#a[10;4];#i[3;0];#i[3;0]]];#i[1;2];#i[1;2]];1]");
}

#[test]
pub fn k_application_order() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code1 = h.borrow_mut().parse(&"a:10;print:{x+1};print[a * 10]".to_string());
    let code2 = h.borrow_mut().parse(&"a:10;print:{x+1};print a * 10".to_string());
    assert_eq!(format!("{}",
                       h.borrow_mut().run(code1, Context::Nil, None).unwrap() ==
                       h.borrow_mut().run(code2, Context::Nil, None).unwrap()),
               "true");
}

#[test]
pub fn k_akkerman() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"f:{[x;y]$[0=x;1+y;$[0=y;f[x-1;1];f[x-1;f[x;y-1]]]]};f[3;4]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "125");
}

#[test]
pub fn rust_pubsub() {
    let mut p: Publisher<i64> = Publisher::with_capacity(8);
    let s1 = p.subscribe();
    let s2 = p.subscribe();
    if let Some(v) = p.next() {
        *v = 11;
        p.commit();
    }
    if let Some(v) = p.next() {
        *v = 12;
        p.commit();
    }

    for i in 0..2 {
        if let Some(v) = s1.recv() {
            assert!(*v == 11 + i);
            s1.commit();
        }
    }
    for i in 0..2 {
        if let Some(v) = s2.recv() {
            assert!(*v == 11 + i);
            s2.commit();
        }
    }
}
// #[test]
// pub fn k_pubsub() {
// let ref mut sched = Scheduler::with_channel(0);
// let s = into_raw(sched);
// let code = "p0:pub[0;8]; s1:sub[0;p0]; s2:sub[0;p0]; snd[p0;11]; snd[p0;12]; print[rcv s1; rcv s2; rcv s1; rcv s2]";
// let shell = from_raw(s).spawn(Job::Cps(CpsTask::new(sched.mem())),
// Termination::Corecursive,
// Some(code));
//
// let t = into_raw(sched.tasks.get_mut(shell.0).expect("no shell"));
// from_raw(t).0.exec(Some(code));
// let mut poll;
// let mut msg1 = Message::Nop;
// let mut ctx = Context::Nil;
// poll = from_raw(t).0.poll(ctx.clone(), from_raw(sched));
// match poll.clone() {
// Poll::Yield(Context::Intercore(i)) => msg1 = i.clone(),
// _ => (),
// }
// ctx = intercore(from_raw(s), Some(use_(&mut msg1)), &mut from_raw(s).bus);
// poll = from_raw(t).0.poll(ctx.clone(), from_raw(sched));
// println!("ctx 1: {:?}", ctx.clone());
// let mut msg2 = Message::Nop;
// match poll.clone() {
// Poll::Yield(Context::Intercore(i)) => msg2 = i.clone(),
// _ => (),
// }
// ctx = intercore(from_raw(s), Some(use_(&mut msg2)), &mut from_raw(s).bus);
// poll = from_raw(t).0.poll(ctx.clone(), from_raw(sched));
// println!("ctx 2: {:?}", ctx.clone());
// let mut msg3 = Message::Nop;
// match poll.clone() {
// Poll::Yield(Context::Intercore(i)) => msg3 = i.clone(),
// _ => (),
// }
// ctx = intercore(from_raw(s), Some(use_(&mut msg3)), &mut from_raw(s).bus);
// poll = from_raw(t).0.poll(ctx.clone(), from_raw(sched));
// println!("ctx 3: {:?}", ctx.clone());
// println!("poll: {:?}", poll.clone());
// match poll.clone() {
// Poll::End(Context::Node(s)) => assert_eq!(format!("{}", s), "[11 11 12 12]"),
// _ => assert_eq!(1, 0),
// }
// }
//
#[test]
pub fn k_partial1() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"aa:{[x;y]x+y};bb:aa[;2];bb 3".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "5");
}

#[test]
pub fn k_partial2() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"aa:{[x;y;z]x+y+z};bb:aa[;;];bb[1;2;3]".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "6");
}

#[test]
pub fn k_vecop_va() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"(1;2;3)+1".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#i[2;3;4]");
}

#[test]
pub fn k_vecop_vv() {
    let mut mem = Memory::new();
    let h = handle::new(Interpreter::new(unsafe { UnsafeShared::new(&mut mem as *mut Memory) }).unwrap());

    let code = h.borrow_mut().parse(&"(1;2;3)+(1;2;3)".to_string());
    assert_eq!(format!("{}", h.borrow_mut().run(code, Context::Nil, None).unwrap()),
               "#i[2;4;6]");
}