
// TODO: remove unstable features

#![feature(fnbox, static_in_const, heap_api, unboxed_closures, oom, alloc, box_syntax, optin_builtin_traits, const_fn, step_by)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_unsafe)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_features)]
#![allow(unused_mut)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

// TODO: remove these deps

extern crate net2;
extern crate nix;
extern crate alloc;
extern crate deque;
extern crate num_cpus;
extern crate rand;

#[macro_use]
pub mod session_types;
pub mod reactors;
pub mod timers;
pub mod io;
pub mod slab;
pub mod commands;
pub mod args;
pub mod iterators;
pub mod queues;

#[macro_use]
extern crate libc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate core;
#[macro_use]
extern crate bitflags;
