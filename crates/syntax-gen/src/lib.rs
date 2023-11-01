//! Generates opinionated Rust code from an [ungrammar][1].
//!
//! [1]: https://github.com/rust-analyzer/ungrammar

#![deny(clippy::pedantic, missing_debug_implementations, missing_docs, rust_2018_idioms)]

mod alt;
mod ast;
mod kind;
mod seq;
mod token;
mod util;

use crate::util::Cx;
use fast_hash::FxHashSet;
use std::{collections::HashMap, hash::BuildHasher};
use ungrammar::{Grammar, Rule};

pub use token::{Kind as TokenKind, Token};

/// The options to pass to `gen`.
#[derive(Debug)]
pub struct Options<'a, S> {
  /// The name of the language.
  pub lang: &'a str,
  /// A list of all the `SyntaxKind`s which should be made as trivia.
  pub trivia: &'a [&'a str],
  /// Text of the ungrammar for the language, possibly via `include_str!`.
  pub grammar: &'a str,
  /// A map from token names to documentation.
  pub doc: &'a HashMap<&'a str, &'a str, S>,
  /// A map from special tokens names to descriptions for those tokens.
  pub special: &'a HashMap<&'a str, &'a str, S>,
}

/// Generates Rust code from the `grammar` of the `lang` and writes it to two files:
///
/// - `$OUT_DIR/kind.rs`, which will contain definitions for the language's `SyntaxKind` and
///   associated types, using all the different tokens extracted from `grammar`.
/// - `$OUT_DIR/ast.rs`, which will contain a strongly-typed API for traversing an abstract syntax
///   tree, based on the `grammar`.
///
/// The generated Rust files will depend on:
///
/// - `rowan` from crates.io
/// - `token` from language-util
///
/// # Panics
///
/// If this process failed.
pub fn gen<S>(opts: &Options<'_, S>)
where
  S: BuildHasher,
{
  let lang = token::ident(opts.lang);
  let grammar: Grammar = opts.grammar.parse().expect("couldn't parse ungrammar");
  let tokens = token::TokenDb::new(&grammar, opts.doc, opts.special);
  let mut types = Vec::<proc_macro2::TokenStream>::new();
  let mut node_syntax_kinds = Vec::<proc_macro2::Ident>::new();
  let mut cx = Cx { lang, grammar, tokens, token_alts: FxHashSet::default() };
  let mut token_alts = FxHashSet::default();
  // first process all the alts
  for node in cx.grammar.iter() {
    let data = &cx.grammar[node];
    let rules = match &data.rule {
      Rule::Alt(rules) => rules.as_slice(),
      _ => continue,
    };
    types.push(alt::get(&cx, &mut token_alts, token::ident(&data.name), rules));
  }
  // it would be nicer if we could just mutate token_alts on the cx but we have an active shared
  // borrow to iterate over the grammar. so we use a kludge.
  cx.token_alts = token_alts;
  // then everything else
  for node in cx.grammar.iter() {
    let data = &cx.grammar[node];
    let rules = match &data.rule {
      Rule::Alt(_) => continue,
      Rule::Seq(rules) => rules.as_slice(),
      rule => std::slice::from_ref(rule),
    };
    let name = token::ident(&data.name);
    node_syntax_kinds.push(name.clone());
    types.push(seq::get(&cx, &name, rules));
  }
  let ast_rs = ast::get(&cx.lang, &types);
  write_output(ast_rs, "ast.rs");
  let trivia: Vec<_> = opts.trivia.iter().map(|&x| token::ident(x)).collect();
  let kind_rs = kind::get(cx, &trivia, node_syntax_kinds);
  write_output(kind_rs, "kind.rs");
}

fn write_output(output: proc_macro2::TokenStream, basename: &str) {
  let out_dir = std::env::var_os("OUT_DIR").unwrap();
  let dst = std::path::Path::new(&out_dir).join(basename);
  let file = syn::parse2(output).unwrap();
  let formatted = prettyplease::unparse(&file);
  std::fs::write(dst, formatted).unwrap();
}
