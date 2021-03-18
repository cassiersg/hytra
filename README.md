hytra
=====

[![Build Status](https://travis-ci.org/Amanieu/thread_local-rs.svg?branch=master)](https://travis-ci.org/Amanieu/thread_local-rs)
[![Crates.io](https://img.shields.io/crates/v/hytra.svg)](https://crates.io/crates/hytra)
[CI](https://github.com/cassiersg/hytra/actions)

*A beast that eats your data from many threads.*

This library provides the
[`TrAcc`](https://docs.rs/hytra/*/hytra/struct.TrAcc.html) type, which allows
you to accumulate data in a single logical variable at maximum performance:
each thread uses its own copy of the data, which removes conention slowdown.

Hytra has been inspired by Java's
[`LongAccumulator`](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/LongAccumulator.html),
[`DoubleAccumulator`](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/DoubleAccumulator.html),
[`LongAdder`](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/LongAdder.html)
and
[`DoubleAdder`](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/DoubleAdder.html).

[Documentation](https://docs.rs/hytra/)

## Minimum Rust version

This crate's minimum supported Rust version (MSRV) is 1.36.0.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

