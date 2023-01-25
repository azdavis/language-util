//! Tests.

use crate::{CharUtf16, PositionDb, PositionUtf8};
use text_size_util::{TextRange, TextSize};

#[test]
fn text_pos() {
  let text = "hello\nworld";
  let table = [
    (0, 0, 0),
    (1, 0, 1),
    (5, 0, 5),
    (6, 1, 0),
    (7, 1, 1),
    (8, 1, 2),
    (10, 1, 4),
    (11, 1, 5),
    (12, 1, 6),
  ];

  let index = PositionDb::new(text);
  for (offset, line, col) in table {
    assert_eq!(index.position(offset.into()).unwrap(), PositionUtf8 { line, col });
  }

  let text = "\nhello\nworld";
  let table = [(0, 0, 0), (1, 1, 0), (2, 1, 1), (6, 1, 5), (7, 2, 0)];
  let index = PositionDb::new(text);
  for (offset, line, col) in table {
    assert_eq!(index.position(offset.into()).unwrap(), PositionUtf8 { line, col });
  }
}

#[test]
fn char_len() {
  assert_eq!('メ'.len_utf8(), 3);
  assert_eq!('メ'.len_utf16(), 1);
}

#[test]
fn empty() {
  let col_index = PositionDb::new(
    "
const C: char = 'x';
",
  );
  assert_eq!(col_index.utf16_lines.len(), 0);
}

#[test]
fn single_char() {
  let col_index = PositionDb::new(
    "
const C: char = 'メ';
",
  );

  assert_eq!(col_index.utf16_lines.len(), 1);
  assert_eq!(col_index.utf16_lines[&1].len(), 1);
  assert_eq!(col_index.utf16_lines[&1][0], CharUtf16 { start: 17.into(), end: 20.into() });

  // UTF-8 to UTF-16, no changes
  assert_eq!(col_index.to_utf16_col(1, 15.into()), 15);

  // UTF-8 to UTF-16
  assert_eq!(col_index.to_utf16_col(1, 22.into()), 20);

  // UTF-16 to UTF-8, no changes
  assert_eq!(col_index.to_utf8_col(1, 15), TextSize::from(15));

  // UTF-16 to UTF-8
  assert_eq!(col_index.to_utf8_col(1, 19), TextSize::from(21));

  let col_index = PositionDb::new("a𐐏b");
  assert_eq!(col_index.to_utf8_col(0, 3), TextSize::from(5));
}

#[test]
fn string() {
  let col_index = PositionDb::new(
    "
const C: char = \"メ メ\";
",
  );

  assert_eq!(col_index.utf16_lines.len(), 1);
  assert_eq!(col_index.utf16_lines[&1].len(), 2);
  assert_eq!(col_index.utf16_lines[&1][0], CharUtf16 { start: 17.into(), end: 20.into() });
  assert_eq!(col_index.utf16_lines[&1][1], CharUtf16 { start: 21.into(), end: 24.into() });

  // UTF-8 to UTF-16
  assert_eq!(col_index.to_utf16_col(1, 15.into()), 15);

  assert_eq!(col_index.to_utf16_col(1, 21.into()), 19);
  assert_eq!(col_index.to_utf16_col(1, 25.into()), 21);

  assert!(col_index.to_utf16_col(2, 15.into()) == 15);

  // UTF-16 to UTF-8
  assert_eq!(col_index.to_utf8_col(1, 15), TextSize::from(15));

  // メ UTF-8: 0xE3 0x83 0xA1, UTF-16: 0x30E1
  assert_eq!(col_index.to_utf8_col(1, 17), TextSize::from(17)); // first メ at 17..20
  assert_eq!(col_index.to_utf8_col(1, 18), TextSize::from(20)); // space
  assert_eq!(col_index.to_utf8_col(1, 19), TextSize::from(21)); // second メ at 21..24

  assert_eq!(col_index.to_utf8_col(2, 15), TextSize::from(15));
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
