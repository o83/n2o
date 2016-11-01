Enso Operating System
=====================

[![Build Status](https://travis-ci.org/AlgoTradingHub/kernel.svg?branch=master)](https://travis-ci.org/AlgoTradingHub/kernel)
[![Gitter Chat](https://img.shields.io/gitter/room/badges/shields.svg)](https://gitter.im/voxoz/kernel)

Features
--------

* MIO Compatible Network Server with Connections
* Future and Stream rich combinators
* Session Types and π-calculus semantics
* MPSC, SPMC, SPSC queues
* 10-40ns latency
* Free from Dependencies

Test Network Server
-------------------

```
  $ cargo build
  $ cargo test
  $ ./target/debug/server
IO Server started
registering with poller
registering; token=Token(10000000); interests=Ready {Readable}
Server run loop starting...
```

In another process:

```
  $ ./target/debug/client
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

* Viktor Sovietov
* Anton Kundenko
* Ievgenii Lysiuchenko
* Mykola Oleksiienko
* Maxim Sokhatsky
