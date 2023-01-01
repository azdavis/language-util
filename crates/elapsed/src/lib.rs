//! Track how long function calls take.

#![deny(clippy::pedantic, missing_debug_implementations, missing_docs, rust_2018_idioms)]

use std::time::{Duration, Instant};

/// Calls `f` and returns the result and the duration it took to do the call.
pub fn time<F, T>(f: F) -> (T, Duration)
where
  F: FnOnce() -> T,
{
  let start = Instant::now();
  let res = f();
  let elapsed = Instant::now().duration_since(start);
  (res, elapsed)
}

/// Calls `f` and logs the message and the time it took to do so at the Info level.
pub fn log<F, T>(msg: &str, f: F) -> T
where
  F: FnOnce() -> T,
{
  let (ret, elapsed) = time(f);
  log::info!("{msg}: {elapsed:?}");
  ret
}
