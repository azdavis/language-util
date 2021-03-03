use crate::util::{ident, unwrap_node, Cx};
use identifier_case::pascal_to_snake;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::hash::Hash;
use ungrammar::{Node, Rule, Token};

pub(crate) fn get(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let mut counts = Counts::default();
  let fields = rules.iter().map(|rule| field(cx, &mut counts, rule));
  let derives = if name == "Root" {
    quote! { #[derive(Debug, Clone)] }
  } else {
    quote! {}
  };
  quote! {
    #derives
    pub struct #name(SyntaxNode);
    impl #name {
      #(#fields)*
    }
    impl Cast for #name {
      fn cast(elem: SyntaxElement) -> Option<Self> {
        let node = elem.into_node()?;
        (node.kind() == SK::#name).then(|| Self(node))
      }
    }
    impl Syntax for #name {
      fn syntax(&self) -> &SyntaxNode {
        &self.0
      }
    }
  }
}

type Counts<T> = rustc_hash::FxHashMap<T, usize>;

fn get_idx<T>(counts: &mut Counts<T>, key: T) -> usize
where
  T: Hash + Eq,
{
  let entry = counts.entry(key).or_default();
  let ret = *entry;
  *entry += 1;
  ret
}

enum Modifier {
  Rep,
  Opt,
  Regular,
}

fn field<'cx>(
  cx: &'cx Cx,
  counts: &mut Counts<&'cx str>,
  rule: &Rule,
) -> TokenStream {
  match rule {
    Rule::Labeled { label, rule } => {
      labeled_field(cx, counts, label.as_str(), rule)
    }
    Rule::Node(node) => node_field(cx, counts, Modifier::Regular, None, *node),
    Rule::Token(tok) => token_field(cx, counts, None, *tok),
    Rule::Opt(r) => node_field(cx, counts, Modifier::Opt, None, unwrap_node(r)),
    Rule::Rep(r) => node_field(cx, counts, Modifier::Rep, None, unwrap_node(r)),
    Rule::Alt(_) | Rule::Seq(_) => panic!("bad field rule {:?}", rule),
  }
}

fn token_field<'cx>(
  cx: &'cx Cx,
  counts: &mut Counts<&'cx str>,
  name: Option<&str>,
  token: Token,
) -> TokenStream {
  let kind = cx.tokens.name(token);
  let name = match name {
    None => ident(&pascal_to_snake(kind)),
    Some(x) => ident(x),
  };
  let idx = get_idx(counts, kind);
  let kind = ident(kind);
  quote! {
    pub fn #name(&self) -> Option<SyntaxToken> {
      token(self, SK::#kind, #idx)
    }
  }
}

fn labeled_field<'cx>(
  cx: &'cx Cx,
  counts: &mut Counts<&'cx str>,
  label: &str,
  rule: &Rule,
) -> TokenStream {
  match rule {
    Rule::Node(node) => {
      node_field(cx, counts, Modifier::Regular, Some(label), *node)
    }
    Rule::Token(tok) => token_field(cx, counts, Some(label), *tok),
    Rule::Opt(r) => {
      node_field(cx, counts, Modifier::Opt, Some(label), unwrap_node(r))
    }
    Rule::Labeled { .. } | Rule::Seq(_) | Rule::Alt(_) | Rule::Rep(_) => {
      panic!("bad labeled field rule {:?}", rule)
    }
  }
}

fn node_field<'cx>(
  cx: &'cx Cx,
  counts: &mut Counts<&'cx str>,
  modifier: Modifier,
  name: Option<&str>,
  node: Node,
) -> TokenStream {
  let kind = &cx.grammar[node].name;
  let idx = get_idx(counts, kind);
  let owned;
  let name = match name {
    None => {
      owned = pascal_to_snake(kind);
      &owned
    }
    Some(x) => x,
  };
  let kind = ident(kind);
  let name_ident;
  let ret_ty;
  let body;
  match modifier {
    Modifier::Rep => {
      name_ident = format_ident!("{}s", name);
      ret_ty = quote! { impl Iterator<Item = #kind> };
      body = quote! { children(self) };
    }
    Modifier::Opt | Modifier::Regular => {
      name_ident = ident(name);
      ret_ty = quote! { Option<#kind> };
      body = quote! { children(self).nth(#idx) };
    }
  }
  quote! {
    pub fn #name_ident(&self) -> #ret_ty {
      #body
    }
  }
}
