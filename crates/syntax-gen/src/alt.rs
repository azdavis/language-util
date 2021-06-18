use crate::util::{ident, unwrap_node, unwrap_token, Cx};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use rustc_hash::FxHashSet;
use ungrammar::Rule;

pub(crate) fn get(
  cx: &Cx,
  token_alts: &mut FxHashSet<Ident>,
  name: Ident,
  rules: &[Rule],
) -> TokenStream {
  match rules.first().unwrap() {
    Rule::Node(_) => get_nodes(cx, name, rules),
    Rule::Token(_) => {
      token_alts.insert(name.clone());
      get_tokens(cx, name, rules)
    }
    bad => panic!("bad alt rule {:?}", bad),
  }
}

fn get_nodes(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let lang = &cx.lang;
  let mut defs = Vec::with_capacity(rules.len());
  let mut casts = Vec::with_capacity(rules.len());
  let mut syntaxes = Vec::with_capacity(rules.len());
  for rule in rules {
    let name = ident(&cx.grammar[unwrap_node(rule)].name);
    defs.push(quote! { #name(#name) });
    casts.push(quote! { SK::#name => Self::#name(#name(node)) });
    syntaxes.push(quote! { Self::#name(x) => x.as_ref() });
  }
  quote! {
    pub enum #name {
      #(#defs ,)*
    }
    impl HasLanguage for #name {
      type Language = #lang;
    }
    impl TryFrom<SyntaxNode> for #name {
      type Error = ();
      fn try_from(node: SyntaxNode) -> Result<Self, Self::Error> {
        let ret = match node.kind() {
          #(#casts ,)*
          _ => return Err(()),
        };
        Ok(ret)
      }
    }
    impl AsRef<SyntaxNode> for #name {
      fn as_ref(&self) -> &SyntaxNode {
        match self {
          #(#syntaxes ,)*
        }
      }
    }
  }
}

fn get_tokens(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let lang = &cx.lang;
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
    impl HasLanguage for #name {
      type Language = #lang;
    }
    impl TryFrom<SyntaxToken> for #name {
      type Error = ();
      fn try_from(token: SyntaxToken) -> Result<Self, Self::Error> {
        let kind = match token.kind() {
          #(#casts ,)*
          _ => return Err(()),
        };
        Ok(Self { token, kind })
      }
    }
  }
}
