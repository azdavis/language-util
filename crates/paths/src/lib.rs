//! Types for working with paths.

use fast_hash::FxHashMap;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// A store of paths.
#[derive(Debug, Default)]
pub struct Store {
  id_to_path: Vec<CleanPathBuf>,
  path_to_id: FxHashMap<CleanPathBuf, PathId>,
}

impl Store {
  /// Returns a new `Store`.
  #[must_use]
  pub fn new() -> Self {
    Store::default()
  }

  /// Returns an ID for this path.
  pub fn get_id(&mut self, path: &CleanPath) -> PathId {
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
  pub fn get_id_owned(&mut self, path: CleanPathBuf) -> PathId {
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
  pub fn get_path(&self, id: PathId) -> &CleanPath {
    self.id_to_path[id.0.to_usize()].as_clean_path()
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

/// A set of path IDs.
pub type PathSet = nohash_hasher::IntSet<PathId>;

/// A clean path.
///
/// "Clean" paths are absolute and contain no `.` or `..`.
///
/// They may, however, not be canonical because of symlinks.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CleanPath(Path);

impl ToOwned for CleanPath {
  type Owned = CleanPathBuf;

  fn to_owned(&self) -> Self::Owned {
    CleanPathBuf(self.as_path().to_owned())
  }
}

impl CleanPath {
  fn new_unchecked(path: &Path) -> &Self {
    let ptr = std::ptr::from_ref(path) as *const CleanPath;
    // SAFETY: CleanPath is repr(transparent)ly Path
    unsafe { &*ptr }
  }

  /// Returns the underlying [`Path`].
  #[must_use]
  pub fn as_path(&self) -> &Path {
    &self.0
  }

  /// Returns the parent of this. If it exists, it will be clean.
  pub fn parent(&self) -> Option<&CleanPath> {
    self.0.parent().map(CleanPath::new_unchecked)
  }

  /// Joins `self` with `other`.
  ///
  /// See [`CleanPathBuf::push`].
  pub fn join<P: AsRef<Path>>(&self, other: P) -> CleanPathBuf {
    let mut ret = self.to_owned();
    ret.push(other);
    ret
  }
}

/// A cleaned path buffer.
///
/// See [`CleanPath`] for discussion of what it means for a path to be "clean".
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CleanPathBuf(PathBuf);

impl std::borrow::Borrow<CleanPath> for CleanPathBuf {
  fn borrow(&self) -> &CleanPath {
    self.as_clean_path()
  }
}

impl std::borrow::Borrow<Path> for CleanPathBuf {
  fn borrow(&self) -> &Path {
    self.as_path()
  }
}

impl CleanPathBuf {
  /// Makes a new `CleanPathBuf`.
  ///
  /// Returns `None` if the path is not absolute.
  #[must_use]
  pub fn new<P: AsRef<Path>>(path: P) -> Option<Self> {
    Self::new_from_path(path.as_ref())
  }

  fn new_from_path(path: &Path) -> Option<Self> {
    path.is_absolute().then(|| Self::new_unchecked(path))
  }

  /// requires the `Path` is already known to be absolute
  ///
  /// largely lifted from cargo
  fn new_unchecked(path: &Path) -> Self {
    debug_assert!(path.is_absolute());

    let mut components = path.components().peekable();
    let mut ret = if let Some(&c @ Component::Prefix(..)) = components.peek() {
      components.next();
      PathBuf::from(c.as_os_str())
    } else {
      PathBuf::new()
    };

    for component in components {
      match component {
        Component::Prefix(..) => unreachable!("prefix can only occur at start"),
        Component::RootDir => ret.push(component.as_os_str()),
        Component::CurDir => {}
        Component::ParentDir => {
          // ignore if we have no parent
          ret.pop();
        }
        Component::Normal(c) => ret.push(c),
      }
    }

    Self(ret)
  }

  /// Returns this as an [`CleanPath`].
  #[must_use]
  pub fn as_clean_path(&self) -> &CleanPath {
    CleanPath::new_unchecked(self.0.as_path())
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

  /// Extends `self` with `path`, while preserving `self`'s cleanliness.
  ///
  /// Note that:
  ///
  /// - If `path` is relative, it will be joined at the end of `self`.
  /// - If path is absolute, it will replace `self`.
  ///
  /// In either case, `self` remains absolute.
  ///
  /// # Examples
  ///
  /// When pushing a relative path:
  ///
  /// ```
  /// # use paths::CleanPathBuf;
  /// use std::path::Path;
  ///
  /// let mut a = CleanPathBuf::new("/foo/bar").unwrap();
  /// let b = Path::new("../quz");
  /// a.push(b);
  /// assert_eq!(a.as_path(), Path::new("/foo/quz"));
  /// ```
  ///
  /// When pushing an absolute path:
  ///
  /// ```
  /// # use paths::CleanPathBuf;
  /// use std::path::Path;
  ///
  /// let mut a = CleanPathBuf::new("/foo/bar").unwrap();
  /// let b = Path::new("/quz/../blob/./glop");
  /// a.push(b);
  /// assert_eq!(a.as_path(), Path::new("/blob/glop"));
  /// ```
  pub fn push<P: AsRef<Path>>(&mut self, path: P) {
    self.0.push(path);
    *self = Self::new_unchecked(self.as_path());
  }
}

/// A file system.
pub trait FileSystem {
  /// Returns the current directory.
  ///
  /// # Errors
  ///
  /// If [`std::env::current_dir`] errored or returned a non-absolute path.
  fn current_dir(&self) -> std::io::Result<CleanPathBuf>;

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
  fn current_dir(&self) -> std::io::Result<CleanPathBuf> {
    match CleanPathBuf::new(std::env::current_dir()?) {
      Some(x) => Ok(x),
      None => Err(std::io::Error::other("path from `std::env::current_dir` was not absolute")),
    }
  }

  fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
  }

  fn read_to_bytes(&self, path: &Path) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
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
/// Doesn't totally handle all `Path`s. But this is mainly intended for basic testing purposes, so
/// it's fine.
#[derive(Debug, Default)]
pub struct MemoryFileSystem {
  /// The in-memory storage.
  pub inner: BTreeMap<CleanPathBuf, String>,
}

impl MemoryFileSystem {
  /// Returns a new `MemoryFileSystem`.
  #[must_use]
  pub fn new(inner: BTreeMap<CleanPathBuf, String>) -> Self {
    Self { inner }
  }

  /// Returns a clean path buf for the root directory, `/`.
  #[must_use]
  pub fn root() -> CleanPathBuf {
    CleanPathBuf(PathBuf::from("/"))
  }
}

impl FileSystem for MemoryFileSystem {
  fn current_dir(&self) -> std::io::Result<CleanPathBuf> {
    Ok(Self::root())
  }

  fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
    match self.inner.get(path) {
      Some(x) => Ok(x.clone()),
      None => Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
    }
  }

  fn read_to_bytes(&self, path: &Path) -> std::io::Result<Vec<u8>> {
    self.read_to_string(path).map(String::into_bytes)
  }

  fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let iter = self.inner.keys().filter_map(|pb| {
      let p = pb.as_path();
      (p.starts_with(path) && p != path).then(|| pb.to_owned().into_path_buf())
    });
    Ok(iter.collect())
  }

  fn is_file(&self, path: &Path) -> bool {
    self.inner.contains_key(path)
  }
}

#[cfg(test)]
fn _obj_safe(_: &dyn FileSystem) {}

#[test]
fn clean_path() {
  let start = Path::new("/foo/bar");
  let back_up = Path::new("../quz/blob");
  let gross = Path::new("/foo/bar/../quz/blob");
  let clean = Path::new("/foo/quz/blob");
  // ew
  assert_eq!(start.join(back_up), gross);
  // ah, that's better
  assert_eq!(CleanPathBuf::new(gross).unwrap().as_path(), clean);
}
