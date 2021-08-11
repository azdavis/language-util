//! A [`Sink`] for Rowan trees.

use crate::Sink;
use rowan::{
  GreenNodeBuilder, Language, SyntaxKind, SyntaxNode, TextRange, TextSize,
};
use token::Token;

/// The sink, which wraps a Rowan `GreenNodeBuilder`.
#[derive(Debug)]
pub struct RowanSink {
  builder: GreenNodeBuilder<'static>,
  range: Option<TextRange>,
  errors: Vec<Error>,
}

impl RowanSink {
  /// Finish the builder.
  pub fn finish<L>(self) -> (SyntaxNode<L>, Vec<Error>)
  where
    L: Language,
  {
    (SyntaxNode::new_root(self.builder.finish()), self.errors)
  }
}

impl Default for RowanSink {
  fn default() -> Self {
    Self {
      builder: GreenNodeBuilder::default(),
      range: None,
      errors: Vec::new(),
    }
  }
}

impl<K> Sink<K> for RowanSink
where
  K: Into<SyntaxKind>,
{
  fn enter(&mut self, kind: K) {
    self.builder.start_node(kind.into());
  }

  fn token(&mut self, token: Token<'_, K>) {
    self.builder.token(token.kind.into(), token.text);
    let start = self.range.as_ref().map_or(0.into(), |range| range.end());
    let end = start + TextSize::of(token.text);
    self.range = Some(TextRange::new(start, end));
  }

  fn exit(&mut self) {
    self.builder.finish_node();
  }

  fn error(&mut self, message: String) {
    self.errors.push(Error {
      range: self.range.expect("error with no tokens"),
      message,
    });
  }
}

/// A parse error.
#[derive(Debug)]
pub struct Error {
  /// The range of the error.
  pub range: TextRange,
  /// The message.
  pub message: String,
}
