
// O-DSL AST

use std::fmt;
use std::iter;
use std::vec;
use std::result::Result;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use commands::command;
use streams::interpreter;
use streams::interpreter::*;
use std::cell::UnsafeCell;
use std::{mem, ptr, isize};

#[derive(Debug)]
pub enum Error {
    ParseError,
    EvalError { desc: String, ast: String },
    InternalError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError => write!(f, "Parse error!\n"),
            Error::EvalError { ref desc, ref ast } => write!(f, "Eval error: {}.\nCaused here: {}\n", desc, ast),
            Error::InternalError => write!(f, "Internal error!\n"),
        }
    }
}

#[derive(PartialEq,Debug, Clone)]
pub enum Type {
    Nil = 0,
    Number = 1,
    Char = 2,
    Symbol = 3,
    List = 4,
    Dictionary = 5,
    Function = 6,
    View = 7,
    NameRef = 8,
    Verb = 9,
    Adverb = 10,
    Return = 11,
    Cond = 12,
    Native = 13,
    Quote = 14,
}


// OK LANG

//        a          l           a-a         l-a         a-l         l-l         triad    tetrad
// "+" : [ident,     flip,       ad(plus),   ad(plus),   ad(plus),   ad(plus),   null,    null  ],
// "-" : [am(negate),am(negate), ad(minus),  ad(minus),  ad(minus),  ad(minus),  null,    null  ],
// "*" : [first,     first,      ad(times),  ad(times),  ad(times),  ad(times),  null,    null  ],
// "%" : [sqrt,      am(sqrt),   ad(divide), ad(divide), ad(divide), ad(divide), null,    null  ],
// "!" : [iota,      odometer,   mod,        md,         ar(mod),    md,         null,    null  ],
// "&" : [where,     where,      ad(min),    ad(min),    ad(min),    ad(min),    null,    null  ],
// "|" : [rev,       rev,        ad(max),    ad(max),    ad(max),    ad(max),    null,    null  ],
// "<" : [asc,       asc,        ad(less),   ad(less),   ad(less),   ad(less),   null,    null  ],
// ">" : [desc,      desc,       ad(more),   ad(more),   ad(more),   ad(more),   null,    null  ],
// "=" : [imat,      group,      ad(equal),  ad(equal),  ad(equal),  ad(equal),  null,    null  ],
// "~" : [am(not),   am(not),    match,      match,      match,      match,      null,    null  ],
// "," : [enlist,    enlist,     cat,        cat,        cat,        cat,        null,    null  ],
// "^" : [pisnull,   am(pisnull),except,     except,     except,     except,     null,    null  ],
// "#" : [count,     count,      take,       reshape,    take,       reshape,    null,    null  ],
// "_" : [am(floor), am(floor),  drop,       ddrop,      drop,       cut,        null,    null  ],
// "$" : [kfmt,      am(kfmt),   dfmt,       dfmt,       dfmt,       dfmt,       null,    null  ],
// "?" : [real,      unique,     rnd,        pfind,      rnd,        ar(pfind),  splice,  null  ],
// "@" : [type,      type,       atd,        atl,        atd,        ar(atl),    amend4,  amend4],
// "." : [keval,     keval,      call,       call,       call,       call,       dmend4,  dmend4],
// "'" : [null,      null,       null,       atl,        kwindow,    ar(atl),    null,    null  ],
// "/" : [null,      null,       null,       null,       pack,       pack,       null,    null  ],
// "\\": [null,      null,       null,       unpack,     split,      null,       null,    null  ],

#[derive(PartialEq,Debug,Clone, Copy)]
pub enum Verb {
    Plus = 0,
    Minus = 1,
    Times = 2,
    Divide = 3,
    Mod = 4,
    Min = 5,
    Max = 6,
    Less = 7,
    More = 8,
    Equal = 9,
    Match = 10,
    Concat = 11,
    Except = 12,
    Take = 13,
    Drop = 14,
    Cast = 15,
    Find = 16,
    At = 17,
    Dot = 18,
    Gets = 19,
    Pack = 20,
    Unpack = 21,
    New = 22,
}

#[derive(Debug)]
pub enum Monadic {
    Flip = 0,
    Negate = 1,
    First = 2,
    Sqrt = 3,
    Iota = 4,
    Where = 5,
    Rev = 6,
    Asc = 7,
    Desc = 8,
    Group = 9,
    Not = 10,
    List = 11,
    Nil = 12,
    Count = 13,
    Floor = 14,
    Fmt = 15,
    Unique = 16,
    Type = 17,
    Eval = 18,
}

