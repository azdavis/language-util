//! Unique identifiers.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![no_std]

/// A unique identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uniq(u32);

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
