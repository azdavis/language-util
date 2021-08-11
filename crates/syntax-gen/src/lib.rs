//! Generates opinionated Rust code from an [ungrammar][1].
//!
//! [1]: https://github.com/rust-analyzer/ungrammar

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

mod alt;
mod seq;
mod token;
mod util;

pub use token::TokenKind;

use crate::util::{ident, Cx};
use proc_macro2::Literal;
use quote::quote;
use rustc_hash::FxHashSet;
use std::cmp::Reverse;
use ungrammar::{Grammar, Rule};

/// Generates Rust code from the `grammar` of the `lang` and writes it to
/// `src/kind.rs` and `src/ast.rs`.
///
/// `lang` is the name of the language, `trivia` is a list of all the
/// `SyntaxKind`s which should be made as trivia, and `grammar` is the grammar
/// for the language.
///
/// `get_token` will be called once for each token in `grammar`, and should
/// return a tuple `(kind, name)`, where `kind` is what kind of token this is (a
/// [`TokenKind`]) and `name` is the name of the token, to be used as an enum
/// variant in the generated `SyntaxKind`.
///
/// The generated Rust files will depend on:
///
/// - `rowan` from crates.io
/// - `token` from language-server-util
/// - `ast-ptr` from language-server-util
///
/// The files will be formatted with rustfmt.
///
/// `src/kind.rs` will contain definitions for the language's `SyntaxKind` and
/// associated types, using all the different tokens extracted from `grammar`
/// and processed with `get_token`.
///
/// `src/ast.rs` will contain a strongly-typed API for traversing a syntax tree
/// for `lang`, based on the `grammar`.
///
/// Returns `Err` if the files could not be written. Panics if certain
/// properties about `grammar` do not hold. (Read the source/panic messages to
/// find out what they are.)
pub fn gen<F>(
  lang: &str,
  trivia: &[&str],
  grammar: Grammar,
  get_token: F,
) -> std::io::Result<()>
where
  F: Fn(&str) -> (TokenKind, String),
{
  let lang = ident(lang);
  let tokens = token::TokenDb::new(&grammar, get_token);
  let mut types = Vec::new();
  let trivia: Vec<_> = trivia.iter().map(|&x| ident(x)).collect();
  let mut node_syntax_kinds = Vec::new();
  let mut cx = Cx {
    lang,
    grammar,
    tokens,
    token_alts: FxHashSet::default(),
  };
  let mut token_alts = FxHashSet::default();
  // first process all the alts
  for node in cx.grammar.iter() {
    let data = &cx.grammar[node];
    let rules = match &data.rule {
      Rule::Alt(rules) => rules.as_slice(),
      _ => continue,
    };
    types.push(alt::get(&cx, &mut token_alts, ident(&data.name), rules));
  }
  // it would be nicer if we could just mutate token_alts on the cx but we have
  // an active shared borrow to iterate over the grammar. so we use a kludge.
  cx.token_alts = token_alts;
  // then everything else
  for node in cx.grammar.iter() {
    let data = &cx.grammar[node];
    let rules = match &data.rule {
      Rule::Alt(_) => continue,
      Rule::Seq(rules) => rules.as_slice(),
      rule => std::slice::from_ref(rule),
    };
    let name = ident(&data.name);
    node_syntax_kinds.push(name.clone());
    types.push(seq::get(&cx, name, rules));
  }
  let Cx {
    grammar,
    tokens,
    lang,
    ..
  } = cx;
  let keywords = {
    let mut xs: Vec<_> = tokens
      .keywords
      .into_iter()
      .map(|(tok, s)| (grammar[tok].name.as_str(), ident(&s)))
      .collect();
    xs.sort_unstable_by_key(|&(name, _)| (Reverse(name.len()), name));
    xs
  };
  let keyword_arms = keywords.iter().map(|(name, kind)| {
    let bs = Literal::byte_string(name.as_bytes());
    quote! { #bs => Self::#kind }
  });
  let punctuation = {
    let mut xs: Vec<_> = tokens
      .punctuation
      .into_iter()
      .map(|(tok, s)| (grammar[tok].name.as_str(), ident(&s)))
      .collect();
    xs.sort_unstable_by_key(|&(name, _)| (Reverse(name.len()), name));
    xs
  };
  let punctuation_len = punctuation.len();
  let punctuation_elements = punctuation.iter().map(|(name, kind)| {
    let bs = Literal::byte_string(name.as_bytes());
    quote! { (#bs, Self::#kind) }
  });
  let special = {
    let mut xs: Vec<_> = tokens.special.into_iter().map(|x| x.1).collect();
    xs.sort_unstable();
    xs
  };
  let desc_arms = punctuation
    .iter()
    .chain(keywords.iter())
    .map(|&(name, ref kind)| {
      let name = format!("`{}`", name);
      quote! { Self::#kind => #name }
    })
    .chain(special.iter().map(|&(ref name, desc)| {
      let kind = util::ident(name);
      quote! { Self::#kind => #desc }
    }));
  let self_trivia = trivia.iter().map(|id| {
    quote! { Self::#id }
  });
  // the order is intentional
  let syntax_kinds: Vec<_> = trivia
    .iter()
    .chain(keywords.iter().chain(punctuation.iter()).map(|(_, id)| id))
    .cloned()
    .chain(special.iter().map(|(name, _)| util::ident(name)))
    .chain(node_syntax_kinds)
    .collect();
  let last_syntax_kind = syntax_kinds.last().unwrap();
  let kind = quote! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(u16)]
    pub enum SyntaxKind {
      #(#syntax_kinds ,)*
    }

    impl SyntaxKind {
      pub const PUNCTUATION: [(&'static [u8], Self); #punctuation_len] = [
        #(#punctuation_elements ,)*
      ];

      pub fn keyword(bs: &[u8]) -> Option<Self> {
        let ret = match bs {
          #(#keyword_arms ,)*
          _ => return None,
        };
        Some(ret)
      }

      pub fn token_desc(&self) -> Option<&'static str> {
        let ret = match *self {
          #(#desc_arms ,)*
          _ => return None,
        };
        Some(ret)
      }
    }

    impl token::Triviable for SyntaxKind {
      fn is_trivia(&self) -> bool {
        matches!(*self, #(#self_trivia)|*)
      }
    }

    impl From<SyntaxKind> for rowan::SyntaxKind {
      fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
      }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum #lang {}

    impl rowan::Language for #lang {
      type Kind = SyntaxKind;

      fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::#last_syntax_kind as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
      }

      fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
      }
    }

    pub type SyntaxNode = rowan::SyntaxNode<#lang>;
    pub type SyntaxToken = rowan::SyntaxToken<#lang>;
    pub type SyntaxElement = rowan::SyntaxElement<#lang>;
  };
  let ast = quote! {
    #![doc = "Abstract syntax trees."]

    use crate::kind::{SyntaxKind as SK, SyntaxNode, SyntaxToken, #lang};
    use ast_ptr::HasLanguage;
    use std::convert::{TryFrom, TryInto};

    #[allow(unused)]
    fn tokens<P>(parent: &P, kind: SK) -> impl Iterator<Item = SyntaxToken>
    where
      P: AsRef<SyntaxNode>,
    {
      parent
        .as_ref()
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter(move |tok| tok.kind() == kind)
    }

    #[allow(unused)]
    fn token_children<P, C>(parent: &P) -> impl Iterator<Item = C>
    where
      P: AsRef<SyntaxNode>,
      SyntaxToken: TryInto<C>,
    {
      parent
        .as_ref()
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter_map(|x| x.try_into().ok())
    }

    #[allow(unused)]
    fn node_children<P, C>(parent: &P) -> impl Iterator<Item = C>
    where
      P: AsRef<SyntaxNode>,
      SyntaxNode: TryInto<C>,
    {
      parent.as_ref().children().filter_map(|x| x.try_into().ok())
    }

    #(#types)*
  };
  util::write_rust_file("src/kind.rs", kind.to_string().as_ref())?;
  util::write_rust_file("src/ast.rs", ast.to_string().as_ref())?;
  Ok(())
}
