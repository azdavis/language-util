//! Pattern matching.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

mod matching;
mod matrix;
mod types;

pub use matching::{check, Check};
pub use types::{Lang, Pat, RawPat};
