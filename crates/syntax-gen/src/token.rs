use rustc_hash::FxHashMap;

#[derive(Debug)]
pub(crate) struct TokenDb {
  pub(crate) punctuation: FxHashMap<ungrammar::Token, Token>,
  pub(crate) keywords: FxHashMap<ungrammar::Token, Token>,
  pub(crate) special: FxHashMap<ungrammar::Token, Token>,
}

/// A token kind.
#[derive(Debug)]
pub enum TokenKind {
  /// Punctuation, like `{` or `}` or `++`
  Punctuation,
  /// Keywords, i.e. they might be confused as identifiers.
  Keyword,
  /// Special tokens
  Special,
}

/// A token.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
  /// The name of the token.
  pub name: String,
  /// Optional short description.
  pub desc: Option<String>,
  /// Optional long documentation.
  pub doc: Option<String>,
}

impl Token {
  pub(crate) fn name_ident(&self) -> proc_macro2::Ident {
    ident(self.name.as_str())
  }
}

impl TokenDb {
  pub(crate) fn new<F>(grammar: &ungrammar::Grammar, get_token: F) -> Self
  where
    F: Fn(&str) -> (TokenKind, Token),
  {
    let mut punctuation = FxHashMap::default();
    let mut keywords = FxHashMap::default();
    let mut special = FxHashMap::default();
    for token in grammar.tokens() {
      let (kind, tok) = get_token(grammar[token].name.as_ref());
      match kind {
        TokenKind::Punctuation => {
          assert!(punctuation.insert(token, tok).is_none())
        }
        TokenKind::Keyword => {
          assert!(keywords.insert(token, tok).is_none())
        }
        TokenKind::Special => {
          assert!(special.insert(token, tok).is_none())
        }
      }
    }
    Self {
      punctuation,
      keywords,
      special,
    }
  }

  pub(crate) fn get(&self, token: ungrammar::Token) -> &Token {
    None
      .or_else(|| self.punctuation.get(&token))
      .or_else(|| self.keywords.get(&token))
      .or_else(|| self.special.get(&token))
      .unwrap_or_else(|| panic!("{token:?} does not have a name"))
  }
}

pub(crate) fn ident(s: &str) -> proc_macro2::Ident {
  quote::format_ident!("{}", s)
}
