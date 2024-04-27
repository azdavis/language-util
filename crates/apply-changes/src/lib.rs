//! Apply changes to a string.
//!
//! Adapted from rust-analyzer.

/// A change.
#[derive(Debug)]
pub struct Change {
  /// The range of the change. If none, the whole text is replaced by the given text.
  pub range: Option<text_pos::RangeUtf16>,
  /// The text to be applied at the range.
  pub text: String,
}

/// Do it.
pub fn get(contents: &mut String, mut changes: Vec<Change>) {
  // If at least one of the changes is a full document change, use the last of them as the starting
  // point and ignore all previous changes.
  let changes = match changes.iter().rposition(|change| change.range.is_none()) {
    Some(idx) => {
      *contents = std::mem::take(&mut changes[idx].text);
      &changes[idx + 1..]
    }
    None => &changes[..],
  };
  if changes.is_empty() {
    return;
  }

  let mut pos_db = text_pos::PositionDb::new(contents);

  // The changes we got must be applied sequentially, but can cross lines so we have to keep our
  // line index updated. Some clients (e.g. Code) sort the ranges in reverse. As an optimization, we
  // remember the last valid line in the index and only rebuild it if needed. The VFS will normalize
  // the end of lines to `\n`.
  let mut index_valid = u32::MAX;
  for change in changes {
    // The None case can't happen as we have handled it above already
    let Some(range) = change.range else { continue };
    if index_valid <= range.end.line {
      pos_db = text_pos::PositionDb::new(contents);
    }
    index_valid = range.start.line;
    if let Some(range) = pos_db.text_range_utf16(range) {
      contents.replace_range(std::ops::Range::<usize>::from(range), &change.text);
    }
  }
}
