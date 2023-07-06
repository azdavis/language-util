//! Tests.

use crate::{PositionDb, PositionUtf8};
use text_size_util::TextRange;

#[test]
fn text_pos() {
  let text = "hello\nworld";
  let table =
    [(0, 0, 0), (1, 0, 1), (5, 0, 5), (6, 1, 0), (7, 1, 1), (8, 1, 2), (10, 1, 4), (11, 1, 5)];

  let index = PositionDb::new(text);
  for (offset, line, col) in table {
    assert_eq!(index.position_utf8(offset.into()).unwrap(), PositionUtf8 { line, col });
  }

  let text = "\nhello\nworld";
  let table = [(0, 0, 0), (1, 1, 0), (2, 1, 1), (6, 1, 5), (7, 2, 0)];
  let index = PositionDb::new(text);
  for (offset, line, col) in table {
    assert_eq!(index.position_utf8(offset.into()).unwrap(), PositionUtf8 { line, col });
  }
}

#[test]
fn char_len() {
  assert_eq!('メ'.len_utf8(), 3);
  assert_eq!('メ'.len_utf16(), 1);
}

fn r(lo: u32, hi: u32) -> TextRange {
  TextRange::new(lo.into(), hi.into())
}

#[test]
fn split_lines() {
  let text = "a\nbb\nfoo\n";
  let db = PositionDb::new(text);

  let actual = db.lines(r(0, 9)).collect::<Vec<_>>();
  let expected = vec![r(0, 2), r(2, 5), r(5, 9)];
  assert_eq!(actual, expected);

  let text = "";
  let db = PositionDb::new(text);

  let actual = db.lines(r(0, 0)).collect::<Vec<_>>();
  let expected = vec![];
  assert_eq!(actual, expected);

  let text = "\n";
  let db = PositionDb::new(text);

  let actual = db.lines(r(0, 1)).collect::<Vec<_>>();
  let expected = vec![r(0, 1)];
  assert_eq!(actual, expected);
}
