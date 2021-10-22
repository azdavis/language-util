//! AST pointers using Rowan.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

mod raw;

use raw::SyntaxNodePtr;
use rowan::{Language, SyntaxNode};
use std::fmt;
use std::hash::{Hash, Hasher};

/// Types which have a Rowan language.
pub trait HasLanguage {
  /// The language.
  type Language: Language;
}

/// A 'pointer' to an AST node. Stable across re-parses of identical files.
pub struct AstPtr<N>
where
  N: HasLanguage,
{
  raw: SyntaxNodePtr<N::Language>,
}

impl<N> AstPtr<N>
where
  N: HasLanguage + AsRef<SyntaxNode<N::Language>>,
{
  /// Returns a new `AstPtr` for the given node.
  pub fn new(node: &N) -> Self {
    Self {
      raw: SyntaxNodePtr::new(node.as_ref()),
    }
  }
}

impl<N> AstPtr<N>
where
  N: HasLanguage,
  <N::Language as Language>::Kind: Eq,
  SyntaxNode<N::Language>: TryInto<N>,
  <SyntaxNode<N::Language> as TryInto<N>>::Error: fmt::Debug,
{
  /// Given the root node (i.e. it has no parent) that contains the
  /// original node that the `AstPtr` was constructed from, return that
  /// original node.
  pub fn to_node(&self, root: SyntaxNode<N::Language>) -> N {
    self.raw.to_node(root).try_into().unwrap()
  }
}

impl<N> fmt::Debug for AstPtr<N>
where
  N: HasLanguage,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("AstPtr").field("raw", &self.raw).finish()
  }
}

impl<N> Clone for AstPtr<N>
where
  N: HasLanguage,
  <N::Language as Language>::Kind: Copy,
{
  fn clone(&self) -> Self {
    *self
  }
}

impl<N> Copy for AstPtr<N>
where
  N: HasLanguage,
  <N::Language as Language>::Kind: Copy,
{
}

impl<N> PartialEq for AstPtr<N>
where
  N: HasLanguage,
  <N::Language as Language>::Kind: PartialEq<<N::Language as Language>::Kind>,
{
  fn eq(&self, other: &AstPtr<N>) -> bool {
    self.raw == other.raw
  }
}

impl<N> Eq for AstPtr<N>
where
  N: HasLanguage,
  <N::Language as Language>::Kind: Eq,
{
}

impl<N> Hash for AstPtr<N>
where
  N: HasLanguage,
  <N::Language as Language>::Kind: Hash,
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.raw.hash(state)
  }
}