impl Verb {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "+" => Ok(Verb::Plus),
            "-" => Ok(Verb::Minus),
            "*" => Ok(Verb::Times),
            "%" => Ok(Verb::Divide),
            "!" => Ok(Verb::Mod),
            "&" => Ok(Verb::Min),
            "|" => Ok(Verb::Max),
            "<" => Ok(Verb::Less),
            ">" => Ok(Verb::More),
            "=" => Ok(Verb::Equal),
            "~" => Ok(Verb::Match),
            "," => Ok(Verb::Concat),
            "^" => Ok(Verb::Except),
            "#" => Ok(Verb::Take),
            "_" => Ok(Verb::Drop),
            "$" => Ok(Verb::Cast),
            "?" => Ok(Verb::Find),
            "@" => Ok(Verb::At),
            "." => Ok(Verb::Dot),
            ";" => Ok(Verb::New),
            _ => Err(Error::ParseError),
        }
    }
}

impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Verb::Plus => write!(f, "+"),
            Verb::Minus => write!(f, "-"),
            Verb::Times => write!(f, "*"),
            Verb::Divide => write!(f, "%"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(PartialEq,Debug,Clone, Copy)]
pub enum Adverb {
    Each,
    EachPrio,
    EachLeft,
    EachRight,
    Over,
    Scan,
    Iterate,
    Fixed,
    Assign,
    View,
    Separator,
}

impl Adverb {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "/" => Ok(Adverb::Over),
            "\\" => Ok(Adverb::Scan),
            "'" => Ok(Adverb::Each),
            ";" => Ok(Adverb::Separator),
            ";:" => Ok(Adverb::Separator),
            "':" => Ok(Adverb::EachPrio),
            ":" => Ok(Adverb::Assign),
            "::" => Ok(Adverb::View),
            "\\:" => Ok(Adverb::EachLeft),
            "/:" => Ok(Adverb::EachRight),
            _ => Err(Error::ParseError),
        }
    }
}

impl fmt::Display for Adverb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Adverb::Over => write!(f, "/"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(PartialEq,Debug,Clone)]
pub enum AST<'a> {
    // 0
    Nil,
    // 1
    Cons(&'a AST<'a>, &'a AST<'a>),
    // 2
    List(&'a AST<'a>),
    // 3
    Dict(&'a AST<'a>),
    // 4
    Call(&'a AST<'a>, &'a AST<'a>),
    // 5
    Lambda(&'a AST<'a>, &'a AST<'a>),
    // 6
    Verb(Verb, &'a AST<'a>, &'a AST<'a>),
    // 7
    Adverb(Adverb, &'a AST<'a>, &'a AST<'a>),
    // 8
    Ioverb(String),
    // 9
    NameInt(u16),
    SymbolInt(u16),
    SequenceInt(u16),
    Name(String),
    // A
    Number(i64),
    // B
    Hexlit(i64),
    // C
    Bool(bool),
    // D
    Symbol(String),
    // E
    Sequence(String),
    // F
    Cell(Box<Cell<'a>>),
    // Syntactic sugar
    Assign(&'a AST<'a>, &'a AST<'a>),
    //
    Cond(&'a AST<'a>, &'a AST<'a>, &'a AST<'a>),
}

#[derive(Debug)]
pub struct Arena<'a> {
    pub names: UnsafeCell<HashMap<String, u16>>,
    pub symbols: HashMap<String, u16>,
    pub sequences: HashMap<String, u16>,
    asts: UnsafeCell<Vec<AST<'a>>>,
    conts: UnsafeCell<Vec<Cont<'a>>>,
    lazys: UnsafeCell<Vec<Lazy<'a>>>,
}

impl<'a> Arena<'a> {
    pub fn new() -> Arena<'a> {
        Arena {
            names: UnsafeCell::new(HashMap::new()),
            symbols: HashMap::new(),
            sequences: HashMap::new(),
            asts: UnsafeCell::new(Vec::with_capacity(2048)),
            conts: UnsafeCell::new(Vec::with_capacity(2048)),
            lazys: UnsafeCell::new(Vec::with_capacity(2048)),
        }
    }

    pub fn ast(&self, n: AST<'a>) -> &'a AST<'a> {
        let ast = unsafe { &mut *self.asts.get() };
        ast.push(n);
        ast.last().unwrap()
    }

    pub fn lazy(&self, n: Lazy<'a>) -> &'a Lazy<'a> {
        let lazys = unsafe { &mut *self.lazys.get() };
        lazys.push(n);
        lazys.last().unwrap()
    }

    pub fn cont(&self, n: Cont<'a>) -> &'a Cont<'a> {
        let conts = unsafe { &mut *self.conts.get() };
        conts.push(n);
        conts.last().unwrap()
    }

    pub fn intern(&self, s: String) -> &'a AST<'a> {
        let names = unsafe { &mut *self.names.get() };
        if names.contains_key(&s) {
            self.ast(AST::NameInt(names[&s]))
        } else {
            let id = names.len() as u16;
            names.insert(s, id);
            self.ast(AST::NameInt(id))
        }
    }

    pub fn to_string(&self) {
        let ast = unsafe { &mut *self.asts.get() };
        println!("AST {}, {:?}", ast.len(), ast);
    }

    pub fn clean(&self) -> usize {
        let asts = unsafe { &mut *self.asts.get() };
        let lazys = unsafe { &mut *self.lazys.get() };
        let conts = unsafe { &mut *self.conts.get() };
        let l = conts.len() + asts.len() + lazys.len();
        unsafe {
            asts.set_len(0);
            conts.set_len(0);
            lazys.set_len(0);
        };
        l
    }
}

