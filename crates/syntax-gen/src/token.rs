use rustc_hash::FxHashMap;
use ungrammar::{Grammar, Token};

#[derive(Debug)]
pub(crate) struct TokenDb {
  pub(crate) punctuation: FxHashMap<Token, String>,
  pub(crate) keywords: FxHashMap<Token, String>,
  pub(crate) special: FxHashMap<Token, (String, &'static str)>,
}

/// A token kind.
#[derive(Debug)]
pub enum TokenKind {
  /// Punctuation, like `{` or `}` or `++`
  Punctuation,
  /// Keywords, i.e. they might be confused as identifiers.
  Keyword,
  /// Special tokens, with a given description.
  Special(&'static str),
}

impl TokenDb {
  pub(crate) fn new<F>(grammar: &Grammar, get_token: F) -> Self
  where
    F: Fn(&str) -> (TokenKind, String),
  {
    let mut punctuation = FxHashMap::default();
    let mut keywords = FxHashMap::default();
    let mut special = FxHashMap::default();
    for token in grammar.tokens() {
      let (kind, name) = get_token(grammar[token].name.as_ref());
      match kind {
        TokenKind::Punctuation => {
          assert!(punctuation.insert(token, name).is_none());
        }
        TokenKind::Keyword => {
          assert!(keywords.insert(token, name).is_none());
        }
        TokenKind::Special(desc) => {
          assert!(special.insert(token, (name, desc)).is_none());
        }
      }
    }
    Self {
      punctuation,
      keywords,
      special,
    }
  }

  pub(crate) fn name(&self, token: Token) -> &str {
    if let Some(x) = self.punctuation.get(&token) {
      x.as_ref()
    } else if let Some(x) = self.keywords.get(&token) {
      x.as_ref()
    } else if let Some(&(ref x, _)) = self.special.get(&token) {
      x.as_ref()
    } else {
      panic!("{:?} does not have a name", token)
    }
  }
}
