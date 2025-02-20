//! Unique identifiers.

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

/// A maker for [`Uniq`]s.
#[derive(Debug, Default)]
pub struct UniqMk(u32);

impl UniqMk {
  /// Returns a [`Uniq`] not equal to any other [`Uniq`] returned thus far from
  /// this [`UniqMk`].
  ///
  /// # Example
  ///
  /// ```
  /// # use uniq::UniqMk;
  /// let mut uniq_gen = UniqMk::default();
  /// let u1 = uniq_gen.mk();
  /// let u2 = uniq_gen.mk();
  /// assert_ne!(u1, u2);
  /// ```
  pub fn mk(&mut self) -> Uniq {
    let ret = Uniq(self.0);
    // assuming overflow won't happen.
    self.0 += 1;
    ret
  }
}
