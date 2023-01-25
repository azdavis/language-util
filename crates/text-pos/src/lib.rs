//! See [`PositionDb`]. Heavily adapted from rust-analyzer.

#![deny(clippy::pedantic, missing_debug_implementations, missing_docs, rust_2018_idioms)]

#[cfg(test)]
mod tests;

use fast_hash::FxHashMap;
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
          ret.utf16_lines.insert(line, std::mem::take(&mut utf16_chars));
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

  /// Returns the `PositionUtf16` for this `TextSize`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn position_utf16(&self, text_size: TextSize) -> Option<PositionUtf16> {
    self.position_utf8(text_size).map(|x| self.position_to_utf16(x))
  }

  /// Returns the `TextSize` for this `PositionUtf16`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn text_size_utf16(&self, pos: PositionUtf16) -> Option<TextSize> {
    self.text_size_utf8(self.position_to_utf8(pos))
  }

  /// Returns the `RangeUtf16` for this `TextRange`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn range_utf16(&self, text_range: TextRange) -> Option<RangeUtf16> {
    self.range_utf8(text_range).map(|x| self.range_to_utf16(x))
  }

  /// Returns the `TextRange` for this `RangeUtf16`, or `None` if it is out of bounds.
  ///
  /// # Panics
  ///
  /// Upon internal error.
  #[must_use]
  pub fn text_range_utf16(&self, range: RangeUtf16) -> Option<TextRange> {
    self.text_range_utf8(self.range_to_utf8(range))
  }

  fn position_utf8(&self, text_size: TextSize) -> Option<PositionUtf8> {
    let line = self.newlines.partition_point(|&it| it <= text_size).checked_sub(1)?;
    let col = text_size - self.newlines.get(line)?;
    Some(PositionUtf8 { line: line.try_into().unwrap(), col: col.into() })
  }

  fn text_size_utf8(&self, pos: PositionUtf8) -> Option<TextSize> {
    self
      .newlines
      .get(usize::try_from(pos.line).unwrap())
      .map(|text_size| text_size + TextSize::from(pos.col))
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

  fn position_to_utf16(&self, pos: PositionUtf8) -> PositionUtf16 {
    let col = self.col_to_utf16(pos.line, pos.col.into());
    PositionUtf16 { line: pos.line, col: col.try_into().unwrap() }
  }

  fn position_to_utf8(&self, pos: PositionUtf16) -> PositionUtf8 {
    let col = self.col_to_utf8(pos.line, pos.col);
    PositionUtf8 { line: pos.line, col: col.into() }
  }

  fn range_to_utf16(&self, range: RangeUtf8) -> RangeUtf16 {
    RangeUtf16 {
      start: self.position_to_utf16(range.start),
      end: self.position_to_utf16(range.end),
    }
  }

  fn range_to_utf8(&self, range: RangeUtf16) -> RangeUtf8 {
    RangeUtf8 { start: self.position_to_utf8(range.start), end: self.position_to_utf8(range.end) }
  }

  /// Returns an iterator over the lines in the range.
  pub fn lines(&self, range: TextRange) -> impl Iterator<Item = TextRange> + '_ {
    let lo = self.newlines.partition_point(|&it| it < range.start());
    let hi = self.newlines.partition_point(|&it| it <= range.end());
    let all = std::iter::once(range.start())
      .chain(self.newlines[lo..hi].iter().copied())
      .chain(std::iter::once(range.end()));

    all.clone().zip(all.skip(1)).map(|(lo, hi)| TextRange::new(lo, hi)).filter(|it| !it.is_empty())
  }

  fn col_to_utf16(&self, line: u32, col: TextSize) -> usize {
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

  fn col_to_utf8(&self, line: u32, mut col: u32) -> TextSize {
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
  pub fn end_position_utf16(&self) -> PositionUtf16 {
    let pos = self.position_utf8(self.len()).expect("len is in range");
    self.position_to_utf16(pos)
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
