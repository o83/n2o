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
* Zero-Copy Interpreter
* BERT protocol for VM stats

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

Test WebSocket Server
-------------------

Prerequisites:

```
$ sudo apt-get install webfs
$ brew install webfs
$ cd /usr/ports/www/webfs/ && make install clean
```

Start Server:

```
$ rlwrap ./target/debug/wserver
Listening on V4(127.0.0.1:9001)...
Message: [129, 132, 131, 146, 194, 183, 205, 160, 141, 155]
Message: [129, 132, 193, 71, 215, 107, 143, 117, 152, 71]
Message: [129, 132, 23, 14, 207, 245, 71, 71, 129, 178]
```

Open Browser:

```
$ open http://127.0.0.1:8001/etc/status/index.htm
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
* Ievgenii Lysiuchenko, Optimizations, System Programming
* Mykola Oleksiienko, K expertise
* Maxim Sokhatsky, General View

Inspiration
-----------
* Ken Pratt, Rusty Scheme
* Carl Lerche, MIO

