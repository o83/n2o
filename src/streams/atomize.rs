use std::hash::BuildHasherDefault;
use std::rc::Rc;
use std::cell::RefCell;
use streams::interpreter::*;
use streams::env::*;
use commands::ast::*;
use commands::ast;

pub fn atomize<'ast>(p: &'ast AST<'ast>, i: &'ast mut Interpreter<'ast>) -> AST<'ast> {
    match p {
        AST::Cons(ax, bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            ast::cons(a, b)
        }
        AST::Assign(ax, bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Assign(a, b)
        }
        AST::Lambda(ax, bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Lambda(a, b)
        }
        AST::Call(ax, bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Call(a, b)
        }
        AST::Verb(verb, ax, bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Verb(verb, a, b)
        }
        AST::Adverb(adverb, ax, bx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            AST::Adverb(adverb, a, b)
        }
        AST::Cond(ax, bx, cx) => {
            let a = atomize(ax, i);
            let b = atomize(bx, i);
            let c = atomize(cx, i);
            AST::Cond(a, b, c)
        }
        AST::List(ax) => {
            let a = atomize(ax, i);
            AST::List(a)
        }
        AST::Dict(ax) => {
            let a = atomize(ax, i);
            AST::Dict(a)
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
// pub fn replace_env(tape: Tape, env: Rc<RefCell<Environment>>) -> Tape {
// Tape {
// env: env,
// cont: tape.cont,
// }
// }
//
// pub fn extract_env(tape: Tape) -> Rc<RefCell<Environment>> {
// tape.env
// }
//