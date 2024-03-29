//! A thin wrapper around [`text_size`] to add some helper functions and types.

pub use text_size::{TextLen, TextRange, TextSize};

/// A value located in a text file.
#[derive(Debug, Clone, Copy)]
pub struct WithRange<T> {
  /// The value.
  pub val: T,
  /// The location.
  pub range: text_size::TextRange,
}

impl<T> WithRange<T> {
  /// Wrap a new value with the location from `self`.
  pub fn wrap<U>(&self, val: U) -> WithRange<U> {
    WithRange { val, range: self.range }
  }
}

/// Make a text size.
///
/// # Panics
///
/// If this failed, likely from an integer overflow.
#[must_use]
pub fn mk_text_size(n: usize) -> TextSize {
  TextSize::try_from(n).expect("could not make text size")
}
