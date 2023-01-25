//! See [`PositionDb`]. Heavily adapted from rust-analyzer.

#![deny(clippy::pedantic, missing_debug_implementations, missing_docs, rust_2018_idioms)]

#[cfg(test)]
mod tests;

use fast_hash::FxHashMap;
use std::{iter, mem};
use text_size_util::{TextRange, TextSize};

/// Converts between flat [`TextSize`] offsets and `(line, col)` representation, with handling for
/// both UTF-8 and UTF-16 encoded text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionDb {
  /// Offset the the beginning of each line, zero-based. Non-empty.
  newlines: Vec<TextSize>,
  /// List of non-ASCII characters on each line.
  utf16_lines: FxHashMap<u32, Vec<CharUtf16>>,
  len: TextSize,
}

impl PositionDb {
  /// Returns a new `PositionDb` for the text.
  #[must_use]
  pub fn new(text: &str) -> PositionDb {
    let mut ret = PositionDb {
      newlines: Vec::with_capacity(16),
      utf16_lines: FxHashMap::default(),
      len: TextSize::of(text),
    };
    ret.newlines.push(TextSize::from(0));
    let mut utf16_chars = Vec::<CharUtf16>::new();
    let mut row = TextSize::from(0);
    let mut col = TextSize::from(0);
    let mut line = 0u32;
    for c in text.chars() {
      let c_len = TextSize::of(c);
      row += c_len;
      if c == '\n' {
        ret.newlines.push(row);
        // Save any UTF-16 characters seen in the previous line
        if !utf16_chars.is_empty() {
          ret.utf16_lines.insert(line, mem::take(&mut utf16_chars));
        }
        // Prepare for processing the next line
        col = TextSize::from(0);
        line += 1;
        continue;
      }
      if !c.is_ascii() {
        utf16_chars.push(CharUtf16 { start: col, end: col + c_len });
      }
      col += c_len;
    }
    // Save any UTF-16 characters seen in the last line
    if !utf16_chars.is_empty() {
      ret.utf16_lines.insert(line, utf16_chars);
    }
    ret
  }

  /// Returns the `Position` for this `TextSize`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn position(&self, text_size: TextSize) -> Option<PositionUtf8> {
    let line = self.newlines.partition_point(|&it| it <= text_size).checked_sub(1)?;
    let col = text_size - self.newlines.get(line)?;
    Some(PositionUtf8 { line: line.try_into().unwrap(), col: col.into() })
  }

  /// Returns the `TextSize` for this `Position`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn text_size(&self, pos: PositionUtf8) -> Option<TextSize> {
    self
      .newlines
      .get(usize::try_from(pos.line).unwrap())
      .map(|text_size| text_size + TextSize::from(pos.col))
  }

  /// Returns the `Range` for this `TextRange`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn range(&self, text_range: TextRange) -> Option<RangeUtf8> {
    Some(RangeUtf8 {
      start: self.position(text_range.start())?,
      end: self.position(text_range.end())?,
    })
  }

  /// Returns the `TextRange` for this `Range`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn text_range(&self, range: RangeUtf8) -> Option<TextRange> {
    Some(TextRange::new(self.text_size(range.start)?, self.text_size(range.end)?))
  }

  /// Converts a `Position` (which is always for UTF-8) to a `PositionUtf16`.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn to_position_utf16(&self, pos: PositionUtf8) -> PositionUtf16 {
    let col = self.to_utf16_col(pos.line, pos.col.into());
    PositionUtf16 { line: pos.line, col: col.try_into().unwrap() }
  }

  /// Converts a `PositionUtf16` to a `Position` (which is always for UTF-8).
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn to_position_utf8(&self, pos: PositionUtf16) -> PositionUtf8 {
    let col = self.to_utf8_col(pos.line, pos.col);
    PositionUtf8 { line: pos.line, col: col.into() }
  }

  /// Converts a `Range` (which is always for UTF-8) to a `RangeUtf16`.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn to_range_utf16(&self, range: RangeUtf8) -> RangeUtf16 {
    RangeUtf16 {
      start: self.to_position_utf16(range.start),
      end: self.to_position_utf16(range.end),
    }
  }

  /// Converts a `RangeUtf16` to a `Range` (which is always for UTF-8).
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn to_range_utf8(&self, range: RangeUtf16) -> RangeUtf8 {
    RangeUtf8 { start: self.to_position_utf8(range.start), end: self.to_position_utf8(range.end) }
  }

  /// Returns an iterator over the lines in the range.
  pub fn lines(&self, range: TextRange) -> impl Iterator<Item = TextRange> + '_ {
    let lo = self.newlines.partition_point(|&it| it < range.start());
    let hi = self.newlines.partition_point(|&it| it <= range.end());
    let all = iter::once(range.start())
      .chain(self.newlines[lo..hi].iter().copied())
      .chain(iter::once(range.end()));

    all.clone().zip(all.skip(1)).map(|(lo, hi)| TextRange::new(lo, hi)).filter(|it| !it.is_empty())
  }

  fn to_utf16_col(&self, line: u32, col: TextSize) -> usize {
    let mut res: usize = col.into();
    if let Some(utf16_chars) = self.utf16_lines.get(&line) {
      for c in utf16_chars {
        if c.end <= col {
          res -= usize::from(c.len()) - c.len_utf16();
        } else {
          // From here on, all utf16 characters come *after* the character we are mapping, so we
          // don't need to take them into account
          break;
        }
      }
    }
    res
  }

  fn to_utf8_col(&self, line: u32, mut col: u32) -> TextSize {
    if let Some(utf16_chars) = self.utf16_lines.get(&line) {
      for c in utf16_chars {
        if col > u32::from(c.start) {
          col += u32::from(c.len()) - u32::try_from(c.len_utf16()).unwrap();
        } else {
          // From here on, all utf16 characters come *after* the character we are mapping, so we
          // don't need to take them into account
          break;
        }
      }
    }
    col.into()
  }

  /// Returns the length of the original text.
  #[must_use]
  pub fn len(&self) -> TextSize {
    self.len
  }

  /// Returns the end position of the original input.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn end_position(&self) -> PositionUtf8 {
    self.position(self.len()).expect("len is in range")
  }
}

/// A pair of `(line, col)` for UTF-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionUtf8 {
  /// Zero-based.
  pub line: u32,
  /// Zero-based utf8 offset.
  pub col: u32,
}

/// A pair of start and end positions for UTF-8.
///
/// `start` comes before `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeUtf8 {
  /// The start.
  pub start: PositionUtf8,
  /// The end.
  pub end: PositionUtf8,
}

/// A pair of `(line, col)` for UTF-16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionUtf16 {
  /// Zero-based.
  pub line: u32,
  /// Zero-based.
  pub col: u32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CharUtf16 {
  /// Start offset of a character inside a line, zero-based.
  start: TextSize,
  /// End offset of a character inside a line, zero-based.
  end: TextSize,
}

impl CharUtf16 {
  /// Returns the length in 8-bit UTF-8 code units.
  fn len(&self) -> TextSize {
    self.end - self.start
  }

  /// Returns the length in 16-bit UTF-16 code units.
  fn len_utf16(&self) -> usize {
    if self.len() == TextSize::from(4) {
      2
    } else {
      1
    }
  }
}
