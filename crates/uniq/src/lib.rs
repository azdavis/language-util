//! Unique identifiers.

#![deny(clippy::pedantic, missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![no_std]

use core::fmt;

/// A unique identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uniq(u32);

impl fmt::Display for Uniq {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// A generator for [`Uniq`]s.
#[derive(Debug, Default)]
pub struct UniqGen(u32);

impl UniqGen {
  /// Returns a [`Uniq`] not equal to any other [`Uniq`] returned thus far from
  /// this [`UniqGen`].
  ///
  /// # Example
  ///
  /// ```
  /// # use uniq::UniqGen;
  /// let mut uniq_gen = UniqGen::default();
  /// let u1 = uniq_gen.gen();
  /// let u2 = uniq_gen.gen();
  /// assert_ne!(u1, u2);
  /// ```
  pub fn gen(&mut self) -> Uniq {
    let ret = Uniq(self.0);
    // assuming overflow won't happen.
    self.0 += 1;
    ret
  }
}
