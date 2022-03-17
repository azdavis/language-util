//! A [`Sink`] for Rowan trees.

use crate::{Expected, Sink};
use rowan::{
  GreenNodeBuilder, Language, SyntaxKind, SyntaxNode, TextRange, TextSize,
};
use token::Token;

/// The sink, which wraps a Rowan `GreenNodeBuilder`.
#[derive(Debug)]
pub struct RowanSink<K> {
  builder: GreenNodeBuilder<'static>,
  range: TextRange,
  errors: Vec<Error<K>>,
}

impl<K> RowanSink<K> {
  /// Finish the builder.
  pub fn finish<L>(self) -> (SyntaxNode<L>, Vec<Error<K>>)
  where
    L: Language,
  {
    (SyntaxNode::new_root(self.builder.finish()), self.errors)
  }
}

impl<K> Default for RowanSink<K> {
  fn default() -> Self {
    Self {
      builder: GreenNodeBuilder::default(),
      range: TextRange::empty(0.into()),
      errors: Vec::new(),
    }
  }
}

impl<K> Sink<K> for RowanSink<K>
where
  K: Into<SyntaxKind>,
{
  fn enter(&mut self, kind: K) {
    self.builder.start_node(kind.into());
  }

  fn token(&mut self, token: Token<'_, K>) {
    self.builder.token(token.kind.into(), token.text);
    let start = self.range.end();
    let end = start + TextSize::of(token.text);
    self.range = TextRange::new(start, end);
  }

  fn exit(&mut self) {
    self.builder.finish_node();
  }

  fn error(&mut self, expected: Expected<K>) {
    self.errors.push(Error {
      range: self.range,
      expected,
    });
  }
}

/// A parse error.
#[derive(Debug)]
pub struct Error<K> {
  /// The range of the error.
  pub range: TextRange,
  /// The thing that was expected.
  pub expected: Expected<K>,
}
