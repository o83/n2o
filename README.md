The Kernel
==========

TL;DR Autobalancing low-latency non-blocking zero-copy CAS-multicursor queues with priority tasks and scalable timers.

[![Build Status](https://travis-ci.org/AlgoTradingHub/kernel.svg?branch=master)](https://travis-ci.org/AlgoTradingHub/kernel)
[![Gitter Chat](https://img.shields.io/gitter/room/badges/shields.svg)](https://gitter.im/voxoz/kernel)
[![Crates IO](https://img.shields.io/crates/d/kernel.svg)](https://crates.io/crates/kernel)
[![Crates IO](https://img.shields.io/crates/v/kernel.svg)](https://crates.io/crates/kernel)

Features
--------

* MIO Compatible Network Server with Connections
* Vectorizable SMP-aware stream combinators
* MPSC, SPMC, SPSC queues with CAS-semantics on Ring Buffers
* Session Types and π-calculus semantics
* 10-40ns latency
* Minimal Dependencies
* Zero-Copy Interpreter

Prerequisites
-------------

```
$ sudo apt-get install libhwloc-dev
```

Test The O Language
-------------------

```
$ cargo build ; rlwrap ./target/debug/console 
    Finished debug [unoptimized + debuginfo] target(s) in 0.0 secs
Welcome to O-CPS Interpreter v0.11.0!
> fac:{$[x=1;1;x*fac[x-1]]};fac[20]
2432902008176640000
>
```

Sample
------

```rust
extern crate kernel;

use kernel::io::poll::*;
use kernel::io::tcp::*;
use kernel::io::server::*;
use kernel::io::console::*;

fn main() {
    let x = std::thread::spawn(|| net());
    let y = std::thread::spawn(|| console());
    x.join();
}

fn net() {
    let addr = "127.0.0.1:8000".parse::<std::net::SocketAddr>().ok().expect("Parser Error");
    let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut net = Server::new(sock);
    net.run(&mut poll).expect("Failed to run server");
}

fn console() {
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut con = Console::new();
    con.run(&mut poll);
}
```


Test Network Server
-------------------

```
$ cargo build
$ cargo test
$ ./target/debug/server
IO Server started
Console is listening...
Server run loop starting...
```

In another process:

```
$ ./target/debug/client
```

Test Console Server
-------------------

```
$ rlwrap ./target/debug/console
Console is listening...
ENSO
Message: "ENSO"
```

Test Session Types
------------------

```
$ ./target/debug/fix
```

Reading
-------

* Kohei Honda Session Types and π-calculus http://mrg.doc.ic.ac.uk/kohei/
* Rust version http://munksgaard.me/laumann-munksgaard-larsen.pdf
* Haskell Version http://users.eecs.northwestern.edu/~jesse/pubs/haskell-session-types/session08.pdf

Credits
-------

* Viktor Sovietov, Core Suggestions
* Anton Kundenko, Stream Processing
* Ievgenii Lysiuchenko, Optimizations
* Mykola Oleksiienko, K expertise
* Maxim Sokhatsky, General View
* Ken Pratt, Rusty Scheme
* Carl Lerche, MIO
