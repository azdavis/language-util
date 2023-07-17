use crate::token::TokenDb;
use fast_hash::FxHashSet;
use proc_macro2::Ident;
use ungrammar::{Grammar, Node, Rule, Token};

#[derive(Debug)]
pub(crate) struct Cx {
  pub(crate) lang: Ident,
  pub(crate) grammar: Grammar,
  pub(crate) tokens: TokenDb,
  pub(crate) token_alts: FxHashSet<Ident>,
}

pub(crate) fn unwrap_node(rule: &Rule) -> Node {
  match rule {
    Rule::Node(node) => *node,
    _ => panic!("unwrap_node on {rule:?}"),
  }
}

pub(crate) fn unwrap_token(rule: &Rule) -> Token {
  match rule {
    Rule::Token(tok) => *tok,
    _ => panic!("unwrap_token on {rule:?}"),
  }
}
