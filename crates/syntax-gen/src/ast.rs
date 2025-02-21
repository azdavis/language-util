use quote::quote;

pub(crate) fn get(
  lang: &proc_macro2::Ident,
  types: &[proc_macro2::TokenStream],
  file: &str,
) -> proc_macro2::TokenStream {
  quote! {
    use crate::kind::{SyntaxKind as SK, SyntaxNode, SyntaxToken, #lang};
    pub use rowan::ast::{AstNode, AstPtr};

    pub const GENERATED_BY: &str = #file;

    pub type SyntaxNodePtr = rowan::ast::SyntaxNodePtr<#lang>;

    #[allow(unused)]
    fn tokens<P>(parent: &P, kind: SK) -> impl Iterator<Item = SyntaxToken> + use<P>
    where
      P: AstNode<Language = #lang>,
    {
      parent
        .syntax()
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter(move |tok| tok.kind() == kind)
    }

    #[allow(unused)]
    fn token_children<P, C>(parent: &P) -> impl Iterator<Item = C> + use<P, C>
    where
      P: AstNode<Language = #lang>,
      SyntaxToken: TryInto<C>,
    {
      parent
        .syntax()
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter_map(|x| x.try_into().ok())
    }

    #[allow(unused)]
    fn node_children<P, C>(parent: &P) -> impl Iterator<Item = C> + use<P, C>
    where
      P: AstNode<Language = #lang>,
      C: AstNode<Language = #lang>,
    {
      parent.syntax().children().filter_map(C::cast)
    }

    #(#types)*
  }
}
