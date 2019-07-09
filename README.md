The Kernel
==========

TL;DR Autobalancing low-latency non-blocking zero-copy CAS-multicursor queues with priority tasks and scalable timers.

[![Build Status](https://travis-ci.org/AlgoTradingHub/kernel.svg?branch=master)](https://travis-ci.org/AlgoTradingHub/kernel)
[![Gitter Chat](https://img.shields.io/gitter/room/badges/shields.svg)](https://gitter.im/voxoz/kernel)
[![Crates IO](https://img.shields.io/crates/d/kernel.svg)](https://crates.io/crates/kernel)
[![Crates IO](https://img.shields.io/crates/v/kernel.svg)](https://crates.io/crates/kernel)

Articles
--------

* [2016-11-15 O-CPS Интерпретатор](https://tonpa.guru/stream/2016/2016-11-15%20О-CPS%20Интерпретатор.htm)
* [2016-11-15 Векторный DSL](https://tonpa.guru/stream/2016/2016-11-15%20Векторный%20DSL.htm)
* [2016-11-25 Лямбда Калкулус на Стримах](https://tonpa.guru/stream/2016/2016-11-25%20Лямбда%20Калкулус%20на%20Стримах.htm)
* [2016-11-29 Архитектура К-подобного языка и его операционной среды](https://tonpa.guru/stream/2016/2016-11-29%20Архитектура%20К-подобного%20языка%20и%20его%20операционной%20среды.htm)
* [2016-11-29 Неизвестная Миру Лямбда Кодировка с CPS Континуатором](https://tonpa.guru/stream/2016/2016-11-29%20Неизвестная%20Миру%20Лямбда%20Кодировка%20с%20CPS%20Континуатором.htm)
* [2016-12-02 CPS Interpreter in Rust Language](https://tonpa.guru/stream/2016/2016-12-02%20CPS%20Interpreter%20in%20Rust%20Language.htm)
* [2016-12-06 Сферические факториалы](https://tonpa.guru/stream/2016/2016-12-06%20Сферические%20факториалы.htm)
* [2016-12-13 Замена Heap на преалоцированный Vec + Lifetime переменные](https://tonpa.guru/stream/2016/2016-12-13%20Замена%20Heap%20на%20преалоцированный%20Vec%20+%20Lifetime%20переменные.htm)
* [2016-12-21 Аккерман на О](https://tonpa.guru/stream/2016/2016-12-21%20Аккерман%20на%20О.htm)
* [2016-12-27 Аккерман в L1](https://tonpa.guru/stream/2016/2016-12-27%20Аккерман%20в%20L1.htm)
* [2016-12-30 Последний гвоздь в гроб Хаскеля и C++ в этом году!](https://tonpa.guru/stream/2016/2016-12-30%20Последний%20гвоздь%20в%20гроб%20Хаскеля%20и%20C++%20в%20этом%20году!.htm)
* [2017-01-01 O-CPS SPEC](https://tonpa.guru/stream/2017/2017-01-01%20O-CPS%20SPEC.htm)
* [2017-01-10 Safe and Static Methods for Memory Management](https://tonpa.guru/stream/2017/2017-01-10%20Safe%20and%20Static%20Methods%20for%20Memory%20Management.htm)
* [2017-01-12 Дефорестизационные Интерпретаторы O2](https://tonpa.guru/stream/2017/2017-01-12%20Дефорестизационные%20Интерпретаторы%20O2.htm)
* [2017-01-16 Генерация AVX mm256_mul_pd](https://tonpa.guru/stream/2017/2017-01-16%20Генерация%20AVX%20mm256_mul_pd.htm)
* [2017-01-17 Pony vs O-CPS](https://tonpa.guru/stream/2017/2017-01-17%20Pony%20vs%20O-CPS.htm)
* [2017-02-19 O-CPS +AVX2](https://tonpa.guru/stream/2017/2017-02-19%20O-CPS%20+AVX2.htm)

Features
--------

* MIO Compatible Network Server with Connections
* Vectorizable SMP-aware stream combinators
* MPSC, SPMC, SPSC queues with CAS-semantics on Ring Buffers
* Zero-Copy Interpreter and Queues
* Session Types and π-calculus semantics
* 5-20ns latency
* BERT protocol for VM stats
* AVX Vectorization
* Dedicated InterCore Bus Protocol (Star Topology)

The O Language
-------------------

```
$ cargo build ; rlwrap ./target/debug/o -init etc/init.q
    Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
AP core 3
AP core 2
AP core 1
BSP core 0
Welcome to The O Language 1.1.0
o)InterCore Exec 0 "a:pub 1;b:pub 2;c:pub 2;d:[a;b;c]\n\n" Yield(Nil)
InterCore Pub 2 1 Pub { from: 0, to: 1, task_id: 0, name: "", cap: 8 }
InterCore AckPub 0 AckPub { from: 1, to: 0, task_id: 0, result_id: 1 }
InterCore Pub 3 2 Pub { from: 0, to: 2, task_id: 0, name: "", cap: 8 }
InterCore AckPub 0 AckPub { from: 2, to: 0, task_id: 0, result_id: 1 }
InterCore Pub 3 2 Pub { from: 0, to: 2, task_id: 0, name: "", cap: 8 }
InterCore AckPub 0 AckPub { from: 2, to: 0, task_id: 0, result_id: 2 }

o)a:pub 2
InterCore Exec 0 "a:pub 2\n" Yield(Nil)
InterCore Pub 3 2 Pub { from: 0, to: 2, task_id: 0, name: "", cap: 8 }
InterCore AckPub 0 AckPub { from: 2, to: 0, task_id: 0, result_id: 3 }

o)fac:{$[x=1;1;x*fac[x-1]]};fac[20]
InterCore Exec 0 "fac:{$[x=1;1;x*fac[x-1]]};fac[20]\n" End(Node(Value(Number(2432902008176640000))))

o)a
InterCore Exec 0 "a\n" End(Node(Value(Number(3))))

o)(1;2;3)*(2;4;9)
InterCore Exec 0 "(1;2;3)*(2;4;9)\n" End(Node(Value(VecInt([2, 8, 27]))))
```

Enable AVX Vectorization
------------------------

```
$ cat ./cargo/config

[target.x86_64-unknown-linux-gnu]
rustflags="-C target-feature=+avx,+avx2"

$ cargo build --release

$ objdump ./target/release/o -d | grep mulpd
   223f1:	c5 f5 59 0c d3       	vmulpd (%rbx,%rdx,8),%ymm1,%ymm1
   223f6:	c5 dd 59 64 d3 20    	vmulpd 0x20(%rbx,%rdx,8),%ymm4,%ymm4
   22416:	c5 f5 59 4c d3 40    	vmulpd 0x40(%rbx,%rdx,8),%ymm1,%ymm1
   2241c:	c5 dd 59 64 d3 60    	vmulpd 0x60(%rbx,%rdx,8),%ymm4,%ymm4
   2264d:	c5 f5 59 0c d3       	vmulpd (%rbx,%rdx,8),%ymm1,%ymm1
   22652:	c5 e5 59 5c d3 20    	vmulpd 0x20(%rbx,%rdx,8),%ymm3,%ymm3
$ objdump ./target/release/o -d | grep vpmul
   2251c:	c5 d5 f4 fb          	vpmuludq %ymm3,%ymm5,%ymm7
   22525:	c4 41 55 f4 c0       	vpmuludq %ymm8,%ymm5,%ymm8
   2253a:	c5 d5 f4 db          	vpmuludq %ymm3,%ymm5,%ymm3
   22547:	c5 cd f4 ec          	vpmuludq %ymm4,%ymm6,%ymm5
   22550:	c5 cd f4 ff          	vpmuludq %ymm7,%ymm6,%ymm7
   22562:	c5 cd f4 e4          	vpmuludq %ymm4,%ymm6,%ymm4
   22595:	c5 d5 f4 fb          	vpmuludq %ymm3,%ymm5,%ymm7
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

Test WebSocket Server
-------------------

Prerequisites:

```
$ sudo apt-get install webfs
$ brew install webfs
$ cd /usr/ports/www/webfs/ && make install clean
```

Open Browser:

```
$ open http://127.0.0.1:8001/etc/status/index.htm
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
* Denis Golovan, Vectorization, KCell, Interpreter
* Mykola Oleksiienko, K expertise
* Maxim Sokhatsky, General View

Inspiration
-----------
* Ken Pratt, Rusty Scheme
* Carl Lerche, MIO

