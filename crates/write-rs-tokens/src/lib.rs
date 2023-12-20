//! See [`go`].

/// Write a Rust token stream out to the file name `basename` in `OUT_DIR`. The token stream will be
/// formatted.
///
/// # Panics
///
/// If that failed.
pub fn go(token_stream: proc_macro2::TokenStream, basename: &str) {
  let out_dir = std::env::var_os("OUT_DIR").expect("no OUT_DIR env var");
  let dst = std::path::Path::new(&out_dir).join(basename);
  let file = syn::parse2(token_stream).expect("syn parse failed");
  let formatted = prettyplease::unparse(&file);
  std::fs::write(dst, formatted).expect("io failed");
}
