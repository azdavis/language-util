//! Types for working with paths.

pub use glob::{GlobError, PatternError};

use fast_hash::FxHashMap;
use std::path::{Path, PathBuf};

/// A store of paths.
#[derive(Debug, Default)]
pub struct Store {
  id_to_path: Vec<CanonicalPathBuf>,
  path_to_id: FxHashMap<CanonicalPathBuf, PathId>,
}

impl Store {
  /// Returns a new `Store`.
  #[must_use]
  pub fn new() -> Self {
    Store::default()
  }

  /// Returns an ID for this path.
  pub fn get_id(&mut self, path: &CanonicalPath) -> PathId {
    if let Some(x) = self.path_to_id.get(path) {
      *x
    } else {
      let id = PathId(idx::Idx::new(self.id_to_path.len()));
      self.id_to_path.push(path.to_owned());
      self.path_to_id.insert(path.to_owned(), id);
      id
    }
  }

  /// Like `get_id` but the `path` is owned, possibly saving a clone.
  pub fn get_id_owned(&mut self, path: CanonicalPathBuf) -> PathId {
    if let Some(x) = self.path_to_id.get(&path) {
      *x
    } else {
      let id = PathId(idx::Idx::new(self.id_to_path.len()));
      self.id_to_path.push(path.clone());
      self.path_to_id.insert(path, id);
      id
    }
  }

  /// Returns the path for this ID.
  #[must_use]
  pub fn get_path(&self, id: PathId) -> &CanonicalPath {
    self.id_to_path[id.0.to_usize()].as_canonical_path()
  }

  /// Combine `other` into `self`.
  ///
  /// After the call, `self` will contain all the paths that were in `other`.
  ///
  /// Calls `f` with the `PathId`s for each path in `other` according to `(other, self)`.
  pub fn combine<F>(&mut self, other: Self, f: &mut F)
  where
    F: FnMut(PathId, PathId),
  {
    for (idx, path) in other.id_to_path.into_iter().enumerate() {
      let old = PathId(idx::Idx::new(idx));
      let new = self.get_id_owned(path);
      f(old, new);
    }
  }
}

/// A path identifier. Cheap to copy and compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathId(idx::Idx);

impl PathId {
  /// Wrap a value with the path id.
  pub fn wrap<T>(self, val: T) -> WithPath<T> {
    WithPath { path: self, val }
  }
}

impl nohash_hasher::IsEnabled for PathId {}

/// A pair of path id and value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WithPath<T> {
  /// The path id.
  pub path: PathId,
  /// The value.
  pub val: T,
}

/// A map from path IDs to something.
pub type PathMap<T> = nohash_hasher::IntMap<PathId, T>;

/// A canonical and thus absolute path.
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CanonicalPath(Path);

impl ToOwned for CanonicalPath {
  type Owned = CanonicalPathBuf;

  fn to_owned(&self) -> Self::Owned {
    CanonicalPathBuf(self.as_path().to_owned())
  }
}

impl CanonicalPath {
  fn new_unchecked(path: &Path) -> &Self {
    let ptr = std::ptr::from_ref(path) as *const CanonicalPath;
    // SAFETY: CanonicalPath is repr(transparent)ly Path
    unsafe { &*ptr }
  }

  /// Returns the underlying [`Path`].
  #[must_use]
  pub fn as_path(&self) -> &Path {
    &self.0
  }

  /// Returns the parent of this. If it exists, it will be canonical.
  pub fn parent(&self) -> Option<&CanonicalPath> {
    self.0.parent().map(CanonicalPath::new_unchecked)
  }
}

/// A canonical and thus absolute path buffer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CanonicalPathBuf(PathBuf);

impl std::borrow::Borrow<CanonicalPath> for CanonicalPathBuf {
  fn borrow(&self) -> &CanonicalPath {
    self.as_canonical_path()
  }
}

impl CanonicalPathBuf {
  /// Returns this as an [`CanonicalPath`].
  #[must_use]
  pub fn as_canonical_path(&self) -> &CanonicalPath {
    CanonicalPath::new_unchecked(self.0.as_path())
  }

  /// Returns the underlying [`Path`].
  #[must_use]
  pub fn as_path(&self) -> &Path {
    &self.0
  }

  /// Turns this into a [`PathBuf`].
  #[must_use]
  pub fn into_path_buf(self) -> PathBuf {
    self.0
  }
}

/// A file system.
pub trait FileSystem {
  /// Read the contents of a file.
  ///
  /// # Errors
  ///
  /// If the filesystem failed us.
  fn read_to_string(&self, path: &Path) -> std::io::Result<String>;

  /// Read the contents of a file as bytes.
  ///
  /// # Errors
  ///
  /// If the filesystem failed us.
  fn read_to_bytes(&self, path: &Path) -> std::io::Result<Vec<u8>>;

  /// Make a path canonical.
  ///
  /// # Errors
  ///
  /// If the filesystem failed us.
  fn canonical(&self, path: &Path) -> std::io::Result<CanonicalPathBuf>;

  /// Read the entries of a directory. The vec is in arbitrary order.
  ///
  /// # Errors
  ///
  /// If the filesystem failed us.
  fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>>;

  /// Returns whether this is a file. If unknown, returns false.
  fn is_file(&self, path: &Path) -> bool;
}

/// The real file system. Does actual I/O.
#[derive(Debug, Default)]
pub struct RealFileSystem(());

impl FileSystem for RealFileSystem {
  fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
  }

  fn read_to_bytes(&self, path: &Path) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
  }

  fn canonical(&self, path: &Path) -> std::io::Result<CanonicalPathBuf> {
    dunce::canonicalize(path).map(CanonicalPathBuf)
  }

  fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
    std::fs::read_dir(path)?.map(|x| Ok(x?.path())).collect()
  }

  fn is_file(&self, path: &Path) -> bool {
    path.is_file()
  }
}

/// A 'file system' in memory.
///
/// Doesn't totally handle all `Path`s. For instance, it probably gives unexpected results for paths
/// that:
/// - Have trailing `/`
/// - Have `.`
/// - Have `..`
/// - Do not start with `/`
///
/// Also, using `glob` doesn't actually glob.
///
/// But this is mainly intended for basic testing purposes, so it's fine.
#[derive(Debug, Default)]
pub struct MemoryFileSystem(FxHashMap<PathBuf, String>);

impl MemoryFileSystem {
  /// Returns a new `MemoryFileSystem`.
  #[must_use]
  pub fn new(map: FxHashMap<PathBuf, String>) -> Self {
    Self(map)
  }
}

impl FileSystem for MemoryFileSystem {
  fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
    match self.0.get(path) {
      Some(x) => Ok(x.clone()),
      None => Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
    }
  }

  fn read_to_bytes(&self, path: &Path) -> std::io::Result<Vec<u8>> {
    self.read_to_string(path).map(String::into_bytes)
  }

  fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
    Ok(self.0.keys().filter(|&p| p.starts_with(path) && p != path).cloned().collect())
  }

  fn is_file(&self, path: &Path) -> bool {
    self.0.contains_key(path)
  }

  fn canonical(&self, path: &Path) -> std::io::Result<CanonicalPathBuf> {
    Ok(CanonicalPathBuf(path.to_owned()))
  }
}

#[cfg(test)]
fn _obj_safe(_: &dyn FileSystem) {}
