
// O-DSL AST

use std::fmt;
use std::iter;
use std::result::Result;
use std::collections::HashMap;
use commands::command;
use streams::otree;
use reactors::task::Context;
use streams::interpreter::*;
use std::cell::UnsafeCell;
use std::isize;
use std::intrinsics::size_of;
use intercore::message::Message;

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
            Verb::Equal => write!(f, "="),
            Verb::Dot => write!(f, "."),
            Verb::Cast => write!(f, "$"),
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
    Nil,
    Any,
    Cons(&'a AST<'a>, &'a AST<'a>),
    List(&'a AST<'a>),
    Dict(&'a AST<'a>),
    Call(&'a AST<'a>, &'a AST<'a>),
    Assign(&'a AST<'a>, &'a AST<'a>),
    Cond(&'a AST<'a>, &'a AST<'a>, &'a AST<'a>),
    Lambda(Option<&'a otree::Node<'a>>, &'a AST<'a>, &'a AST<'a>),
    Verb(Verb, &'a AST<'a>, &'a AST<'a>),
    Adverb(Adverb, &'a AST<'a>, &'a AST<'a>),
    Table(&'a AST<'a>, &'a AST<'a>),
    Ioverb(String),
    Yield(Context<'a>),
    Value(Value<'a>),
    NameInt(u16),
}

#[derive(PartialEq,Debug,Clone)]
pub enum Value<'a> {
    Nil,
    SymbolInt(u16),
    SequenceInt(u16),
    Number(i64),
    Float(f64),
    VecInt(Vec<i64>),
    VecFloat(Vec<f64>),
    VecAST(Vec<AST<'a>>),
    Ioverb(String),
}

#[derive(Debug)]
pub struct Arena<'a> {
    pub names: UnsafeCell<HashMap<String, u16>>,
    pub symbols: UnsafeCell<HashMap<String, u16>>,
    pub sequences: UnsafeCell<HashMap<String, u16>>,
    pub builtins: u16,
    pub asts: UnsafeCell<Vec<AST<'a>>>,
    pub conts: UnsafeCell<Vec<Cont<'a>>>,
}

fn delta<'a>(this: &'a Cont<'a>, next: &'a Cont<'a>) -> usize {
    let this_ptr = (this as *const Cont<'a>) as usize;
    let next_ptr = (next as *const Cont<'a>) as usize;
    unsafe { (this_ptr - next_ptr) / size_of::<Cont<'a>>() }
}

impl<'a> fmt::Display for Cont<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Cont::Call(callee, cont) => write!(f, "call: {} next: {}", callee, delta(self, cont)),
            &Cont::Func(names, args, body, cont) => write!(f, "func: {} {} next: {}", names, args, delta(self, cont)),
            &Cont::Cond(if_expr, else_expr, cont) => {
                write!(f,
                       "cond: {} {} next: {}",
                       if_expr,
                       else_expr,
                       delta(self, cont))
            }
            &Cont::Assign(name, cont) => write!(f, "assign: {} next: {}", name, delta(self, cont)),
            &Cont::Dict(acc, rest, cont) => write!(f, "dict: {} {} next: {}", acc, rest, delta(self, cont)),
            &Cont::Verb(ref verb, right, swap, cont) => {
                write!(f, "verb: {} {} next: {}", verb, right, delta(self, cont))
            }
            &Cont::Expressions(rest, cont) => write!(f, "expr: {} next: {}", rest, delta(self, cont)),
            x => write!(f, "return"),
        }
    }
}

impl<'a> fmt::Display for Lazy<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Lazy::Defer(node, ast, cont) => write!(f, "defer {} {} {}", node, ast, cont),
            x => write!(f, "return"),
        }
    }
}

