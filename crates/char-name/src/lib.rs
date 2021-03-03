//! Names for punctuation characters.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

/// Returns the name of the given character.
pub fn get(c: char) -> Option<&'static str> {
  let ret = match c {
    '-' => "Minus",
    ',' => "Comma",
    ';' => "Semicolon",
    ':' => "Colon",
    '!' => "Bang",
    '?' => "Question",
    '.' => "Dot",
    '(' => "LRound",
    ')' => "RRound",
    '[' => "LSquare",
    ']' => "RSquare",
    '{' => "LCurly",
    '}' => "RCurly",
    '*' => "Star",
    '/' => "Slash",
    '&' => "And",
    '%' => "Percent",
    '^' => "Carat",
    '+' => "Plus",
    '<' => "Lt",
    '=' => "Eq",
    '>' => "Gt",
    '|' => "Bar",
    '~' => "Tilde",
    _ => return None,
  };
  Some(ret)
}
