//! Names for punctuation characters.

/// Returns the name of the given character, or None if it is unknown.
#[must_use]
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
    '`' => "Tick",
    '$' => "Dollar",
    '@' => "At",
    _ => return None,
  };
  Some(ret)
}

/// Returns the name of the given character.
///
/// # Panics
///
/// If the name of this character is not known.
#[must_use]
pub fn get(c: char) -> &'static str {
  match get_opt(c) {
    Some(s) => s,
    None => panic!("don't know the name for {c}"),
  }
}
