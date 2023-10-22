//! The bridge between lexing and parsing.

#![deny(clippy::pedantic, missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![no_std]

/// A token, a pair of syntax kind and text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token<'a, K> {
  /// The kind of token.
  pub kind: K,
  /// The text of the token.
  pub text: &'a str,
}

/// Types whose values can report whether they are trivia or not.
pub trait Triviable {
  /// Returns whether this is trivia.
  fn is_trivia(&self) -> bool;
}
