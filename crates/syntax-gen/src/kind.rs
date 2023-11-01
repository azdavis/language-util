use crate::util::Cx;
use quote::quote;
use std::cmp::Reverse;

#[allow(clippy::too_many_lines)]
pub(crate) fn get(
  Cx { grammar, tokens, lang, .. }: Cx,
  trivia: &[proc_macro2::Ident],
  node_syntax_kinds: Vec<proc_macro2::Ident>,
  file: &str,
) -> proc_macro2::TokenStream {
  let keywords = {
    let mut xs: Vec<_> = tokens
      .keywords
      .into_iter()
      .map(|(ug_tok, tok)| (grammar[ug_tok].name.as_str(), tok))
      .collect();
    xs.sort_unstable_by_key(|&(name, _)| (Reverse(name.len()), name));
    xs
  };
  let keyword_arms = keywords.iter().map(|(name, tok)| {
    let bs = proc_macro2::Literal::byte_string(name.as_bytes());
    let kind = tok.name_ident();
    quote! { #bs => Self::#kind }
  });
  let punctuation = {
    let mut xs: Vec<_> = tokens
      .punctuation
      .into_iter()
      .map(|(ug_tok, tok)| (grammar[ug_tok].name.as_str(), tok))
      .collect();
    xs.sort_unstable_by_key(|&(name, _)| (Reverse(name.len()), name));
    xs
  };
  let punctuation_len = punctuation.len();
  let punctuation_elements = punctuation.iter().map(|(name, tok)| {
    let bs = proc_macro2::Literal::byte_string(name.as_bytes());
    let kind = tok.name_ident();
    quote! { (#bs, Self::#kind) }
  });
  let special = {
    let mut xs: Vec<_> = tokens.special.into_values().collect();
    xs.sort_unstable();
    xs
  };
  let desc_arms = punctuation
    .iter()
    .chain(keywords.iter())
    .map(|(name, tok)| {
      let desc = tok.desc.clone().unwrap_or_else(|| format!("`{name}`"));
      let kind = tok.name_ident();
      quote! { Self::#kind => #desc }
    })
    .chain(special.iter().filter_map(|tok| {
      let kind = tok.name_ident();
      let desc = tok.desc.as_ref()?;
      Some(quote! { Self::#kind => #desc })
    }));
  let doc_arms =
    punctuation.iter().chain(keywords.iter()).map(|(_, tok)| tok).chain(special.iter()).filter_map(
      |tok| {
        let doc = tok.doc.as_ref()?;
        let kind = tok.name_ident();
        Some(quote! { Self::#kind => #doc })
      },
    );
  let self_trivia = trivia.iter().map(|id| {
    quote! { Self::#id }
  });
  // the order is intentional
  let syntax_kinds: Vec<_> = trivia
    .iter()
    .cloned()
    .chain(keywords.iter().chain(punctuation.iter()).map(|(_, tok)| tok.name_ident()))
    .chain(special.iter().map(crate::token::Token::name_ident))
    .chain(node_syntax_kinds)
    .collect();
  let last_syntax_kind = syntax_kinds.last().unwrap();
  quote! {
    use std::fmt;

    pub const GENERATED_BY: &str = #file;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(u16)]
    pub enum SyntaxKind {
      #(#syntax_kinds ,)*
    }

    impl SyntaxKind {
      pub const PUNCTUATION: [(&'static [u8], Self); #punctuation_len] = [
        #(#punctuation_elements ,)*
      ];

      pub fn keyword(bs: &[u8]) -> Option<Self> {
        let ret = match bs {
          #(#keyword_arms ,)*
          _ => return None,
        };
        Some(ret)
      }

      pub fn token_desc(&self) -> Option<&'static str> {
        let ret = match *self {
          #(#desc_arms ,)*
          _ => return None,
        };
        Some(ret)
      }

      pub fn token_doc(&self) -> Option<&'static str> {
        let ret = match *self {
          #(#doc_arms ,)*
          _ => return None,
        };
        Some(ret)
      }
    }

    impl token::Triviable for SyntaxKind {
      fn is_trivia(&self) -> bool {
        matches!(*self, #(#self_trivia)|*)
      }
    }

    impl fmt::Display for SyntaxKind {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.token_desc() {
          None => fmt::Debug::fmt(self, f),
          Some(s) => f.write_str(s),
        }
      }
    }

    impl From<SyntaxKind> for rowan::SyntaxKind {
      fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
      }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum #lang {}

    impl rowan::Language for #lang {
      type Kind = SyntaxKind;

      fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::#last_syntax_kind as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
      }

      fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
      }
    }

    pub type SyntaxNode = rowan::SyntaxNode<#lang>;
    pub type SyntaxToken = rowan::SyntaxToken<#lang>;
    pub type SyntaxElement = rowan::SyntaxElement<#lang>;
  }
}
