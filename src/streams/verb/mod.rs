pub mod dot;

use commands::ast::*;

// workaround for quoting operators
// see https://github.com/rust-lang/rust/issues/8853
macro_rules! op {
    ($e:expr) => {
        $e
    }
}

macro_rules! dyad_map_expr {
    ($l:tt, =, $r:tt,Float) => {
        if ($l - $r).abs() < 1e-10 { 1 } else { 0 } // TODO: Float tolerance
    };
    ($l:tt, =, $r:tt,Number) => {
        if $l == $r { 1 } else { 0 }
    };
    ($l:tt, !=, $r:tt,Float) => {
        if ($l - $r).abs() > 1e-10 { 1 } else { 0 } // TODO: Float tolerance
    };
    ($l:tt, !=, $r:tt,Number) => {
        if $l != $r { 1 } else { 0 }
    };
    ($l:tt, >, $r:tt, $sink:tt) => {
        if $l > $r { 1 } else { 0 }
    };
    ($l:tt, >=, $r:tt, $sink:tt) => {
        if $l >= $r { 1 } else { 0 }
    };
    ($l:tt, <, $r:tt, $sink:tt) => {
        if $l < $r { 1 } else { 0 }
    };
    ($l:tt, <=, $r:tt, $sink:tt) => {
        if $l <= $r { 1 } else { 0 }
    };
    ($l:tt, $op:tt, $r:tt, $sink:tt) => {
        // all simple operators
        op!($l $op $r)
    };
}

macro_rules! dyad_arith_match{
    ( $name: expr, $l:expr, $r:expr, $( [ $op:tt, $atype:tt, $atom:ident, $vec:ident, $r_atom:ident, $r_vec:ident ] ),* ) => {
        match ($l, $r) {
            $(
                (&AST::Atom(Atom::Value(Value::$atom(l))),
                 &AST::Atom(Atom::Value(Value::$atom(r)))) =>
                    Ok( AST::Atom(Atom::Value(Value::$r_atom(dyad_map_expr!(l,$op,r,$atom)))) ),

                (&AST::Atom(Atom::Value(Value::$atom(ref l))),
                 &AST::Atom(Atom::Value(Value::$vec(ref r)))) => {
                    let a: Vec<$atype> = r.iter()
                        .map(|x| dyad_map_expr!(l,$op,x,$atom))
                        .collect::<Vec<$atype>>();
                    Ok( AST::Atom(Atom::Value(Value::$r_vec(a))) )
                },

                (&AST::Atom(Atom::Value(Value::$vec(ref l))),
                 &AST::Atom(Atom::Value(Value::$atom(ref r)))) => {
                    let a: Vec<$atype> = l.iter()
                        .map(|x| dyad_map_expr!(x,$op,r,$atom))
                        .collect::<Vec<$atype>>();
                    Ok( AST::Atom(Atom::Value(Value::$r_vec(a))) )
                },

                (&AST::Atom(Atom::Value(Value::$vec(ref l))),
                 &AST::Atom(Atom::Value(Value::$vec(ref r)))) => {
                    let a: Vec<$atype> = l.iter()
                        .zip(r)
                        .map(|(l,r)| dyad_map_expr!(l,$op,r,$atom))
                        .collect::<Vec<$atype>>();
                    Ok( AST::Atom(Atom::Value(Value::$r_vec(a))) )
                },                
            )*
            _ =>
                Err(Error::EvalError {
                    desc: format!("{} not supported", $name),
                    ast: format!("{:?} {:?}", $l, $r),
                })
        }
    }
}

macro_rules! dyad_arith{
    ($module:ident, $name:tt, $op:tt, $atype:tt, $r_atom:ident, $r_vec:ident) => {
        mod $module {
            use commands::ast::Value;
            use commands::ast::{AST, Atom, Error};

            pub fn eval<'a>(l: &'a AST<'a>, r: &'a AST<'a>) -> Result<AST<'a>, Error> {

                dyad_arith_match!( $name, l, r,
                                   [$op, $atype, Number, VecInt, $r_atom, $r_vec],
                                   [$op, $atype, Float, VecFloat, $r_atom, $r_vec] )
            }
        }
    };
    ($module:ident, $name:tt, $op:tt) => {
        mod $module {
            use commands::ast::Value;
            use commands::ast::{AST, Atom, Error};

            pub fn eval<'a>(l: &'a AST<'a>, r: &'a AST<'a>) -> Result<AST<'a>, Error> {

                dyad_arith_match!( $name, l, r,
                                   [$op, i64, Number, VecInt, Number, VecInt],
                                   [$op, f64, Float, VecFloat, Float, VecFloat] )
            }
        }
    }
}

dyad_arith!(plus, "Dyad plus", +);
dyad_arith!(minus, "Dyad minus", -);
dyad_arith!(mul, "Dyad mul", *);
dyad_arith!(div, "Dyad div", /);
dyad_arith!(eq, "Dyad eq", =, i64, Number, VecInt);
dyad_arith!(neq, "Dyad neq", !=, i64, Number, VecInt);
dyad_arith!(gt, "Dyad gt", >, i64, Number, VecInt);
dyad_arith!(ge, "Dyad ge", >=, i64, Number, VecInt);
dyad_arith!(lt, "Dyad lt", <, i64, Number, VecInt);
dyad_arith!(le, "Dyad le", <=, i64, Number, VecInt);

pub fn eval<'ast>(verb: Verb, left: &'ast AST<'ast>, right: &'ast AST<'ast>) -> Result<AST<'ast>, Error> {
    match verb {
        Verb::Plus => plus::eval(left, right),
        Verb::Minus => minus::eval(left, right),
        Verb::Times => mul::eval(left, right),
        Verb::Divide => div::eval(left, right),
        Verb::Eq => eq::eval(left, right),
        Verb::NEq => neq::eval(left, right),
        Verb::Gt => gt::eval(left, right),
        Verb::Ge => ge::eval(left, right),
        Verb::Lt => lt::eval(left, right),
        Verb::Le => le::eval(left, right),
        x => {
            Err(Error::EvalError {
                desc: "Verb is not implemented".to_string(),
                ast: format!("{:?} {:?} {:?}", verb, left, right),
            })
        }
    }
}
