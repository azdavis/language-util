//! A [`Sink`] for Rowan trees.

use crate::Sink;
use rowan::{
  GreenNodeBuilder, Language, SyntaxKind, SyntaxNode, TextRange, TextSize,
};
use token::{Token, Triviable};

/// The sink, which wraps a Rowan `GreenNodeBuilder`.
#[derive(Debug)]
pub struct RowanSink<E> {
  builder: GreenNodeBuilder<'static>,
  range: TextRange,
  errors: Vec<Error<E>>,
  no_range: Vec<E>,
}

impl<E> RowanSink<E> {
  /// Finish the builder.
  pub fn finish<L>(mut self) -> (SyntaxNode<L>, Vec<Error<E>>)
  where
    L: Language,
  {
    self.extend_errors();
    let root = SyntaxNode::new_root(self.builder.finish());
    (root, self.errors)
  }

  fn extend_errors(&mut self) {
    let errors =
      std::mem::take(&mut self.no_range)
        .into_iter()
        .map(|kind| Error {
          range: self.range,
          kind,
        });
    self.errors.extend(errors);
  }
}

impl<E> Default for RowanSink<E> {
  fn default() -> Self {
    Self {
      builder: GreenNodeBuilder::default(),
      range: TextRange::empty(0.into()),
      errors: Vec::new(),
      no_range: Vec::new(),
    }
  }
}

impl<K, E> Sink<K, E> for RowanSink<E>
where
  K: Into<SyntaxKind> + Triviable,
{
  fn enter(&mut self, kind: K) {
    self.builder.start_node(kind.into());
  }

  fn token(&mut self, token: Token<'_, K>) {
    let is_trivia = token.kind.is_trivia();
    self.builder.token(token.kind.into(), token.text);
    let start = self.range.end();
    let end = start + TextSize::of(token.text);
    self.range = TextRange::new(start, end);
    if !is_trivia {
      self.extend_errors();
    }
  }

  fn exit(&mut self) {
    self.builder.finish_node();
  }

  fn error(&mut self, error: E) {
    self.no_range.push(error)
  }
}

/// An error.
#[derive(Debug, Clone)]
pub struct Error<E> {
  /// The range.
  pub range: TextRange,
  /// The kind.
  pub kind: E,
}
