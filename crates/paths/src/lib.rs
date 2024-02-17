//! Types for working with paths.

pub use glob::{GlobError, PatternError};

use fast_hash::FxHashMap;
use std::path::{Path, PathBuf};

/// A store of paths.
#[derive(Debug, Default)]
pub struct Store {
  id_to_path: Vec<AbsPathBuf>,
  path_to_id: FxHashMap<AbsPathBuf, PathId>,
}

impl Store {
  /// Returns a new `Store`.
  #[must_use]
  pub fn new() -> Self {
    Store::default()
  }

  /// Returns an ID for this path.
  pub fn get_id(&mut self, path: &AbsPathBuf) -> PathId {
    if let Some(x) = self.path_to_id.get(path) {
      *x
    } else {
      let id = PathId(idx::Idx::new(self.id_to_path.len()));
      self.id_to_path.push(path.clone());
      self.path_to_id.insert(path.clone(), id);
      id
    }
  }

  /// Like `get_id` but the `path` is owned, possibly saving a clone.
  pub fn get_id_owned(&mut self, path: AbsPathBuf) -> PathId {
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
  pub fn get_path(&self, id: PathId) -> &AbsPathBuf {
    &self.id_to_path[id.0.to_usize()]
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

/// A map from paths to something.
pub type PathMap<T> = nohash_hasher::IntMap<PathId, T>;

/// An absolute path buffer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPathBuf(PathBuf);

impl AbsPathBuf {
  /// Returns a new [`AbsPathBuf`] if the [`PathBuf`] is absolute.
  #[must_use]
  pub fn try_new(path: PathBuf) -> Option<Self> {
    path.is_absolute().then_some(Self(path))
  }

  /// Returns the underlying [`Path`].
  #[must_use]
  pub fn as_path(&self) -> &Path {
    self.0.as_path()
  }

  /// Turns this into a [`PathBuf`].
  #[must_use]
  pub fn into_path_buf(self) -> PathBuf {
    self.0
  }

  /// Pushes `path` onto `self`.
  ///
  /// - If `path` is absolute, it replaces `self`.
  /// - If `path` is relative, it is appended onto `self`.
  ///
  /// Either way, `self` remains absolute.
  pub fn push<P: AsRef<Path>>(&mut self, path: P) {
    self.0.push(path);
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
  /// Read the entries of a directory. The vec is in arbitrary order.
  ///
  /// # Errors
  ///
  /// If the filesystem failed us.
  fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>>;
  /// Returns whether this is a file. If unknown, returns false.
  fn is_file(&self, path: &Path) -> bool;
  /// An iterator of paths from `glob`.
  type GlobPaths: Iterator<Item = glob::GlobResult>;
  /// Glob the file system.
  ///
  /// # Errors
  ///
  /// If the pattern is invalid.
  fn glob(&self, pattern: &str) -> Result<Self::GlobPaths, glob::PatternError>;
}

/// The real file system. Does actual I/O.
#[derive(Debug, Default)]
pub struct RealFileSystem(());

impl FileSystem for RealFileSystem {
  fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
  }

  fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
    std::fs::read_dir(path)?.map(|x| Ok(x?.path())).collect()
  }

  fn is_file(&self, path: &Path) -> bool {
    path.is_file()
  }

  type GlobPaths = glob::Paths;

  fn glob(&self, pattern: &str) -> Result<Self::GlobPaths, glob::PatternError> {
    glob::glob(pattern)
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

  fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
    Ok(self.0.keys().filter(|&p| p.starts_with(path) && p != path).cloned().collect())
  }

  fn is_file(&self, path: &Path) -> bool {
    self.0.contains_key(path)
  }

  type GlobPaths = std::vec::IntoIter<glob::GlobResult>;

  fn glob(&self, pattern: &str) -> Result<Self::GlobPaths, glob::PatternError> {
    let cs: Vec<_> = Path::new(pattern).components().collect();
    #[allow(clippy::needless_collect)]
    let ret: Vec<_> = self
      .0
      .keys()
      .filter_map(|path| {
        if cs.len() != path.components().count() {
          return None;
        }
        cs.iter()
          .zip(path.components())
          .all(|(&c, p)| c == std::path::Component::Normal(std::ffi::OsStr::new("*")) || c == p)
          .then(|| Ok(path.clone()))
      })
      .collect();
    Ok(ret.into_iter())
  }
}
