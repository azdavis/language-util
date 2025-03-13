//! A [`Sink`] for Rowan trees.

use crate::Sink;
use rowan::{GreenNodeBuilder, Language, SyntaxKind, SyntaxNode, TextRange, TextSize};
use token::{Token, Triviable};

/// The sink, which wraps a Rowan `GreenNodeBuilder`.
#[derive(Debug)]
pub struct RowanSink<K, E> {
  builder: GreenNodeBuilder<'static>,
  cur: (TextRange, Option<K>),
  errors: Vec<Error<K, E>>,
  no_range: Vec<E>,
}

impl<K, E> RowanSink<K, E>
where
  K: Clone,
{
  /// Finish the builder.
  #[must_use]
  pub fn finish<L>(mut self) -> (SyntaxNode<L>, Vec<Error<K, E>>)
  where
    L: Language,
  {
    self.extend_errors();
    let root = SyntaxNode::new_root(self.builder.finish());
    (root, self.errors)
  }

  fn extend_errors(&mut self) {
    let errors = std::mem::take(&mut self.no_range).into_iter().map(|inner| Error {
      range: self.cur.0,
      kind: self.cur.1.clone(),
      inner,
    });
    self.errors.extend(errors);
  }
}

impl<K, E> Default for RowanSink<K, E> {
  fn default() -> Self {
    Self {
      builder: GreenNodeBuilder::default(),
      cur: (TextRange::empty(0.into()), None),
      errors: Vec::new(),
      no_range: Vec::new(),
    }
  }
}

impl<K, E> Sink<K, E> for RowanSink<K, E>
where
  K: Into<SyntaxKind> + Triviable + Clone,
{
  fn enter(&mut self, kind: K) {
    self.builder.start_node(kind.into());
  }

  fn token(&mut self, token: Token<'_, K>) {
    let kind = token.kind.clone();
    let is_trivia = token.kind.is_trivia();
    self.builder.token(token.kind.into(), token.text);
    let start = self.cur.0.end();
    let end = start + TextSize::of(token.text);
    self.cur = (TextRange::new(start, end), Some(kind));
    if !is_trivia {
      self.extend_errors();
    }
  }

  fn exit(&mut self) {
    self.builder.finish_node();
  }

  fn error(&mut self, error: E) {
    self.no_range.push(error);
  }
}

/// An error.
#[derive(Debug, Clone)]
pub struct Error<K, E> {
  /// The range.
  pub range: TextRange,
  /// The syntax kind.
  pub kind: Option<K>,
  /// The inner error.
  pub inner: E,
}
