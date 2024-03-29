use fast_hash::FxHashMap;
use std::{collections::HashMap, hash::BuildHasher};

#[derive(Debug)]
pub(crate) struct TokenDb {
  pub(crate) punctuation: FxHashMap<ungrammar::Token, Token>,
  pub(crate) keywords: FxHashMap<ungrammar::Token, Token>,
  pub(crate) special: FxHashMap<ungrammar::Token, Token>,
}

/// A token kind.
#[derive(Debug)]
pub enum Kind {
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
  pub(crate) fn new<S>(
    grammar: &ungrammar::Grammar,
    doc: &HashMap<&str, &str, S>,
    special_desc: &HashMap<&str, &str, S>,
  ) -> Self
  where
    S: BuildHasher,
  {
    let mut punctuation = FxHashMap::default();
    let mut keywords = FxHashMap::default();
    let mut special = FxHashMap::default();
    for token in grammar.tokens() {
      let orig_name = grammar[token].name.as_str();
      let kind: Kind;
      let mut name: String;
      let mut desc = None::<String>;
      if let Some(&d) = special_desc.get(orig_name) {
        kind = Kind::Special;
        name = orig_name.to_owned();
        desc = Some(d.to_owned());
      } else if orig_name.chars().any(|c| c.is_ascii_alphabetic()) {
        kind = Kind::Keyword;
        name = identifier_case::snake_to_pascal(orig_name);
        name.push_str("Kw");
      } else {
        kind = Kind::Punctuation;
        name = String::new();
        for c in orig_name.chars() {
          name.push_str(char_name::get(c));
        }
      }
      let tok = Token { name, desc, doc: doc.get(orig_name).map(|&x| x.to_owned()) };
      match kind {
        Kind::Punctuation => {
          assert!(punctuation.insert(token, tok).is_none());
        }
        Kind::Keyword => {
          assert!(keywords.insert(token, tok).is_none());
        }
        Kind::Special => {
          assert!(special.insert(token, tok).is_none());
        }
      }
    }
    Self { punctuation, keywords, special }
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
