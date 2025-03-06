//! See [`Matrix`].

use crate::types::{ConPat, Lang, Pat, RawPat};
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
    Self { rows: self.rows.clone() }
  }
}

impl<L: Lang> fmt::Debug for Matrix<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Matrix").field("rows", &self.rows).finish()
  }
}

impl<L> fmt::Display for Matrix<L>
where
  L: Lang,
  L::Con: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut first = true;
    for row in &self.rows {
      if !first {
        f.write_str("\n")?;
        first = false;
      }
      f.write_str("<")?;
      match row {
        Row::Empty => {}
        Row::NonEmpty(row) => {
          for pat in &row.pats {
            write!(f, "{pat}, ")?;
          }
          fmt::Display::fmt(&row.con_pat, f)?;
        }
      }
      f.write_str(">")?;
    }
    Ok(())
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
        for con_pat in con_pats {
          self.rows.push(Row::NonEmpty(NonEmptyRow { pats: row.clone(), con_pat }));
        }
      }
    }
  }
}

/// Recursively expands or patterns.
fn expand_or<L: Lang>(ac: &mut Vec<ConPat<L>>, pat: Pat<L>) {
  match pat.raw {
    RawPat::Con(p) => ac.push(p),
    RawPat::Or(pats) => {
      for pat in pats {
        expand_or(ac, pat);
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
  /// The last pattern.
  pub con_pat: ConPat<L>,
}

impl<L: Lang> Clone for NonEmptyRow<L> {
  fn clone(&self) -> Self {
    Self { pats: self.pats.clone(), con_pat: self.con_pat.clone() }
  }
}

impl<L: Lang> fmt::Debug for NonEmptyRow<L> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("NonEmptyRow").field("pats", &self.pats).field("con_pat", &self.con_pat).finish()
  }
}
