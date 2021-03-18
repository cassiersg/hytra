// Copyright 2021 UCLouvain
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT License <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option.  This file may not be
// copied, modified, or distributed except according to those terms.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
use crossbeam_utils::thread::scope;
use hytra::{TrAcc, TrAdder};
use std::sync::atomic::{AtomicI64, Ordering};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Single counter implemented with the fetch_add function on std::sync::atomic::AtomicI64.
fn fetch_add(tot: i64, n: u32) -> i64 {
    let res = AtomicI64::new(0);
    scope(|s| {
        for _ in 0..n {
            s.spawn(|_| {
                for _ in 0..black_box(tot) {
                    res.fetch_add(black_box(1), Ordering::Relaxed);
                }
            });
        }
    })
    .unwrap();
    return res.into_inner();
}

/// A local counter in each thread (manual implementation).
fn simple_add(tot: i64, n: u32) -> i64 {
    scope(|s| {
        let handles = (0..n)
            .map(|_| {
                s.spawn(|_| {
                    let mut res = 0;
                    for _ in 0..black_box(tot) {
                        res = black_box(black_box(res) + black_box(1));
                    }
                    return res;
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .sum::<i64>()
    })
    .unwrap()
}

/// Counter with `TrAdd`.
fn tradd(tot: i64, n: u32) -> i64 {
    let res = TrAdder::new();
    scope(|s| {
        for _ in 0..n {
            s.spawn(|_| {
                for _ in 0..black_box(tot) {
                    res.inc(black_box(1));
                }
            });
        }
    })
    .unwrap();
    return res.get();
}

/// Counter with `TrAcc` and a function pointer. This is the same as `TrAdd` only if the compiler
/// is able to de-virtualize the function pointer call.
/// This seems to be the case most of the time, but not always.
fn tradd_fn(tot: i64, n: u32) -> i64 {
    let res = tradd::new(<i64 as std::ops::Add>::add, 0);
    scope(|s| {
        for _ in 0..n {
            s.spawn(|_| {
                for _ in 0..black_box(tot) {
                    res.acc(black_box(1));
                }
            });
        }
    })
    .unwrap();
    return res.get();
}

const N_ADDS: i64 = 1_000_000;

fn bench_simple_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("Adder");
    for i in [1, 2, 4].iter() {
        group.bench_with_input(BenchmarkId::new("fetch_add", i), i, |b, i| {
            b.iter(|| fetch_add(black_box(N_ADDS), *i))
        });
    }
    for i in [1, 2, 4, 8, 16, 32].iter() {
        group.bench_with_input(BenchmarkId::new("simple_add", i), i, |b, i| {
            b.iter(|| simple_add(black_box(N_ADDS), *i))
        });
        group.bench_with_input(BenchmarkId::new("TrAdder", i), i, |b, i| {
            b.iter(|| tradd(black_box(N_ADDS), *i))
        });
        group.bench_with_input(BenchmarkId::new("tradd fn add", i), i, |b, i| {
            b.iter(|| tradd_fn(black_box(N_ADDS), *i))
        });
    }
}

criterion_group!(benches, bench_simple_add);
criterion_main!(benches);
