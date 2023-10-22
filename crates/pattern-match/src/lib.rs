//! Pattern matching.
//!
//! Adapted from ["Warnings for pattern matching"][1] by Luc Maranget, and
//! [`rustc`][2].
//!
//! [1]: http://moscova.inria.fr/~maranget/papers/warn/
//! [2]: https://github.com/rust-lang/rust/tree/master/compiler/rustc_mir_build/src/thir/pattern

#![deny(clippy::pedantic, missing_debug_implementations, missing_docs, rust_2018_idioms)]

mod alg;
mod matrix;
mod types;

pub use alg::check;
pub use types::{Check, CheckError, ConPat, Lang, Pat, RawPat, Result};
