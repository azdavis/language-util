//! See [`PositionDb`]. Heavily adapted from rust-analyzer.

#[cfg(test)]
mod tests;

use std::fmt;
use text_size_util::{TextRange, TextSize};

/// Converts between flat [`TextSize`] offsets and `(line, col)` representation, with handling for
/// both UTF-8 and UTF-16 encoded text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionDb {
  inner: line_index::LineIndex,
}

impl PositionDb {
  /// Returns a new `PositionDb` for the text.
  #[must_use]
  pub fn new(text: &str) -> Self {
    Self { inner: line_index::LineIndex::new(text) }
  }

  /// Returns the `PositionUtf16` for this `TextSize`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn position_utf16(&self, text_size: TextSize) -> Option<PositionUtf16> {
    self.position_to_utf16(self.position_utf8(text_size)?)
  }

  /// Returns the `TextSize` for this `PositionUtf16`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn text_size_utf16(&self, pos: PositionUtf16) -> Option<TextSize> {
    self.text_size_utf8(self.position_to_utf8(pos)?)
  }

  /// Returns the `RangeUtf16` for this `TextRange`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn range_utf16(&self, text_range: TextRange) -> Option<RangeUtf16> {
    self.range_to_utf16(self.range_utf8(text_range)?)
  }

  /// Returns the `TextRange` for this `RangeUtf16`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn text_range_utf16(&self, range: RangeUtf16) -> Option<TextRange> {
    self.text_range_utf8(self.range_to_utf8(range)?)
  }

  fn position_utf8(&self, text_size: TextSize) -> Option<PositionUtf8> {
    let lc = self.inner.try_line_col(text_size)?;
    Some(PositionUtf8 { line: lc.line, col: lc.col })
  }

  fn text_size_utf8(&self, pos: PositionUtf8) -> Option<TextSize> {
    let lc = line_index::LineCol { line: pos.line, col: pos.col };
    self.inner.offset(lc)
  }

  fn range_utf8(&self, text_range: TextRange) -> Option<RangeUtf8> {
    Some(RangeUtf8 {
      start: self.position_utf8(text_range.start())?,
      end: self.position_utf8(text_range.end())?,
    })
  }

  fn text_range_utf8(&self, range: RangeUtf8) -> Option<TextRange> {
    Some(TextRange::new(self.text_size_utf8(range.start)?, self.text_size_utf8(range.end)?))
  }

  fn position_to_utf16(&self, pos: PositionUtf8) -> Option<PositionUtf16> {
    let lc = line_index::LineCol { line: pos.line, col: pos.col };
    let wide = self.inner.to_wide(line_index::WideEncoding::Utf16, lc)?;
    Some(PositionUtf16 { line: wide.line, col: wide.col })
  }

  fn position_to_utf8(&self, pos: PositionUtf16) -> Option<PositionUtf8> {
    let wide = line_index::WideLineCol { line: pos.line, col: pos.col };
    let lc = self.inner.to_utf8(line_index::WideEncoding::Utf16, wide)?;
    Some(PositionUtf8 { line: lc.line, col: lc.col })
  }

  fn range_to_utf16(&self, range: RangeUtf8) -> Option<RangeUtf16> {
    Some(RangeUtf16 {
      start: self.position_to_utf16(range.start)?,
      end: self.position_to_utf16(range.end)?,
    })
  }

  fn range_to_utf8(&self, range: RangeUtf16) -> Option<RangeUtf8> {
    Some(RangeUtf8 {
      start: self.position_to_utf8(range.start)?,
      end: self.position_to_utf8(range.end)?,
    })
  }

  /// Returns an iterator over the lines in the range.
  pub fn lines(&self, range: TextRange) -> impl Iterator<Item = TextRange> + '_ {
    self.inner.lines(range)
  }

  /// Returns the length of the original text.
  #[must_use]
  pub fn len(&self) -> TextSize {
    self.inner.len()
  }

  /// Returns the end position of the original input.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn end_position_utf16(&self) -> PositionUtf16 {
    let pos = self.position_utf8(self.len()).expect("len is in range");
    self.position_to_utf16(pos).expect("len can be utf-16")
  }
}

/// A pair of `(line, col)` for UTF-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PositionUtf8 {
  /// Zero-based.
  line: u32,
  /// Zero-based utf8 offset.
  col: u32,
}

/// A pair of start and end positions for UTF-8.
///
/// `start` comes before `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RangeUtf8 {
  /// The start.
  start: PositionUtf8,
  /// The end.
  end: PositionUtf8,
}

/// A pair of `(line, col)` for UTF-16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionUtf16 {
  /// Zero-based.
  pub line: u32,
  /// Zero-based.
  pub col: u32,
}

impl fmt::Display for PositionUtf16 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}", self.line + 1, self.col + 1)
  }
}

/// A pair of start and end positions for UTF-16.
///
/// `start` comes before `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeUtf16 {
  /// The start.
  pub start: PositionUtf16,
  /// The end.
  pub end: PositionUtf16,
}

impl fmt::Display for RangeUtf16 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}-{}", self.start, self.end)
  }
}