impl<'a> AST<'a> {
    pub fn len(&self) -> usize {
        match self {
            &AST::List(ref car) => car.len(),
            &AST::Cons(_, ref cdr) => 1 + cdr.len(),
            &AST::Nil => 0,
            _ => 1,
        }
    }
    pub fn is_empty(&self) -> bool {
        self == &AST::Nil
    }
    pub fn is_cons(&self) -> bool {
        match self {
            &AST::Cons(_, _) => true,
            _ => false,
        }
    }
    pub fn to_vec(&self) -> Vec<AST<'a>> {
        let mut out = vec![];
        let mut l = self;
        loop {
            match l {
                &AST::Cons(car, cdr) => {
                    out.push((*car).clone());
                    l = cdr;
                }
                &AST::Nil => break,
                x => {
                    out.push((*x).clone());
                    break;
                }
            }
        }
        out
    }
}

#[derive(Debug)]
pub struct AstIntoIterator<'a> {
    curr: &'a AST<'a>,
    done: bool,
}

impl<'a> Iterator for AstIntoIterator<'a> {
    type Item = &'a AST<'a>;
    fn next(&mut self) -> Option<&'a AST<'a>> {
        if self.done {
            return None;
        }
        match self.curr {
            &AST::Cons(car, cdr) => {
                self.curr = cdr;
                return Some(car);
            }
            &AST::Nil => {
                self.done = true;
                return None;
            }
            x => {
                self.done = true;
                return Some(x);
            }
        }
    }
}

impl<'a> iter::IntoIterator for &'a AST<'a> {
    type Item = &'a AST<'a>;
    type IntoIter = AstIntoIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        AstIntoIterator {
            curr: &self,
            done: false,
        }
    }
}


impl<'a> fmt::Display for AST<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AST::Nil => write!(f, ""),
            AST::Cons(ref a, &AST::Nil) => write!(f, "{}", a),
            AST::Cons(ref a, ref b) => write!(f, "{} {}", a, b),
            AST::List(ref a) => write!(f, "({})", a),
            AST::Dict(ref d) => write!(f, "[{}]", d),
            AST::Call(ref a, ref b) => write!(f, "{} {}", a, b),
            AST::Lambda(a, b) => {
                match *a {
                    AST::Nil => write!(f, "{{[x]{}}}", b),
                    _ => {
                        let args = format!("{}", a).replace(" ", ";");
                        write!(f, "{{[{}]{}}}", args, b)
                    }
                }
            }
            AST::Verb(ref v, ref a, ref b) => write!(f, "{}{}{}", a, v, b),
            AST::Adverb(ref v, ref a, ref b) => write!(f, "{}{}{}", a, v, b),
            AST::Ioverb(ref v) => write!(f, "{}", v),
            AST::Number(n) => write!(f, "{}", n),
            AST::Hexlit(h) => write!(f, "0x{}", h),
            AST::Bool(b) => write!(f, "{:?}", b),
            AST::Name(ref n) => write!(f, "{}", n),
            AST::Symbol(ref s) => write!(f, "{}", s),
            AST::Sequence(ref s) => write!(f, "{:?}", s),
            AST::NameInt(ref n) => write!(f, "^{}", n),
            AST::SymbolInt(ref s) => write!(f, "{}", s),
            AST::SequenceInt(ref s) => write!(f, "{:?}", s),
            AST::Cell(ref c) => write!(f, "{}", c),
            AST::Assign(ref a, ref b) => write!(f, "{}:{}", a, b),
            AST::Cond(ref c, ref a, ref b) => write!(f, "$[{};{};{}]", c, a, b),
        }

    }
}

pub fn extract_name<'a>(a: &'a AST<'a>) -> u16 {
    match a {
        &AST::NameInt(s) => s,
        x => 0,
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Cell<'a> {
    t: Type,
    v: Vec<AST<'a>>,
}

