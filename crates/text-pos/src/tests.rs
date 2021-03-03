use crate::{Position, PositionDb, TextSize};

fn check(s: &str, tests: &[(u32, u32, u32)]) {
  let lines = PositionDb::new(s);
  for &(idx, line, character) in tests {
    let text_size = TextSize::from(idx);
    let pos = Position { line, character };
    assert_eq!(lines.position(text_size), pos);
    assert_eq!(lines.text_size(pos), text_size);
  }
}

#[test]
fn simple() {
  check(
    "hello\nnew\nworld\n",
    &[
      (0, 0, 0),
      (1, 0, 1),
      (4, 0, 4),
      (5, 0, 5),
      (6, 1, 0),
      (9, 1, 3),
      (10, 2, 0),
      (11, 2, 1),
      (15, 2, 5),
      (16, 3, 0),
    ],
  );
}

#[test]
fn leading_newline() {
  check(
    "\n\nhey\n\nthere",
    &[
      (0, 0, 0),
      (1, 1, 0),
      (2, 2, 0),
      (3, 2, 1),
      (5, 2, 3),
      (6, 3, 0),
      (7, 4, 0),
      (8, 4, 1),
      (12, 4, 5),
    ],
  );
}

#[test]
fn lsp_spec_example() {
  check(
    "a𐐀b",
    &[
      (0, 0, 0),
      (1, 0, 1),
      // 2, 3, 4 impossible because 𐐀 is 4 bytes long in UTF-8
      (5, 0, 3),
      (6, 0, 4),
    ],
  );
}
