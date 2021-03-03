use crate::token::TokenDb;
use proc_macro2::Ident;
use quote::format_ident;
use ungrammar::{Grammar, Node, Rule, Token};

#[derive(Debug)]
pub(crate) struct Cx {
  pub(crate) grammar: Grammar,
  pub(crate) tokens: TokenDb,
}

pub(crate) fn ident(s: &str) -> Ident {
  format_ident!("{}", s)
}

pub(crate) fn unwrap_node(rule: &Rule) -> Node {
  match rule {
    Rule::Node(node) => *node,
    _ => panic!("unwrap_node on {:?}", rule),
  }
}

pub(crate) fn unwrap_token(rule: &Rule) -> Token {
  match rule {
    Rule::Token(tok) => *tok,
    _ => panic!("unwrap_token on {:?}", rule),
  }
}
