//! The main algorithm.

use crate::matrix::Matrix;
use crate::types::{Check, CheckError, ConPat, Lang, Pat, RawPat, Result};
use fast_hash::FxHashSet;

/// Does the check.
///
/// Returns an error if the patterns or types passed don't make sense, or if the
/// `lang` returned an error.
///
/// This should never panic. It's a bug if this panics.
pub fn check<L: Lang>(
  lang: &L,
  pats: Vec<Pat<L>>,
  ty: L::Ty,
) -> Result<Check<L>> {
  let mut ac = FxHashSet::default();
  for pat in pats.iter() {
    get_pat_indices(&mut ac, pat);
  }
  let mut mtx = Matrix::<L>::default();
  for pat in pats {
    useful(lang, &mut ac, 0, &mtx, vec![(pat.clone(), ty.clone())])?;
    mtx.push(vec![pat]);
  }
  let missing: Vec<_> =
    useful(lang, &mut ac, 0, &mtx, vec![(Pat::any_no_idx(lang), ty)])?
      .witnesses
      .into_iter()
      .map(|mut w| {
        assert_eq!(w.len(), 1);
        w.pop().expect("just checked length")
      })
      .collect();
  Ok(Check {
    unreachable: ac,
    missing,
  })
}

/// Adds all the pat indices in the Pat to the set.
fn get_pat_indices<L: Lang>(ac: &mut FxHashSet<L::PatIdx>, pat: &Pat<L>) {
  if let Some(idx) = pat.idx {
    ac.insert(idx);
  }
  match &pat.raw {
    RawPat::Con(con_pat) => {
      for pat in &con_pat.args {
        get_pat_indices(ac, pat);
      }
    }
    RawPat::Or(pats) => {
      for pat in pats {
        get_pat_indices(ac, pat);
      }
    }
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

type TypedPatVec<L> = Vec<(Pat<L>, <L as Lang>::Ty)>;

/// Returns whether the pattern stack is useful for this matrix.
fn useful<L: Lang>(
  lang: &L,
  ac: &mut FxHashSet<L::PatIdx>,
  depth: usize,
  mtx: &Matrix<L>,
  mut val: TypedPatVec<L>,
) -> Result<Useful<Pat<L>>> {
  if let Some(nc) = mtx.num_cols() {
    assert_eq!(nc, val.len());
  }
  let (pat, ty) = match val.pop() {
    Some(x) => x,
    None => {
      return Ok(if mtx.num_rows() == 0 {
        Useful::yes()
      } else {
        Useful::no()
      });
    }
  };
  let mut ret = Useful::<Pat<L>>::no();
  let idx = pat.idx;
  match pat.raw {
    RawPat::Or(or_pats) => {
      let mut m = mtx.clone();
      for pat in or_pats {
        let mut val = val.clone();
        val.push((pat, ty.clone()));
        ret.extend(useful(lang, ac, depth + 1, &m, val.clone())?);
        m.push(val.into_iter().map(|(x, _)| x).collect());
      }
    }
    RawPat::Con(con_pat) => {
      let last_col = mtx.non_empty_rows().map(|r| &r.con_pat.con);
      for con in lang.split(&ty, &con_pat.con, last_col, depth)? {
        let mut m = Matrix::<L>::default();
        for row in mtx.non_empty_rows() {
          let new = specialize(lang, &ty, &row.con_pat, &con)?;
          if let Some(new) = new {
            let mut pats = row.pats.clone();
            pats.extend(new.into_iter().map(|(x, _)| x));
            m.push(pats);
          }
        }
        let new = specialize(lang, &ty, &con_pat, &con)?
          .expect("p_con must cover itself");
        let new_len = new.len();
        let mut val = val.clone();
        val.extend(new);
        let mut u = useful(lang, ac, depth + 1, &m, val)?;
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
  Ok(ret)
}

/// Specializes a constructor pat.
///
/// The pat has type `ty` and is specialized with the given other value constructor `con`.
fn specialize<L: Lang>(
  lang: &L,
  ty: &L::Ty,
  pat: &ConPat<L>,
  val_con: &L::Con,
) -> Result<Option<TypedPatVec<L>>> {
  let ret = if lang.covers(&pat.con, &lang.any()) {
    if !pat.args.is_empty() {
      return Err(CheckError);
    }
    let tys = lang.get_arg_tys(ty, val_con)?;
    let ret: Vec<_> = tys
      .into_iter()
      .map(|t| (Pat::any_no_idx(lang), t))
      .rev()
      .collect();
    Some(ret)
  } else if lang.covers(&pat.con, val_con) {
    let tys = lang.get_arg_tys(ty, val_con)?;
    if tys.len() < pat.args.len() {
      return Err(CheckError);
    }
    // the `>` case can happen in the case of e.g. record patterns with missing labels.
    let mut ret: Vec<_> = pat
      .args
      .iter()
      .cloned()
      .chain(std::iter::repeat(Pat::any_no_idx(lang)))
      .zip(tys)
      .collect();
    ret.reverse();
    Some(ret)
  } else {
    None
  };
  Ok(ret)
}
