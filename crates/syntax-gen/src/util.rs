use crate::token::TokenDb;
use proc_macro2::Ident;
use quote::format_ident;
use std::fs::OpenOptions;
use std::io::{Result, Write as _};
use std::process::{Command, Stdio};
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

pub(crate) fn write_rust_file(name: &str, contents: &str) -> Result<()> {
  let mut prog = Command::new("rustfmt")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
  let mut stdout = prog.stdout.take().unwrap();
  let mut out_file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(name)?;
  prog.stdin.take().unwrap().write_all(contents.as_bytes())?;
  std::io::copy(&mut stdout, &mut out_file)?;
  assert!(prog.wait()?.success());
  Ok(())
}
