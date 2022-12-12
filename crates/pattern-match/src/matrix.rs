//! See [`Matrix`].

use crate::types::{Lang, Pat, RawPat};
use std::fmt;

/// A 2-D matrix of [`Pat`]s.
pub(crate) struct Matrix<L: Lang> {
  /// invariant: all rows are the same length.
  rows: Vec<Row<L>>,
}

impl<L: Lang> Default for Matrix<L> {
  fn default() -> Self {
    Self { rows: Vec::new() }
  }
}

impl<L: Lang> Clone for Matrix<L> {
  fn clone(&self) -> Self {
    Self {
      rows: self.rows.clone(),
    }
  }
}

impl<L: Lang> fmt::Debug for Matrix<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Matrix").field("rows", &self.rows).finish()
  }
}

impl<L: Lang> Matrix<L> {
  /// Returns the number of rows.
  pub(crate) fn num_rows(&self) -> usize {
    self.rows.len()
  }

  /// Returns the number of columns, or `None` if there are no rows.
  pub(crate) fn num_cols(&self) -> Option<usize> {
    self.rows.first().map(Row::len)
  }

  /// Returns an iterator over the non-empty rows. Panics if the rows are empty.
  pub(crate) fn non_empty_rows(&self) -> impl Iterator<Item = &NonEmptyRow<L>> {
    self.rows.iter().map(|r| match r {
      Row::Empty => panic!("empty row"),
      Row::NonEmpty(r) => r,
    })
  }

  /// Adds a row to the bottom of the matrix.
  ///
  /// If the row ends with a [`Pat::Or`], the row will be expanded into many
  /// rows.
  ///
  /// Panics if `row.len()` is not equal to the number of columns in this
  /// matrix.
  pub(crate) fn push(&mut self, mut row: Vec<Pat<L>>) {
    if let Some(nc) = self.num_cols() {
      assert_eq!(nc, row.len());
    }
    match row.pop() {
      None => self.rows.push(Row::Empty),
      Some(pat) => {
        let mut con_pats = Vec::new();
        expand_or(&mut con_pats, pat);
        for (con, args) in con_pats {
          self.rows.push(Row::NonEmpty(NonEmptyRow {
            pats: row.clone(),
            con,
            args,
          }));
        }
      }
    }
  }
}

/// A matrix row.
enum Row<L: Lang> {
  Empty,
  NonEmpty(NonEmptyRow<L>),
}

impl<L: Lang> Clone for Row<L> {
  fn clone(&self) -> Self {
    match self {
      Self::Empty => Self::Empty,
      Self::NonEmpty(r) => Self::NonEmpty(r.clone()),
    }
  }
}

impl<L: Lang> fmt::Debug for Row<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Row::Empty => f.write_str("Empty"),
      Row::NonEmpty(row) => f.debug_tuple("NonEmpty").field(&row).finish(),
    }
  }
}

impl<L: Lang> Row<L> {
  fn len(&self) -> usize {
    match self {
      Row::Empty => 0,
      Row::NonEmpty(r) => r.pats.len() + 1,
    }
  }
}

/// An non-empty row, whose last element is a non-or pattern with the given
/// constructor and arguments.
pub(crate) struct NonEmptyRow<L: Lang> {
  /// The other patterns in this row.
  pub pats: Vec<Pat<L>>,
  /// The constructor of the last pattern, which is not an or-pattern.
  pub con: L::Con,
  /// The arguments of the last pattern, which is not an or-pattern.
  pub args: Vec<Pat<L>>,
}

impl<L: Lang> Clone for NonEmptyRow<L> {
  fn clone(&self) -> Self {
    Self {
      pats: self.pats.clone(),
      con: self.con.clone(),
      args: self.args.clone(),
    }
  }
}

impl<L: Lang> fmt::Debug for NonEmptyRow<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("NonEmptyRow")
      .field("pats", &self.pats)
      .field("con", &self.con)
      .field("args", &self.args)
      .finish()
  }
}

/// Recursively expands or patterns.
fn expand_or<L: Lang>(ac: &mut Vec<(L::Con, Vec<Pat<L>>)>, pat: Pat<L>) {
  match pat.raw {
    RawPat::Con(con, args) => ac.push((con, args)),
    RawPat::Or(pats) => {
      for pat in pats {
        expand_or(ac, pat);
      }
    }
  }
}
