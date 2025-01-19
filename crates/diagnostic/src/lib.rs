//! Utilities for diagnostics (colloquially, "errors").

use std::fmt;

/// A diagnostic message about a bit of code.
#[derive(Debug)]
pub struct Diagnostic {
  /// The range of the file this diagnostic applies to.
  pub range: text_pos::RangeUtf16,
  /// The message of the diagnostic.
  pub message: String,
  /// The severity.
  pub severity: Severity,
}

impl fmt::Display for Diagnostic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}: {}: {}", self.range, self.severity, self.message)
  }
}

/// The severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
  /// Should maybe be addressed, but can compile without addressing.
  Warning,
  /// Can't compile unless addressed.
  Error,
}

impl fmt::Display for Severity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Severity::Warning => f.write_str("warning"),
      Severity::Error => f.write_str("error"),
    }
  }
}

/// A diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Code(u16);

impl Code {
  /// Returns a Code for this.
  #[must_use]
  pub fn n(n: u16) -> Self {
    Self(n)
  }

  /// Return this as an [`i32`].
  #[must_use]
  pub fn as_i32(&self) -> i32 {
    self.0.into()
  }
}

impl fmt::Display for Code {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl std::str::FromStr for Code {
  type Err = ParseCodeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match u16::from_str(s) {
      Ok(n) => Ok(Self(n)),
      Err(e) => Err(ParseCodeError(e)),
    }
  }
}

/// An error when a [`Code`] could not be parsed from a str.
#[derive(Debug)]
pub struct ParseCodeError(std::num::ParseIntError);

impl fmt::Display for ParseCodeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "couldn't parse code: {}", self.0)
  }
}

impl std::error::Error for ParseCodeError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&self.0)
  }
}
