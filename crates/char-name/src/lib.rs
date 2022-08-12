//! Names for punctuation characters.

/// Returns the name of the given character, or None if it is unknown.
pub fn get_opt(c: char) -> Option<&'static str> {
  let ret = match c {
    '_' => "Underscore",
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
    '#' => "Hash",
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

/// Returns the name of the given character, or panics if it is unknown.
pub fn get(c: char) -> &'static str {
  match get_opt(c) {
    Some(s) => s,
    None => panic!("don't know the name for {c}"),
  }
}
