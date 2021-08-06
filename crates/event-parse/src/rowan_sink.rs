//! A `Sink` for Rowan trees.

use rowan::{GreenNode, GreenNodeBuilder, SyntaxKind, TextRange, TextSize};
use token::Token;

/// The sink, which wraps a Rowan `GreenNodeBuilder`.
#[derive(Debug, Default)]
pub struct RowanSink<T> {
  builder: GreenNodeBuilder<'static>,
  range: Option<TextRange>,
  errors: Vec<Error<T>>,
}

impl<T> RowanSink<T> {
  /// Finish the builder.
  pub fn finish(self) -> (GreenNode, Vec<Error<T>>) {
    (self.builder.finish(), self.errors)
  }
}

impl<T> crate::Sink<T> for RowanSink<T>
where
  T: Into<SyntaxKind>,
{
  fn enter(&mut self, kind: T) {
    self.builder.start_node(kind.into());
  }

  fn token(&mut self, token: Token<'_, T>) {
    self.builder.token(token.kind.into(), token.text);
    let start = self.range.as_ref().map_or(0.into(), |range| range.end());
    let end = start + TextSize::of(token.text);
    self.range = Some(TextRange::new(start, end));
  }

  fn exit(&mut self) {
    self.builder.finish_node();
  }

  fn error(&mut self, expected: Vec<T>) {
    self.errors.push(Error {
      range: self.range.expect("error with no tokens"),
      expected,
    });
  }
}

/// A parse error.
#[derive(Debug)]
pub struct Error<T> {
  /// The range of the unexpected token.
  pub range: TextRange,
  /// The tokens that would have been allowed.
  pub expected: Vec<T>,
}