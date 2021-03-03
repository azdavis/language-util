use crate::util::{ident, unwrap_node, unwrap_token, Cx};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use ungrammar::Rule;

pub(crate) fn get(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  match rules.first().unwrap() {
    Rule::Node(_) => get_nodes(cx, name, rules),
    Rule::Token(_) => get_tokens(cx, name, rules),
    bad => panic!("bad alt rule {:?}", bad),
  }
}

fn get_nodes(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let mut defs = Vec::with_capacity(rules.len());
  let mut casts = Vec::with_capacity(rules.len());
  let mut syntaxes = Vec::with_capacity(rules.len());
  for rule in rules {
    let name = ident(&cx.grammar[unwrap_node(rule)].name);
    defs.push(quote! { #name(#name) });
    casts.push(quote! { SK::#name => Self::#name(#name(node)) });
    syntaxes.push(quote! { Self::#name(x) => x.syntax() });
  }
  quote! {
    pub enum #name {
      #(#defs ,)*
    }
    impl Cast for #name {
      fn cast(elem: SyntaxElement) -> Option<Self> {
        let node = elem.into_node()?;
        let ret = match node.kind() {
          #(#casts ,)*
          _ => return None,
        };
        Some(ret)
      }
    }
    impl Syntax for #name {
      fn syntax(&self) -> &SyntaxNode {
        match self {
          #(#syntaxes ,)*
        }
      }
    }
  }
}

fn get_tokens(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let name_kind = format_ident!("{}Kind", name);
  let mut defs = Vec::with_capacity(rules.len());
  let mut casts = Vec::with_capacity(rules.len());
  let mut to_strs = Vec::with_capacity(rules.len());
  for rule in rules {
    let tok = unwrap_token(rule);
    let name = ident(&cx.tokens.name(tok));
    let text = cx.grammar[tok].name.as_str();
    defs.push(quote! { #name });
    casts.push(quote! { SK::#name => #name_kind::#name });
    to_strs.push(quote! { Self::#name => #text });
  }
  quote! {
    pub enum #name_kind {
      #(#defs ,)*
    }
    impl #name_kind {
      pub fn to_str(&self) -> &'static str {
        match *self {
          #(#to_strs ,)*
        }
      }
    }
    pub struct #name {
      pub token: SyntaxToken,
      pub kind: #name_kind,
    }
    impl Cast for #name {
      fn cast(elem: SyntaxElement) -> Option<Self> {
        let token = elem.into_token()?;
        let kind = match token.kind() {
          #(#casts ,)*
          _ => return None,
        };
        Some(Self { kind, token })
      }
    }
  }
}
