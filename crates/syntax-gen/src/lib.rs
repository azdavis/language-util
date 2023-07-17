//! Generates opinionated Rust code from an [ungrammar][1].
//!
//! [1]: https://github.com/rust-analyzer/ungrammar

#![deny(missing_debug_implementations, missing_docs, rust_2018_idioms)]

mod alt;
mod ast;
mod kind;
mod seq;
mod token;
mod util;

use crate::util::Cx;
use fast_hash::FxHashSet;
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
  let lang = token::ident(lang);
  let tokens = token::TokenDb::new(&grammar, get_token);
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
    let name = token::ident(&data.name);
    node_syntax_kinds.push(name.clone());
    types.push(seq::get(&cx, name, rules));
  }
  let ast_rs = ast::get(&cx.lang, types);
  util::write_rust_file(out_dir.join("ast.rs").as_path(), ast_rs.to_string().as_str())?;
  let trivia: Vec<_> = trivia.iter().map(|&x| token::ident(x)).collect();
  let kind_rs = kind::get(cx, trivia, node_syntax_kinds);
  util::write_rust_file(out_dir.join("kind.rs").as_path(), kind_rs.to_string().as_str())?;
  Ok(())
}
