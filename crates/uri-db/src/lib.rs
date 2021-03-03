//! A database of URIs of C0 source files.
//!
//! This depends on the `url` crate, but we call them "URIs". Basically, we're
//! just following what `lsp-types` calls them.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

pub use url::Url as Uri;

use rustc_hash::FxHashMap;
use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Index;

/// A URI database, which can only hold up to `u32::MAX` many items.
#[derive(Debug, Default)]
pub struct UriDb {
  id_to_uri: Vec<Option<Uri>>,
  uri_to_id: FxHashMap<Uri, UriId>,
}

impl UriDb {
  /// Inserts a URI into the database.
  pub fn insert(&mut self, uri: Uri) -> UriId {
    if let Some(ret) = self.get_id(&uri) {
      return ret;
    }
    let ret = UriId(self.id_to_uri.len() as u32);
    self.id_to_uri.push(Some(uri.clone()));
    assert!(self.uri_to_id.insert(uri, ret).is_none());
    ret
  }

  /// Removes a URI from the database.
  ///
  /// Returns the ID of the URI if it was in the database.
  pub fn remove(&mut self, uri: &Uri) -> Option<UriId> {
    let id = self.uri_to_id.remove(uri)?;
    self.id_to_uri[id.0 as usize] = None;
    Some(id)
  }

  /// Returns the ID associated with this URI.
  pub fn get_id<Q>(&self, key: &Q) -> Option<UriId>
  where
    Uri: Borrow<Q>,
    Q: ?Sized + Hash + Eq,
  {
    self.uri_to_id.get(key).copied()
  }

  /// Returns the URI associated with this ID.
  pub fn get(&self, file_id: UriId) -> Option<&Uri> {
    self.id_to_uri.get(file_id.0 as usize)?.as_ref()
  }

  /// Returns an iterator over the IDs.
  pub fn iter(&self) -> impl Iterator<Item = UriId> {
    (0..(self.id_to_uri.len() as u32)).map(UriId)
  }
}

impl Index<UriId> for UriDb {
  type Output = Uri;
  fn index(&self, index: UriId) -> &Self::Output {
    self.get(index).expect("no URI for ID")
  }
}

/// A URI identifier.
///
/// Yes, this is a "uniform resource identifier identifier". We only use this to
/// avoid cloning URIs all over the place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UriId(u32);