impl<'a> Arena<'a> {
    pub fn new() -> Arena<'a> {
        let (builtins, asts) = Arena::init(UnsafeCell::new(Vec::with_capacity(2048 * 2048)));
        Arena {
            asts: asts,
            names: UnsafeCell::new(HashMap::new()),
            symbols: UnsafeCell::new(HashMap::new()),
            sequences: UnsafeCell::new(HashMap::new()),
            conts: UnsafeCell::new(Vec::with_capacity(2048 * 2048)),
            builtins: builtins,
        }
    }

    pub fn ast_len(&'a self) -> usize {
        unsafe { &*self.asts.get() }.len()
    }

    pub fn cont_len(&'a self) -> usize {
        unsafe { &*self.conts.get() }.len()
    }

    pub fn nil(&'a self) -> &'a AST<'a> {
        unsafe { &(*self.asts.get())[0] } // see Arena::init for details
    }

    pub fn any(&'a self) -> &'a AST<'a> {
        unsafe { &(*self.asts.get())[1] } // see Arena::init for details
    }

    //   pub fn yield_(&'a self) -> &'a AST<'a> {
    //       unsafe { &(*self.asts.get())[2] } // see Arena::init for details
    //   }

    pub fn dump(&'a self) {
        let x = unsafe { &mut *self.asts.get() };
        for i in x.iter() {
            println!("ast {}", i);
        }
        let x = unsafe { &mut *self.conts.get() };
        for i in x.iter() {
            println!("cont {}", i);
        }
    }

    pub fn ast(&self, n: AST<'a>) -> &'a AST<'a> {
        let ast = unsafe { &mut *self.asts.get() };
        ast.push(n);
        ast.last().unwrap()
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

    pub fn intern_symbol(&self, s: String) -> &'a AST<'a> {
        let symbols = unsafe { &mut *self.symbols.get() };
        if symbols.contains_key(&s) {
            self.ast(AST::Value(Value::SymbolInt(symbols[&s])))
        } else {
            let id = symbols.len() as u16;
            symbols.insert(s, id);
            self.ast(AST::Value(Value::SymbolInt(id)))
        }
    }
    pub fn intern_sequence(&self, s: String) -> &'a AST<'a> {
        let sequences = unsafe { &mut *self.sequences.get() };
        if sequences.contains_key(&s) {
            self.ast(AST::Value(Value::SequenceInt(sequences[&s])))
        } else {
            let id = sequences.len() as u16;
            sequences.insert(s, id);
            self.ast(AST::Value(Value::SequenceInt(id)))
        }
    }
    pub fn to_string(&self) {
        let ast = unsafe { &mut *self.asts.get() };
        println!("AST {}, {:?}", ast.len(), ast);
    }

    pub fn init(asts: UnsafeCell<Vec<AST<'a>>>) -> (u16, UnsafeCell<Vec<AST<'a>>>) {
        let a = unsafe { &mut *asts.get() };
        assert!(a.len() == 0);
        a.push(AST::Nil);       // Nil   - index 0
        a.push(AST::Any);       // Any   - index 1
//        a.push(AST::Yield(Context::Nil));     // Yield - index 2
        (a.len() as u16, asts)
    }

    pub fn clean(&self) -> usize {
        let asts = unsafe { &mut *self.asts.get() };
        let conts = unsafe { &mut *self.conts.get() };
        let l = conts.len() + asts.len();
        unsafe {
            asts.set_len(self.builtins as usize);
            conts.set_len(0);
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

struct vi64<'a>(&'a Vec<i64>);

impl<'a> fmt::Display for vi64<'a> {
    // cannot implement trait directly for Vec<i64> :(
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.0
            .into_iter()
            .map(|x| x.to_string())
            .fold(String::new(), |acc, x| if acc == "" {
                x
            } else {
                format!("{};{}", acc, x)
            });
        write!(f, "{}", str)
    }
}

struct vf64<'a>(&'a Vec<f64>);

impl<'a> fmt::Display for vf64<'a> {
    // cannot implement trait directly for Vec<i64> :(
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.0
            .into_iter()
            .map(|x| x.to_string())
            .fold(String::new(), |acc, x| if acc == "" {
                x
            } else {
                format!("{};{}", acc, x)
            });
        write!(f, "{}", str)
    }
}

