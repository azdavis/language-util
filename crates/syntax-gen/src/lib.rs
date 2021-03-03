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
mod write;

pub use token::TokenKind;

use crate::util::{ident, Cx};
use proc_macro2::Literal;
use quote::quote;
use std::cmp::Reverse;
use ungrammar::{Grammar, Rule};

enum Kind {
  Seq,
  Alt,
}

/// Generates Rust code from the `grammar` of the `lang` and writes it to
/// `src/kind.rs` and `src/ast.rs`.
///
/// `get_token` will be called for each token in `grammar`, and should return
/// `(kind, name`), where `kind` is what kind of token this is (a [`TokenKind`])
/// and `name` is the name of the token, to be used as an enum variant in the
/// generated `SyntaxKind`.
///
/// The generated Rust files will depend on `rowan` and `event-parse`. The files
/// will be formatted with rustfmt.
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
pub fn gen<F>(lang: &str, grammar: Grammar, get_token: F) -> std::io::Result<()>
where
  F: Fn(&str) -> (TokenKind, String),
{
  let lang = ident(lang);
  let tokens = token::TokenDb::new(&grammar, get_token);
  let cx = Cx { grammar, tokens };
  let mut types = Vec::new();
  let mut syntax_kinds = Vec::new();
  for node in cx.grammar.iter() {
    let data = &cx.grammar[node];
    let name = ident(&data.name);
    let (kind, rules) = match &data.rule {
      Rule::Seq(rules) => (Kind::Seq, rules.as_slice()),
      Rule::Alt(rules) => (Kind::Alt, rules.as_slice()),
      rule => (Kind::Seq, std::slice::from_ref(rule)),
    };
    let ty = match kind {
      Kind::Seq => {
        syntax_kinds.push(name.clone());
        seq::get(&cx, name, rules)
      }
      Kind::Alt => alt::get(&cx, name, rules),
    };
    types.push(ty);
  }
  let Cx { grammar, tokens } = cx;
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
  let new_syntax_kinds = keywords
    .iter()
    .cloned()
    .chain(punctuation.iter().cloned())
    .map(|x| x.1)
    .chain(special.iter().map(|&(ref name, _)| util::ident(name)));
  syntax_kinds.extend(new_syntax_kinds);
  let last_syntax_kind = syntax_kinds.last().unwrap();
  let kind = quote! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(u16)]
    pub enum SyntaxKind {
      Whitespace,
      LineComment,
      BlockComment,
      Invalid,
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

    impl event_parse::Triviable for SyntaxKind {
      fn is_trivia(&self) -> bool {
        matches!(
          *self,
          Self::Whitespace
          | Self::LineComment
          | Self::BlockComment
          | Self::Invalid
        )
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
    #![allow(clippy::iter_nth_zero)]

    use crate::kind::{SyntaxElement, SyntaxKind as SK, SyntaxNode, SyntaxToken};

    pub trait Cast: Sized {
      fn cast(elem: SyntaxElement) -> Option<Self>;
    }

    pub trait Syntax {
      fn syntax(&self) -> &SyntaxNode;
    }

    fn token<P>(parent: &P, kind: SK, idx: usize) -> Option<SyntaxToken>
    where
      P: Syntax,
    {
      parent
        .syntax()
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter(move |tok| tok.kind() == kind)
        .nth(idx)
    }

    fn children<P, C>(parent: &P) -> impl Iterator<Item = C>
    where
      P: Syntax,
      C: Cast,
    {
      parent.syntax().children_with_tokens().filter_map(C::cast)
    }

    #(#types)*
  };
  write::write_rust_file("src/kind.rs", kind.to_string().as_ref())?;
  write::write_rust_file("src/ast.rs", ast.to_string().as_ref())?;
  Ok(())
}
