//! Generates opinionated Rust code from an [ungrammar][1].
//!
//! [1]: https://github.com/rust-analyzer/ungrammar

#![deny(missing_debug_implementations, missing_docs, rust_2018_idioms)]

mod alt;
mod seq;
mod token;
mod util;

use crate::util::Cx;
use proc_macro2::Literal;
use quote::quote;
use rustc_hash::FxHashSet;
use std::cmp::Reverse;
use token::ident;
use ungrammar::{Grammar, Rule};

pub use token::{Token, TokenKind};

/// Generates Rust code from the `grammar` of the `lang` and writes it to
/// `OUT_DIR/kind.rs` and `OUT_DIR/ast.rs`.
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
/// - `token` from language-util
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
  out_dir: &std::path::Path,
  lang: &str,
  trivia: &[&str],
  grammar: Grammar,
  get_token: F,
) -> std::io::Result<()>
where
  F: Fn(&str) -> (TokenKind, Token),
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
      .map(|(ug_tok, tok)| (grammar[ug_tok].name.as_str(), tok))
      .collect();
    xs.sort_unstable_by_key(|&(name, _)| (Reverse(name.len()), name));
    xs
  };
  let keyword_arms = keywords.iter().map(|(name, tok)| {
    let bs = Literal::byte_string(name.as_bytes());
    let kind = tok.name_ident();
    quote! { #bs => Self::#kind }
  });
  let punctuation = {
    let mut xs: Vec<_> = tokens
      .punctuation
      .into_iter()
      .map(|(ug_tok, tok)| (grammar[ug_tok].name.as_str(), tok))
      .collect();
    xs.sort_unstable_by_key(|&(name, _)| (Reverse(name.len()), name));
    xs
  };
  let punctuation_len = punctuation.len();
  let punctuation_elements = punctuation.iter().map(|(name, tok)| {
    let bs = Literal::byte_string(name.as_bytes());
    let kind = tok.name_ident();
    quote! { (#bs, Self::#kind) }
  });
  let special = {
    let mut xs: Vec<_> = tokens.special.into_values().collect();
    xs.sort_unstable();
    xs
  };
  let desc_arms = punctuation
    .iter()
    .chain(keywords.iter())
    .map(|(name, tok)| {
      let desc = tok.desc.clone().unwrap_or_else(|| format!("`{name}`"));
      let kind = tok.name_ident();
      quote! { Self::#kind => #desc }
    })
    .chain(special.iter().filter_map(|tok| {
      let kind = tok.name_ident();
      let desc = tok.desc.as_ref()?;
      Some(quote! { Self::#kind => #desc })
    }));
  let doc_arms = punctuation
    .iter()
    .chain(keywords.iter())
    .map(|(_, tok)| tok)
    .chain(special.iter())
    .filter_map(|tok| {
      let doc = tok.doc.as_ref()?;
      let kind = tok.name_ident();
      Some(quote! { Self::#kind => #doc })
    });
  let self_trivia = trivia.iter().map(|id| {
    quote! { Self::#id }
  });
  // the order is intentional
  let syntax_kinds: Vec<_> = trivia
    .iter()
    .cloned()
    .chain(
      keywords
        .iter()
        .chain(punctuation.iter())
        .map(|(_, tok)| tok.name_ident()),
    )
    .chain(special.iter().map(|tok| tok.name_ident()))
    .chain(node_syntax_kinds)
    .collect();
  let last_syntax_kind = syntax_kinds.last().unwrap();
  let kind = quote! {
    use std::fmt;

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

      pub fn token_doc(&self) -> Option<&'static str> {
        let ret = match *self {
          #(#doc_arms ,)*
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

    impl fmt::Display for SyntaxKind {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.token_desc() {
          None => write!(f, "{:?}", self),
          Some(s) => f.write_str(s),
        }
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
    use crate::kind::{SyntaxKind as SK, SyntaxNode, SyntaxToken, #lang};
    pub use rowan::ast::{AstNode, AstPtr};

    pub type SyntaxNodePtr = rowan::ast::SyntaxNodePtr<#lang>;

    #[allow(unused)]
    fn tokens<P>(parent: &P, kind: SK) -> impl Iterator<Item = SyntaxToken>
    where
      P: AstNode<Language = #lang>,
    {
      parent
        .syntax()
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter(move |tok| tok.kind() == kind)
    }

    #[allow(unused)]
    fn token_children<P, C>(parent: &P) -> impl Iterator<Item = C>
    where
      P: AstNode<Language = #lang>,
      SyntaxToken: TryInto<C>,
    {
      parent
        .syntax()
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter_map(|x| x.try_into().ok())
    }

    #[allow(unused)]
    fn node_children<P, C>(parent: &P) -> impl Iterator<Item = C>
    where
      P: AstNode<Language = #lang>,
      C: AstNode<Language = #lang>,
    {
      parent.syntax().children().filter_map(C::cast)
    }

    #(#types)*
  };
  util::write_rust_file(
    out_dir.join("kind.rs").as_path(),
    kind.to_string().as_str(),
  )?;
  util::write_rust_file(
    out_dir.join("ast.rs").as_path(),
    ast.to_string().as_str(),
  )?;
  Ok(())
}
