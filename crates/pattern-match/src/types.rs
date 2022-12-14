//! See [`Pat`].

use rustc_hash::FxHashSet;
use std::fmt::{self, Debug};
use std::hash::Hash;

/// std's Result with our [`CheckError`].
pub type Result<T, E = CheckError> = std::result::Result<T, E>;

/// An error that occurred while checking.
#[derive(Debug)]
pub struct CheckError;

impl fmt::Display for CheckError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("could not check pattern match")
  }
}

impl std::error::Error for CheckError {}

/// The result of checking.
pub struct Check<L: Lang> {
  /// The indices of unreachable patterns.
  pub unreachable: FxHashSet<L::PatIdx>,
  /// Some patterns that weren't covered by the match.
  pub missing: Vec<Pat<L>>,
}

impl<L: Lang> fmt::Debug for Check<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Check")
      .field("unreachable", &self.unreachable)
      .field("missing", &self.missing)
      .finish()
  }
}

/// The language we do pattern matching on.
pub trait Lang {
  /// A pattern identifier.
  type PatIdx: Debug + Copy + Eq + Hash;

  /// A constructor.
  type Con: Debug + Clone;

  /// A type.
  type Ty: Debug + Clone;

  /// Returns a constructor for 'anything', like a wildcard or variable pattern.
  ///
  /// An `any` pattern should have no arguments.
  fn any(&self) -> Self::Con;

  /// Splits a constructor with the given type into 'real' constructors.
  ///
  /// `cons` are the constructors that are already somewhat covered.
  fn split<'a, I>(
    &self,
    ty: &Self::Ty,
    con: &Self::Con,
    cons: I,
  ) -> Result<Vec<Self::Con>>
  where
    Self::Con: 'a,
    I: Iterator<Item = &'a Self::Con>;

  /// Returns the types of the arguments to a constructor pattern with the given
  /// type `ty` and constructor `con`.
  fn get_arg_tys(
    &self,
    ty: &Self::Ty,
    con: &Self::Con,
  ) -> Result<Vec<Self::Ty>>;

  /// Returns whether `lhs` covers `rhs`. Sometimes this is as simple as returning `lhs == rhs`.
  fn covers(&self, lhs: &Self::Con, rhs: &Self::Con) -> bool;
}

/// A pattern.
pub struct Pat<L: Lang> {
  /// The raw pattern.
  pub raw: RawPat<L>,
  /// The index.
  pub idx: Option<L::PatIdx>,
}

impl<L: Lang> fmt::Debug for Pat<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Pat")
      .field("raw", &self.raw)
      .field("idx", &self.idx)
      .finish()
  }
}

impl<L: Lang> Clone for Pat<L> {
  fn clone(&self) -> Self {
    Self {
      raw: self.raw.clone(),
      idx: self.idx,
    }
  }
}

impl<L: Lang> Pat<L> {
  /// Returns an `any` pattern with no `PatIdx`.
  pub fn any_no_idx(lang: &L) -> Self {
    Self {
      raw: RawPat::Con(ConPat {
        con: lang.any(),
        args: Vec::new(),
      }),
      idx: None,
    }
  }

  /// Returns a constructor pattern.
  pub fn con(con: L::Con, args: Vec<Self>, idx: L::PatIdx) -> Self {
    Self::con_(con, args, Some(idx))
  }

  /// Returns a constructor with no arguments.
  pub fn zero(con: L::Con, idx: L::PatIdx) -> Self {
    Self::con_(con, Vec::new(), Some(idx))
  }

  pub(crate) fn con_(
    con: L::Con,
    args: Vec<Self>,
    idx: Option<L::PatIdx>,
  ) -> Self {
    Self {
      raw: RawPat::Con(ConPat { con, args }),
      idx,
    }
  }

  /// Returns an or pattern.
  pub fn or(pats: Vec<Self>, idx: L::PatIdx) -> Self {
    Self {
      raw: RawPat::Or(pats),
      idx: Some(idx),
    }
  }
}

/// A raw pattern.
pub enum RawPat<L: Lang> {
  /// A constructor pattern.
  Con(ConPat<L>),
  /// An or pattern.
  Or(Vec<Pat<L>>),
}

impl<L: Lang> fmt::Debug for RawPat<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      RawPat::Con(c) => f.debug_tuple("Con").field(c).finish(),
      RawPat::Or(ps) => f.debug_tuple("Or").field(ps).finish(),
    }
  }
}

impl<L: Lang> Clone for RawPat<L> {
  fn clone(&self) -> Self {
    match self {
      RawPat::Con(c) => RawPat::Con(c.clone()),
      RawPat::Or(ps) => RawPat::Or(ps.clone()),
    }
  }
}

/// A constructor pattern.
pub struct ConPat<L: Lang> {
  /// The constructor.
  pub con: L::Con,
  /// The arguments.
  pub args: Vec<Pat<L>>,
}

impl<L: Lang> fmt::Debug for ConPat<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ConPat")
      .field("con", &self.con)
      .field("args", &self.args)
      .finish()
  }
}

impl<L: Lang> Clone for ConPat<L> {
  fn clone(&self) -> Self {
    Self {
      con: self.con.clone(),
      args: self.args.clone(),
    }
  }
}