impl<'a> fmt::Display for AST<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AST::Nil => write!(f, ""),
            AST::Any => write!(f, "Any"),
            AST::Cons(ref a, &AST::Nil) => write!(f, "{}", a),
            AST::Cons(ref a, ref b) => write!(f, "{} {}", a, b),
            AST::List(ref a) => write!(f, "({})", a),
            AST::Table(a, b) => write!(f, "([{}]{})", a, b),
            AST::Dict(ref d) => write!(f, "[{}]", d),
            AST::Call(ref a, ref b) => write!(f, "{} {}", a, b),
            AST::Lambda(_, a, b) => {
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
            AST::Assign(ref a, ref b) => write!(f, "{}:{}", a, b),
            AST::Cond(ref c, ref a, ref b) => write!(f, "$[{};{};{}]", c, a, b),
            AST::Yield(ref c) => write!(f, "Yield {:?}", c),
            AST::NameInt(ref n) => write!(f, "^{}", n),
            AST::Value(ref v) => {
                match v {
                    &Value::Number(n) => write!(f, "{}", n),
                    &Value::SymbolInt(ref s) => write!(f, "{}", s),
                    &Value::SequenceInt(ref s) => write!(f, "{:?}", s),
                    &Value::VecInt(ref v) => write!(f, "#i[{}]", vi64(v)),
                    &Value::VecFloat(ref v) => write!(f, "#f[{}]", vf64(v)),
                    // &Value::VecValue(ref v) => write!(f, "#a[{}]", v),
                    &Value::Ioverb(ref v) => write!(f, "{}", v),
                    _ => write!(f, "Not implemented yet."),
                }
            }
            _ => write!(f, "Not implemented yet."),
        }

    }
}

pub fn extract_name<'a>(a: &'a AST<'a>) -> u16 {
    match a {
        &AST::NameInt(s) => s,
        _ => 0,
    }
}

pub fn nil<'a>(arena: &'a Arena<'a>) -> &'a AST<'a> {
    arena.ast(AST::Nil)
}

pub fn any<'a>(arena: &'a Arena<'a>) -> &'a AST<'a> {
    arena.ast(AST::Any)
}

pub fn ast<'a>(n: AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    arena.ast(n)
}

pub fn cont<'a>(n: Cont<'a>, arena: &'a Arena<'a>) -> &'a Cont<'a> {
    arena.cont(n)
}

pub fn call<'a>(l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    ast(AST::Call(l, r), arena)
}

pub fn cons<'a>(l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    // if (l == r) && (l == &AST::Nil) {
    //    return arena.nil();
    // }
    ast(AST::Cons(l, r), arena)
}

pub fn fun<'a>(l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    match *l {
        AST::Nil => arena.ast(AST::Lambda(None, arena.intern("x".to_string()), r)),
        _ => arena.ast(AST::Lambda(None, l, r)),
    }
}

pub fn table<'a>(l: &'a AST<'a>, r: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    arena.ast(AST::Table(l, r))
}

pub fn dict<'a>(l: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    // println!("Dict: {:?}", l);
    match l {
        //        &AST::Cons(&AST::Nil, b) => arena.ast(AST::Cons(b, arena.nil())),
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

struct intlist_r {
    isvec: bool,
    len: usize,
}

fn is_intlist<'a>(l: &'a AST<'a>) -> intlist_r {
    // returns if cons-list contains only integers and its length

    let not_intlist = intlist_r {
        isvec: false,
        len: 0,
    };

    match l {
        &AST::Cons(a, b) => {
            let la = is_intlist(a);
            let lb = if la.isvec { is_intlist(b) } else { not_intlist };
            intlist_r {
                isvec: la.isvec && lb.isvec,
                len: la.len + lb.len,
            }
        }
        &AST::Nil => {
            intlist_r {
                isvec: true,
                len: 0,
            }
        }
        &AST::Value(ref x) => {
            match x {
                &Value::Number(i64) => {
                    intlist_r {
                        isvec: true,
                        len: 1,
                    }
                }
                &Value::Float(f64) => {
                    intlist_r {
                        isvec: true,
                        len: 1,
                    }
                }
                x => not_intlist,
            }
        }
        x => not_intlist,
    }
}

