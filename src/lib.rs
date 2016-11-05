
// TODO: remove unstable features

#![feature(fnbox, static_in_const, heap_api, unboxed_closures, oom, alloc, box_syntax, optin_builtin_traits, question_mark, const_fn)]

// TODO: remove these deps

extern crate net2;
extern crate nix;
extern crate alloc;
extern crate mio;
extern crate slab;

pub mod abstractions;
pub mod reactors;
pub mod timers;
pub mod io;

#[macro_use]
extern crate libc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate core;
#[macro_use]
extern crate bitflags;

