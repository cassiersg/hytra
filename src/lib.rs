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

#![deny(missing_docs)]

//! Hytra
//! A beast that eats your data from many threads.
//!
//! The main type in this library is [`TrAcc`], which allows you to accumulate data in a single
//! variable from multiple threads extremely fast. A specialized version is [`TrAdder`], that
//! contains an sum.
//!
//! Hytra has been inspired by Java's
//! [`LongAccumulator`](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/LongAccumulator.html),
//! [`DoubleAccumulator`](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/DoubleAccumulator.html),
//! [`LongAdder`](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/LongAdder.html)
//! and
//! [`DoubleAdder`](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/atomic/DoubleAdder.html).
//!
//!  [`TrAcc`]: struct.TrAcc.html
//!  [`TrAdder`]: struct.TrAdder.html

use atomic::Atomic;
use crossbeam_utils::CachePadded;
use num_traits::Zero;
use std::ops::Deref;
use std::sync::atomic::Ordering;
use thread_local::ThreadLocal;

/// This is workaround for the fact that the Fn trait is not stable.
/// We could have `TrAcc<T, F: Fn(T, T) -> T>`.  However, since the `Fn` trait is not stable, this
/// would not allow to have `TrAcc` for an accumulator other than a closure (which makes the type
/// un-namable) or a function pointer (which means dynamic dispatch).
/// The `FnAcc` is a custom trait that we use as a purpose-specific variant of `Fn(T, T) -> T`.
pub trait FnAcc<T> {
    /// Call the function.
    fn call(&self, arg1: T, arg2: T) -> T;
}

impl<T, U: Fn(T, T) -> T> FnAcc<T> for U {
    fn call(&self, arg1: T, arg2: T) -> T {
        self(arg1, arg2)
    }
}

/// The threaded accumulator allows to accumulate data in a single state from multiple threads
/// without contention, which allows performance to scale well with the number of
/// thread/processors.
///
/// The accumulation function must be associative an commutative, and the `identity` element must be
/// the neutral element w.r.t. the accumulation function.
///
/// The accumulated state can be any `Copy + Send` state, and the implementation uses atomic
/// instructions if supported by the architecture for the size of `T` (in which case this
/// datastructure is lock-free), and mutexes otherwise.
///
/// This optimizes for accumulation speed, at the expense of increased memory usage (the state is
/// replicated once for each thread) and cost of reading the accumulated state (which has to walk
/// over the states of each thread).
///
/// ```rust
/// use hytra::TrAcc;
/// let acc: TrAcc<i64, _> = TrAcc::new(|a, b| a*b, 1);
/// let acc_ref = &acc;
/// crossbeam_utils::thread::scope(|s| {
///     for j in 1..=2 {
///         s.spawn(move |_| {
///             for i in 1..=3 {
///                 acc_ref.acc(i*j);
///             }
///         });
///     }
/// })
/// .unwrap();
/// assert_eq!(acc.get(), (1*2*3)*((2*1)*(2*2)*(2*3)));
/// ```
#[derive(Debug)]
pub struct TrAcc<T: Copy + Send, F: FnAcc<T>> {
    state: ThreadLocal<CachePadded<Atomic<T>>>,
    acc_fn: F,
    identity: T,
}

impl<T: Copy + Send, F: Sync + FnAcc<T>> TrAcc<T, F> {
    /// Create a a `TrAcc`.
    pub fn new(acc_fn: F, identity: T) -> Self {
        Self {
            state: ThreadLocal::new(),
            acc_fn,
            identity,
        }
    }
    /// Accumulate `x`. If `state` is the current state, the new state is `fn_acc(state, x)`.
    ///
    /// This function has `Release` semantic w.r.t. the accumulator.
    pub fn acc(&self, x: T) {
        // Since writes to the thread-local are uncontented, we can have a relaxed load.
        let local_acc: &Atomic<T> = self
            .state
            .get_or(|| CachePadded::new(Atomic::new(self.identity)))
            .deref();
        let res = self.acc_fn.call(local_acc.load(Ordering::Relaxed), x);
        // We use a release for the store such that it synchronizes with the acquire of the get
        // function.
        local_acc.store(res, Ordering::Release);
    }
    /// Return the current state.
    ///
    /// This function has `Acquire` semantic w.r.t. the accumulator.
    pub fn get(&self) -> T {
        // The Acquire ordering synchronizes with the acc function.
        return self
            .state
            .iter()
            .map(|x| x.load(Ordering::Acquire))
            .fold(self.identity, |a, b| self.acc_fn.call(a, b));
    }
}

struct Adder<T>(std::marker::PhantomData<fn() -> T>);

impl<T: std::ops::Add<T, Output = T>> FnAcc<T> for Adder<T> {
    fn call(&self, arg1: T, arg2: T) -> T {
        <T as std::ops::Add>::add(arg1, arg2)
    }
}

/// The threaded add allows to increment and decrement an integer from multiple threads without
/// contention, which allows performance to scale well with the number of
/// thread/processors. `TrAdder` can wrap any primitive integer type.
///
/// **Overflow behavior.**
/// Overflow may occur if the sum of the increments in any subset of the threads overflows, even if
/// the total leads to no overflow. Overflow semantic is the same as for primitive types (panic or
/// wrapping).
///
/// See [`TrAcc`] for a discussion of performance characteristics.
///
/// ```rust
/// use hytra::TrAdder;
/// let adder: TrAdder<i64> = TrAdder::new();
/// crossbeam_utils::thread::scope(|s| {
///     for _ in 0..10 {
///         s.spawn(|_| {
///             for _ in 0..10 {
///                 adder.inc(1);
///             }
///         });
///     }
/// })
/// .unwrap();
/// assert_eq!(adder.get(), 100);
/// ```
///
///  [`TrAcc`]: struct.ThreadLocal.html
pub struct TrAdder<T: Copy + Zero + std::ops::Add<T, Output = T> + Send>(TrAcc<T, Adder<T>>);

impl<T: Copy + Zero + std::ops::Add<T, Output = T> + Send> TrAdder<T> {
    /// Create a new `TrAdder` initialized to 0.
    pub fn new() -> Self {
        Self(TrAcc::new(Adder(Default::default()), T::zero()))
    }
    /// Increment the `TrAdder`.
    pub fn inc(&self, x: T) {
        self.0.acc(x);
    }
    /// Return the value of the `TrAdder`.
    pub fn get(&self) -> T {
        self.0.get()
    }
}

impl<T: Copy + Zero + std::ops::Add<T, Output = T> + Send> Default for TrAdder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_adder_single_thread() {
    let adder: TrAdder<i64> = TrAdder::new();
    for i in 0..10i64 {
        assert_eq!(adder.get(), i);
        adder.inc(1);
    }
}
