use rowan::{Language, SyntaxNode, TextRange};
use std::fmt;
use std::hash::{Hash, Hasher};

pub(crate) struct SyntaxNodePtr<L>
where
  L: Language,
{
  kind: L::Kind,
  range: TextRange,
}

impl<L> SyntaxNodePtr<L>
where
  L: rowan::Language,
{
  pub(crate) fn new(node: &SyntaxNode<L>) -> Self {
    Self {
      kind: node.kind(),
      range: node.text_range(),
    }
  }
}

impl<L> SyntaxNodePtr<L>
where
  L: rowan::Language,
  L::Kind: Eq,
{
  pub(crate) fn to_node(&self, mut root: SyntaxNode<L>) -> SyntaxNode<L> {
    assert!(root.parent().is_none());
    loop {
      if root.text_range() == self.range && root.kind() == self.kind {
        return root;
      }
      root = root
        .children()
        .find(|x| x.text_range().contains_range(self.range))
        .unwrap();
    }
  }
}

impl<L> fmt::Debug for SyntaxNodePtr<L>
where
  L: Language,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("SyntaxNodePtr")
      .field("kind", &self.kind)
      .field("range", &self.range)
      .finish()
  }
}

impl<L> Clone for SyntaxNodePtr<L>
where
  L: Language,
  L::Kind: Copy,
{
  fn clone(&self) -> Self {
    *self
  }
}

impl<L> Copy for SyntaxNodePtr<L>
where
  L: Language,
  L::Kind: Copy,
{
}

impl<L> PartialEq for SyntaxNodePtr<L>
where
  L: Language,
  L::Kind: PartialEq<L::Kind>,
{
  fn eq(&self, other: &SyntaxNodePtr<L>) -> bool {
    self.kind == other.kind && self.range == other.range
  }
}

impl<L> Eq for SyntaxNodePtr<L>
where
  L: Language,
  L::Kind: Eq,
{
}

impl<L> Hash for SyntaxNodePtr<L>
where
  L: Language,
  L::Kind: Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.kind.hash(state);
    self.range.hash(state);
  }
}