fn to_intlist<'a>(l: &'a AST<'a>, len: usize, arena: &'a Arena<'a>) -> &'a AST<'a> {
    // converts list of integers to specialized vector

    // println!("to_intlist : {:?} {}", l, len);
    let mut i: Vec<i64> = Vec::with_capacity(len);
    let mut f: Vec<f64> = Vec::with_capacity(len);
    for v in l.into_iter() {
        match v {
            &AST::Value(Value::Number(x)) => {
                i.push(x);
            }
            &AST::Value(Value::Float(x)) => {
                f.push(x);
            }
            _ => panic!("Unexpected non-number type"),
        }
    }
    if i.len() >= f.len() {
        arena.ast(AST::Value(Value::VecInt(i)))
    } else {
        arena.ast(AST::Value(Value::VecFloat(f)))
    }
}

fn postprocess<'a>(t: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    // AST postprocessing
    // - lists of integers => VecInt
    // - lists of floats   => VecFloat
    // - general lists     => VecAST

    // println!("postprocess input: {:?}", t);
    match t {
        &AST::List(l) => {
            // println!("postprocess List: {:?}", l);

            let li = is_intlist(l);
            if li.isvec {
                to_intlist(l, li.len, arena)
            } else {
                arena.ast(AST::List(l))
            }
        }
        &AST::Cons(a, b) => arena.ast(AST::Cons(postprocess(a, arena), postprocess(b, arena))),
        &AST::Dict(d) => arena.ast(AST::Dict(postprocess(d, arena))), 
        &AST::Assign(a, b) => arena.ast(AST::Assign(postprocess(a, arena), postprocess(b, arena))), 
        &AST::Call(a, b) => arena.ast(AST::Call(postprocess(a, arena), postprocess(b, arena))), 
        &AST::Cond(a, b, c) => {
            arena.ast(AST::Cond(postprocess(a, arena),
                                postprocess(b, arena),
                                postprocess(c, arena)))
        }
        &AST::Lambda(t, a, b) => arena.ast(AST::Lambda(t, postprocess(a, arena), postprocess(b, arena))),
        &AST::Verb(v, a, b) => arena.ast(AST::Verb(v, postprocess(a, arena), postprocess(b, arena))),
        &AST::Adverb(adv, a, b) => arena.ast(AST::Adverb(adv, postprocess(a, arena), postprocess(b, arena))),
        &AST::Table(a, b) => arena.ast(AST::Table(postprocess(a, arena), postprocess(b, arena))),
        x => arena.ast(x.clone()),       // nothing to postprocess
    }
}

pub fn parse<'a>(arena: &'a Arena<'a>, s: &String) -> &'a AST<'a> {
    postprocess(command::parse_Mex(arena, s).unwrap(), arena)
}

pub fn rev_list<'a>(l: &'a AST<'a>, arena: &'a Arena<'a>) -> &'a AST<'a> {
    let mut res = arena.ast(AST::Nil);
    let mut from = match l {
        &AST::List(xs) => xs,
        _ => panic!(),
    };
    // println!(" from {:?}", from);
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
        Verb::Dot => {
            match (l, r) {
                (&AST::Value(Value::Number(x)), &AST::Value(Value::Number(y))) => {
                    arena.ast(AST::Value(Value::Float(x as f64 + (y as f64 / 10.0))))
                }
                _ => arena.ast(AST::Verb(v, l, r)),
            }
        }
        Verb::Cast => {
            let rexpr = match r {
                &AST::Dict(d) => {
                    match d {
                        &AST::Cons(a, b) => {
                            match b {
                                &AST::Cons(t, f) => arena.ast(AST::Cond(a, t, f)),
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
    let mut res = arena.nil();
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
                    // &AST::Nil => {
                    //    res = arena.nil();
                    //    from = xs;
                    // }
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
