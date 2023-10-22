use crate::token::ident;
use crate::util::{unwrap_node, unwrap_token, Cx};
use fast_hash::FxHashSet;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use ungrammar::Rule;

pub(crate) fn get(
  cx: &Cx,
  token_alts: &mut FxHashSet<Ident>,
  name: Ident,
  rules: &[Rule],
) -> TokenStream {
  match rules.first().unwrap() {
    Rule::Node(_) => get_nodes(cx, &name, rules),
    Rule::Token(_) => {
      let ts = get_tokens(cx, &name, rules);
      token_alts.insert(name);
      ts
    }
    bad => panic!("bad alt rule {bad:?}"),
  }
}

fn get_nodes(cx: &Cx, name: &Ident, rules: &[Rule]) -> TokenStream {
  let lang = &cx.lang;
  let mut defs = Vec::with_capacity(rules.len());
  let mut kinds = Vec::with_capacity(rules.len());
  let mut casts = Vec::with_capacity(rules.len());
  let mut syntaxes = Vec::with_capacity(rules.len());
  for rule in rules {
    let name = ident(&cx.grammar[unwrap_node(rule)].name);
    defs.push(quote! { #name(#name) });
    kinds.push(quote! { SK::#name });
    casts.push(quote! { SK::#name => Self::#name(#name(node)) });
    syntaxes.push(quote! { Self::#name(x) => x.syntax() });
  }
  quote! {
    pub enum #name {
      #(#defs ,)*
    }
    impl AstNode for #name {
      type Language = #lang;

      fn can_cast(kind: SK) -> bool {
        matches!(kind, #(#kinds)|*)
      }

      fn cast(node: SyntaxNode) -> Option<Self> {
        let ret = match node.kind() {
          #(#casts ,)*
          _ => return None,
        };
        Some(ret)
      }

      fn syntax(&self) -> &SyntaxNode {
        match self {
          #(#syntaxes ,)*
        }
      }
    }
  }
}

fn get_tokens(cx: &Cx, name: &Ident, rules: &[Rule]) -> TokenStream {
  let name_kind = format_ident!("{}Kind", name);
  let mut defs = Vec::with_capacity(rules.len());
  let mut casts = Vec::with_capacity(rules.len());
  let mut to_strs = Vec::with_capacity(rules.len());
  for rule in rules {
    let tok = unwrap_token(rule);
    let name = cx.tokens.get(tok).name_ident();
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
