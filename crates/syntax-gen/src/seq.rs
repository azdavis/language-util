use crate::token::ident;
use crate::util::Cx;
use identifier_case::pascal_to_snake;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::hash::Hash;
use ungrammar::Rule;

pub(crate) fn get(cx: &Cx, name: &Ident, rules: &[Rule]) -> TokenStream {
  let lang = &cx.lang;
  let mut counts = Counts::default();
  let fields = rules.iter().map(|rule| field(cx, &mut counts, rule));
  let mut derives = quote! {};
  let mut extra_impl = quote! {};
  if name == "Root" {
    derives = quote! { #[derive(Clone)] };
    extra_impl = quote! {
      impl std::fmt::Debug for #name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          self.syntax().fmt(f)
        }
      }
    }
  }
  quote! {
    #derives
    pub struct #name(SyntaxNode);
    impl #name {
      #(#fields)*
    }
    #extra_impl
    impl AstNode for #name {
      type Language = #lang;

      fn can_cast(kind: SK) -> bool {
        kind == SK::#name
      }

      fn cast(node: SyntaxNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
      }

      fn syntax(&self) -> &SyntaxNode {
        &self.0
      }
    }
  }
}

type Counts<T> = fast_hash::FxHashMap<T, usize>;

fn get_idx<T>(counts: &mut Counts<T>, key: T) -> usize
where
  T: Hash + Eq,
{
  let entry = counts.entry(key).or_default();
  let ret = *entry;
  *entry += 1;
  ret
}

#[derive(Debug)]
enum Modifier {
  Regular,
  Repeated,
  Optional,
}

impl Modifier {
  fn is_regular(&self) -> bool {
    matches!(self, Self::Regular)
  }
}

fn field<'cx>(cx: &'cx Cx, counts: &mut Counts<&'cx str>, mut rule: &Rule) -> TokenStream {
  let mut modifier = Modifier::Regular;
  let mut label: Option<&str> = None;
  let name: &str;
  let base_ty: Ident;
  let base_body: TokenStream;
  loop {
    match rule {
      Rule::Node(node) => {
        name = cx.grammar[*node].name.as_str();
        base_ty = ident(name);
        base_body = if cx.token_alts.contains(&base_ty) {
          quote! { token_children(self) }
        } else {
          quote! { node_children(self) }
        };
        break;
      }
      Rule::Token(tok) => {
        name = cx.tokens.get(*tok).name.as_str();
        base_ty = ident("SyntaxToken");
        let name_ident = ident(name);
        base_body = quote! { tokens(self, SK::#name_ident) };
        break;
      }
      Rule::Labeled { label: l, rule: r } => {
        if let Some(old) = label {
          panic!("already have label {old}, cannot have new label {l}");
        }
        label = Some(l.as_str());
        rule = r.as_ref();
      }
      Rule::Opt(r) => {
        assert!(modifier.is_regular(), "cannot make optional");
        modifier = Modifier::Optional;
        rule = r.as_ref();
      }
      Rule::Rep(r) => {
        assert!(modifier.is_regular(), "cannot make repeated");
        modifier = Modifier::Repeated;
        rule = r.as_ref();
      }
      Rule::Seq(_) | Rule::Alt(_) => panic!("bad field rule: {rule:?}"),
    }
  }
  #[allow(clippy::single_match_else)]
  let field_name = match label {
    Some(x) => ident(x),
    None => {
      let to_snake = pascal_to_snake(name);
      match modifier {
        Modifier::Repeated => format_ident!("{to_snake}s"),
        Modifier::Optional | Modifier::Regular => ident(&to_snake),
      }
    }
  };
  let idx = get_idx(counts, name);
  let ret_ty: TokenStream;
  let body: TokenStream;
  match modifier {
    Modifier::Repeated => {
      ret_ty = quote! { impl Iterator<Item = #base_ty> };
      body = base_body;
    }
    Modifier::Optional | Modifier::Regular => {
      ret_ty = quote! { Option<#base_ty> };
      body = if idx == 0 {
        quote! { #base_body.next() }
      } else {
        quote! { #base_body.nth(#idx) }
      };
    }
  };
  quote! {
    pub fn #field_name(&self) -> #ret_ty {
      #body
    }
  }
}
