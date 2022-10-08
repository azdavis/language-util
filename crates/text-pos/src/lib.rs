//! Positions in text.

#![deny(missing_debug_implementations, missing_docs, rust_2018_idioms)]

#[cfg(test)]
mod tests;

use std::fmt;
use text_size::{TextRange, TextSize};

#[derive(Debug)]
struct Line {
  end: TextSize,
  /// pairs of (where this char was in the line, the difference between the
  /// number of bytes needed to represent this char in utf8 and utf16)
  non_ascii: Box<[(TextSize, u32)]>,
}

/// A database allowing translations between [`Position`]s and [`TextSize`]s.
#[derive(Debug)]
pub struct PositionDb {
  /// invariant: non-empty
  lines: Box<[Line]>,
}

impl PositionDb {
  /// Returns a `PositionDb` for the input.
  pub fn new(s: &str) -> Self {
    let mut end = TextSize::from(0);
    let mut col = TextSize::from(0);
    let mut lines = Vec::with_capacity(1);
    let mut non_ascii = Vec::new();
    for c in s.chars() {
      if !c.is_ascii() {
        // it should never happen that for a given c, the len_utf16 for c is
        // greater than the len_utf8 for c.
        let diff: u32 = (c.len_utf8() - c.len_utf16()).try_into().unwrap();
        non_ascii.push((col, diff));
      }
      if c == '\n' {
        lines.push(Line {
          end,
          non_ascii: non_ascii.into_boxed_slice(),
        });
        non_ascii = Vec::new();
        col = TextSize::from(0);
      }
      let ts = TextSize::of(c);
      end += ts;
      col += ts;
    }
    lines.push(Line {
      end,
      non_ascii: non_ascii.into_boxed_slice(),
    });
    Self {
      lines: lines.into_boxed_slice(),
    }
  }

  /// Translates a `TextSize` into a `Position`.
  ///
  /// The `TextSize` must be within the bounds of the original input.
  pub fn position(&self, text_size: TextSize) -> Option<Position> {
    let line = self.lines.iter().position(|line| text_size <= line.end)?;
    let text_size = match line.checked_sub(1) {
      None => text_size,
      Some(prev) => text_size - self.start(prev)?,
    };
    let mut character = u32::from(text_size);
    for &(idx, diff) in self.lines.get(line)?.non_ascii.iter() {
      if idx < text_size {
        character -= diff;
      } else {
        break;
      }
    }
    Some(Position {
      line: line.try_into().ok()?,
      character,
    })
  }

  /// Translates a `Position` into a `TextSize`.
  ///
  /// The `Position` must be within the bounds of the original input.
  pub fn text_size(&self, pos: Position) -> Option<TextSize> {
    let line: usize = pos.line.try_into().ok()?;
    let start = line
      .checked_sub(1)
      .and_then(|line| self.start(line))
      .unwrap_or_else(|| TextSize::from(0));
    let mut col = pos.character;
    for &(idx, diff) in self.lines.get(line)?.non_ascii.iter() {
      if u32::from(idx) < col {
        col += diff;
      } else {
        break;
      }
    }
    Some(start + TextSize::from(col))
  }

  /// Translates a `TextRange` into a `Range`.
  ///
  /// The `TextRange` must be within the bounds of the original input.
  pub fn range(&self, text_range: TextRange) -> Option<Range> {
    Some(Range {
      start: self.position(text_range.start())?,
      end: self.position(text_range.end())?,
    })
  }

  /// Translates a `Range` into a `TextRange`.
  ///
  /// The `Range` must be within the bounds of the original input.
  pub fn text_range(&self, range: Range) -> Option<TextRange> {
    Some(TextRange::new(
      self.text_size(range.start)?,
      self.text_size(range.end)?,
    ))
  }

  /// Return the TextSize of the end of the original text.
  pub fn end_text_size(&self) -> TextSize {
    self.lines.last().expect("empty lines").end
  }

  /// Return the Position of the end of the original text.
  pub fn end_position(&self) -> Position {
    self
      .position(self.end_text_size())
      .expect("end_text_size is valid")
  }

  fn start(&self, line: usize) -> Option<TextSize> {
    // 1 for the newline
    Some(self.lines.get(line)?.end + TextSize::from(1))
  }
}

/// A position in text by line and character.
///
/// Suitable for when the text is represented in UTF-16.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
  /// The line, zero-based.
  pub line: u32,
  /// The character, zero-based.
  pub character: u32,
}

impl fmt::Display for Position {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}", self.line + 1, self.character + 1)
  }
}

/// A pair of start and end positions.
///
/// `start` comes before `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range {
  /// The start.
  pub start: Position,
  /// The end.
  pub end: Position,
}

impl fmt::Display for Range {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}-{}", self.start, self.end)
  }
}
