
// O: The Language

grammar;
use commands::ast;
use commands::ast::*;
use core::str::FromStr;
pub Mex:   AST = { ExprList };

// Three Type of Brackets and Their Lists 9 LOC

List:      AST = { "(" ")" => AST::Nil, "(" <l:ExprList> ")" => list(l), };
Dict:      AST = { "[" "]" => AST::Nil, "[" <l:ExprList> "]" => dict(l), };

Lambda:    AST = { "{" <m:ExprList> "}"                      => fun(AST::Nil,m),
                   "{[" <c:NameList> "]" <m:ExprList> "}"    => fun(c,m),
                   "{" "}"                                   => fun(AST::Nil,AST::Nil)};

Call:      AST = { <c:Noun> <a:CallCont>                     => call(c,a), };
NameList:  AST = { Name, <o:Name> <m:NameCont>               => cons(o,m), };
ExprList:  AST = { ExprCont, Expr, <o:Expr> <m:ExprCont>     => cons(o,m), };

CallCont:  AST = { Noun, <m:Call>                            => m };
NameCont:  AST = { ";" => AST::Nil, ";" <m:NameList>         => m };
ExprCont:  AST = { ";" => AST::Nil, ";" <m:ExprList>         => m };


// Expressions and Statements 4 LOC

Noun:      AST = { Name, Number, Hexlit, Bool, Symbol, List, Dict, Sequence, Lambda, Ioverb };
Expr:      AST = { Noun, Call, Verbs, Adverbs, };

Verbs:     AST = {           <v:Verb>               => verb(v,AST::Nil,AST::Nil),
                             <v:Verb>     <r:Expr>  => verb(v,AST::Nil,r),
                   <l:Call>  <v:Verb>     <r:Expr>  => verb(v,l,r),
                   <l:Noun>  <v:Verb>     <r:Expr>  => verb(v,l,r), };

Adverbs:   AST = {           <a:Adverb>             => adverb(a,AST::Nil,AST::Nil),
                             <a:Adverb>   <e:Expr>  => adverb(a,AST::Nil,e),
                   <l:Call>  <a:Adverb>   <r:Expr>  => adverb(a,l,r),
                   <l:Noun>  <a:Adverb>   <r:Expr>  => adverb(a,l,r), };

// NOM

Number:    AST = { <n:r"\d+">                         => AST::Number(u64::from_str(n).unwrap()), };
Hexlit:    AST = { <h:r"0x[a-zA-Z\d]+">               => AST::Number(u64::from_str_radix(&h[2..], 16).unwrap()), };
Bool:      AST = { <b:r"[01]+b">                      => AST::Number(u64::from_str_radix(&b[0..b.len()-1], 2).unwrap()), };
Name:      AST = { <n:r"[a-zA-Z][-a-zA-Z\d]*">        => AST::Name(String::from(n)), };
Symbol:    AST = { <s:r"`([a-z][a-z0-9]*)?">          => AST::Symbol(String::from(&s[1..s.len()])), };
Adverb: Adverb = { <a:r"[\x27:\x5C\x2F]:?">           => Adverb::from_str(a).unwrap(), };
Verb:     Verb = { <v:r"[+\x2D*$%!&|<>=~,^#_?@.]">    => Verb::from_str(v).unwrap(), };
Ioverb:    AST = { <i:r"\d+:">                        => AST::Ioverb(String::from(i),Box::new(AST::Nil)), };
Sequence:  AST = { <s:r"\x22(\\.|[^\x5C\x22])*\x22">  => AST::Sequence(String::from(&s[1..s.len()-1])), };
