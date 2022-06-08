//! The main algorithm.

use crate::matrix::Matrix;
use crate::types::{Check, Lang, Pat, RawPat};
use rustc_hash::FxHashSet;

/// Does the check.
pub fn check<L: Lang>(lang: &L, pats: Vec<Pat<L>>, ty: L::Ty) -> Check<L> {
  let mut ac = FxHashSet::default();
  for pat in pats.iter() {
    get_pat_indices(&mut ac, pat);
  }
  let mut matrix = Matrix::default();
  for pat in pats {
    useful(lang, &mut ac, &matrix, vec![(pat.clone(), ty.clone())]);
    matrix.push(vec![pat]);
  }
  let missing: Vec<_> =
    useful(lang, &mut ac, &matrix, vec![(Pat::any_no_idx(lang), ty)])
      .witnesses
      .into_iter()
      .map(|mut w| {
        assert_eq!(w.len(), 1);
        w.pop().expect("just checked length")
      })
      .collect();
  Check {
    unreachable: ac,
    missing,
  }
}

struct Useful<P> {
  /// invariant: no Pat will be Or
  witnesses: Vec<Vec<P>>,
}

impl<P> Useful<P> {
  fn yes() -> Self {
    Self {
      witnesses: vec![vec![]],
    }
  }

  fn no() -> Self {
    Self { witnesses: vec![] }
  }

  fn extend(&mut self, other: Self) {
    self.witnesses.extend(other.witnesses);
  }
}

/// Returns whether the pattern stack is useful for this matrix.
fn useful<L: Lang>(
  lang: &L,
  ac: &mut FxHashSet<L::PatIdx>,
  matrix: &Matrix<L>,
  mut val: Vec<(Pat<L>, L::Ty)>,
) -> Useful<Pat<L>> {
  if let Some(nc) = matrix.num_cols() {
    assert_eq!(nc, val.len());
  }
  if val.is_empty() {
    return if matrix.num_rows() == 0 {
      Useful::yes()
    } else {
      Useful::no()
    };
  }
  let (pat, ty) = val.pop().unwrap();
  let mut ret = Useful::no();
  let idx = pat.idx;
  match pat.raw {
    RawPat::Or(or_pats) => {
      let mut matrix = matrix.clone();
      for pat in or_pats {
        let mut val = val.clone();
        val.push((pat, ty.clone()));
        ret.extend(useful(lang, ac, &matrix, val.clone()));
        matrix.push(val.into_iter().map(|x| x.0).collect());
      }
    }
    RawPat::Con(p_con, p_args) => {
      let last_col = matrix.non_empty_rows().map(|r| &r.con);
      for con in lang.split(&ty, &p_con, last_col) {
        let mut m = Matrix::default();
        for row in matrix.non_empty_rows() {
          let new = specialize(lang, &ty, &row.con, &row.args, &con);
          if let Some(new) = new {
            let mut pats = row.pats.clone();
            pats.extend(new.into_iter().map(|x| x.0));
            m.push(pats);
          }
        }
        let new = specialize(lang, &ty, &p_con, &p_args, &con)
          .expect("p_con must cover itself");
        let new_len = new.len();
        let mut val = val.clone();
        val.extend(new);
        let mut u = useful(lang, ac, &m, val);
        for w in u.witnesses.iter_mut() {
          let args: Vec<_> = w.drain(w.len() - new_len..).rev().collect();
          w.push(Pat::con_(con.clone(), args, idx));
        }
        ret.extend(u);
      }
    }
  }
  if let Some(idx) = idx {
    if !ret.witnesses.is_empty() {
      ac.remove(&idx);
    }
  }
  ret
}

/// Specializes a constructor pat.
///
/// The pat
/// - has type `ty`;
/// - has been broken into its constituent constructor `pat_con` and arguments
/// `pat_args`;
/// - is specialized with the given other value constructor `val_con`.
fn specialize<L: Lang>(
  lang: &L,
  ty: &L::Ty,
  pat_con: &L::Con,
  pat_args: &[Pat<L>],
  val_con: &L::Con,
) -> Option<Vec<(Pat<L>, L::Ty)>> {
  let ret: Vec<_> = if *pat_con == lang.any() {
    assert!(pat_args.is_empty());
    let tys = lang.get_arg_tys(ty, val_con);
    tys
      .into_iter()
      .map(|t| (Pat::any_no_idx(lang), t))
      .rev()
      .collect()
  } else if val_con == pat_con {
    let tys = lang.get_arg_tys(ty, val_con);
    assert_eq!(tys.len(), pat_args.len());
    pat_args.iter().cloned().zip(tys).rev().collect()
  } else {
    return None;
  };
  Some(ret)
}

/// Adds all the pat indices in the Pat to the set.
fn get_pat_indices<L: Lang>(ac: &mut FxHashSet<L::PatIdx>, pat: &Pat<L>) {
  if let Some(idx) = pat.idx {
    ac.insert(idx);
  }
  match &pat.raw {
    RawPat::Con(_, pats) | RawPat::Or(pats) => {
      for pat in pats {
        get_pat_indices(ac, pat);
      }
    }
  }
}