impl<'a> fmt::Display for Cell<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(");
        for i in &self.v {
            write!(f, "{}", i);
        }
        write!(f, ")")
    }
}

pub fn nil<'a>(arena: &'a Arena<'a>) -> &'a AST<'a> {
    arena.ast(AST::Nil)
}

pub fn ast<'a>(n: AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    arena.ast(n)
}

pub fn cont<'a>(n: Cont<'a>, arena: &'a Arena<'a>) -> &'a Cont<'a> {
    arena.cont(n)
}

pub fn lazy<'a>(n: Lazy<'a>, arena: &'a Arena<'a>) -> &'a Lazy<'a> {
    arena.lazy(n)
}

pub fn call<'a>(l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    ast(AST::Call(l, r), arena)
}

pub fn cons<'a>(l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    ast(AST::Cons(l, r), arena)
}

pub fn fun<'a>(l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    match *l {
        AST::Nil => arena.ast(AST::Lambda(arena.intern("x".to_string()), r)),
        _ => arena.ast(AST::Lambda(l, r)),
    }
}

pub fn dict<'a>(l: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    match l {
        &AST::Cons(a, b) => arena.ast(AST::Dict(l)),
        x => x,
    }
}

pub fn list<'a>(l: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    match l {
        &AST::Cons(a, b) => arena.ast(AST::List(l)),
        x => x,
    }
}

pub fn parse<'a>(arena: &'a Arena<'a>, s: &String) -> &'a AST<'a> {
    command::parse_Mex(arena, s).unwrap()
}

pub fn rev_list<'a>(l: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    let mut res = arena.ast(AST::Nil);
    let mut from = match l {
        &AST::List(xs) => xs,
        _ => panic!(),
    };
    println!(" from {:?}", from);
    let mut done = false;
    loop {
        if done {
            break;
        }
        match from {
            &AST::Cons(x, xs) => {
                res = arena.ast(AST::Cons(x, res));
                from = xs;
            }
            &AST::Nil => break,
            x => {
                res = arena.ast(AST::Cons(arena.ast(x.clone()), res));
                done = true;
            }
        }
    }
    arena.ast(AST::List(res))
}


pub fn verb<'a>(v: Verb, l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    match v {
        Verb::Cast => {
            let rexpr = match r {
                &AST::Dict(d) => {
                    match d {
                        &AST::Cons(a, b) => {
                            match b {
                                &AST::Cons(t, f) => arena.ast(AST::Cond(a, t, arena.ast(AST::List(f)))),
                                x => x,
                            }
                        }
                        x => x,
                    }
                }
                x => x, 
            };
            match *l {
                AST::Nil => rexpr,
                _ => arena.ast(AST::Call(l, rexpr)), 
            }
        }
        _ => {
            match r { // optional AST transformations could be done during parsing
                &AST::Adverb(ref a, al, ar) => {
                    match a {
                        &Adverb::Assign => arena.ast(AST::Assign(al, ar)),
                        x => {
                            arena.ast(AST::Adverb(x.clone(),
                                                  arena.ast(AST::Verb(v, l, arena.ast(AST::Nil))),
                                                  ar))
                        }

                    }
                }
                _ => arena.ast(AST::Verb(v, l, r)),
            }
        }
    }
}

pub fn adverb<'a>(a: Adverb, l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    match a {
        Adverb::Assign => arena.ast(AST::Assign(l, r)),
        _ => arena.ast(AST::Adverb(a, l, r)),
    }
}

pub fn rev_dict<'a>(l: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    let mut res = arena.ast(AST::Nil);
    let mut from = l;
    loop {
        match from {
            &AST::Cons(x, xs) => {
                match x {
                    &AST::Dict(z) => {
                        let mut rev = rev_dict(z, arena);
                        res = arena.ast(AST::Cons(arena.ast(AST::Dict(rev)), res));
                        from = xs;
                    }
                    &AST::Cons(_, _) => {
                        let mut rev = rev_dict(x, arena);
                        res = arena.ast(AST::Cons(arena.ast(AST::Dict(rev)), res));
                        from = xs;
                    }
                    y => {
                        res = arena.ast(AST::Cons(y, res));
                        from = xs;
                    }
                }
            }
            &AST::Dict(x) => {
                let mut rev = rev_dict(x, arena);
                // println!("f x: {:?}", res);
                match res {
                    &AST::Nil => res = arena.ast(AST::Dict(rev)),
                    _ => res = arena.ast(AST::Cons(arena.ast(AST::Dict(rev)), res)),
                }
                break;
            }
            &AST::Nil => break,
            x => {
                // println!("x: {:?}", x);
                res = arena.ast(AST::Cons(x, res));
                break;
            }
        }
    }
    res
}
