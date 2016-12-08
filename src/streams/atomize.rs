use std::hash::BuildHasherDefault;
use std::rc::Rc;
use std::cell::RefCell;
use streams::interpreter::*;
use streams::env::*;
use commands::ast::*;
use commands::ast;

pub fn atomize(p: AST, i: &mut Interpreter) -> AST {
    match p {
        AST::Cons(box ax, box bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            ast::cons(a, b)
        }
        AST::Assign(box ax, box bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Assign(box a, box b)
        }
        AST::Lambda(box ax, box bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Lambda(box a, box b)
        }
        AST::Call(box ax, box bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Call(box a, box b)
        }
        AST::Verb(verb, box ax, box bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Verb(verb, box a, box b)
        }
        AST::Adverb(adverb, box ax, box bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Adverb(adverb, box a, box b)
        }
        AST::Cond(box ax, box bx, box cx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            let c = atomize(cx, i);
            AST::Cond(box a, box b, box c)
        }
        AST::List(box ax) => {
            let a = atomize(ax, i);
            AST::List(box a)
        }
        AST::Dict(box ax) => {
            let a = atomize(ax, i);
            AST::Dict(box a)
        }
        AST::Name(s) => {
            if i.names.contains_key(&s) {
                AST::NameInt(i.names[&s])
            } else {
                let a = i.names_size;
                i.names.insert(s.clone(), a);
                i.names_size = a + 1;
                AST::NameInt(a)
            }
        }
        x => x,
    }
}

pub fn replace_env(k: Cont, env: Rc<RefCell<Environment>>) -> Cont {
    match k {
        Cont::Expressions(a, e, c) => Cont::Expressions(a, env, c),
        Cont::Assign(a, e, c) => Cont::Assign(a, env, c),
        Cont::Cond(a, b, e, c) => Cont::Cond(a, b, env, c),
        Cont::Func(a, b, e, c) => Cont::Func(a, b, env, c),
        Cont::Call(a, b, e, c) => Cont::Call(a, b, env, c),
        Cont::Verb(verb, a, u, e, c) => Cont::Verb(verb, a, u, env, c),
        Cont::Adverb(adverb, a, e, c) => Cont::Adverb(adverb, a, env, c),
        x => x,
    }
}

pub fn extract_env(k: Cont, env: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
    match k {
        Cont::Expressions(a, e, c) => e,
        Cont::Assign(a, e, c) => e,
        Cont::Cond(a, b, e, c) => e,
        Cont::Func(a, b, e, c) => e,
        Cont::Call(a, b, e, c) => e,
        Cont::Verb(verb, a, u, e, c) => e,
        Cont::Adverb(adverb, a, e, c) => e,
        x => Environment::new_child(env),
    }
}
