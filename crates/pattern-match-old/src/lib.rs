//! Determine whether a sequence of patterns is completely and non-redundantly
//! exhaustive.
//!
//! Adapted from "ML pattern match compilation and partial evaluation" by Peter
//! Sestoft.

// One who both has read his paper and this implementation will notice some
// differences between the two:
//
// - We don't compute an explicit decision tree because we're not compiling
//   anything.
// - We don't record access information because of the same.
// - We do keep track of matched patterns because we want to report which
//   pattern(s) are unreachable.
// - We switch around the order of some work lists (vectors) for efficiency.
// - We reorganize the types used to encode invariants. For instance, instead of
//   having two lists which ought to be the same length, have a single list of
//   structs with two fields.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]

use rustc_hash::FxHashSet;
use std::hash::Hash;

/// A pattern.
#[derive(Debug, Clone)]
pub enum Pat<C> {
  /// Matches anything.
  Any,
  /// Matches a constructor with the given arguments.
  Con(C, Vec<Pat<C>>),
}

impl<C> From<C> for Pat<C> {
  fn from(c: C) -> Self {
    Self::Con(c, Vec::new())
  }
}

/// A constructor.
pub trait Con: Clone + Eq + Hash {
  /// Returns the span of this constructor.
  fn span(&self) -> Span;
}

/// A measure of how many constructors exist for a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
  /// A finite number.
  Finite(usize),
  /// Positive infinity.
  Infinity,
}

/// A description of an object being matched (the "match head"). We use this to
/// cumulatively record information about the match head as we see more
/// patterns.
#[derive(Clone)]
enum Desc<C> {
  /// We know that the match head is this Con, and the arguments to that Con are
  /// described.
  Pos(C, Vec<Desc<C>>),
  /// We know that the match head is not any of these Con.
  Neg(FxHashSet<C>),
}

impl<C> Default for Desc<C> {
  fn default() -> Self {
    Self::Neg(FxHashSet::default())
  }
}

/// The return from `static_match`.
enum StaticMatch<C> {
  /// The Con is consistent with the Desc.
  Yes,
  /// The Con is not consistent with the Desc.
  No,
  /// The Con might be consistent with the Desc. If this is returned, then the
  /// Desc was Neg.
  Maybe(FxHashSet<C>),
}

/// An item in the work list.
#[derive(Clone)]
struct WorkItem<C> {
  /// The constructor.
  con: C,
  /// Descriptions about the processed arguments to the constructor. These are
  /// in the actual order of the arguments, so the first one is the leftmost
  /// argument in the source.
  descs: Vec<Desc<C>>,
  /// The un-processed arguments in reverse order. These are backwards, so the
  /// first one is the rightmost argument in the source, and the last one is the
  /// next one to be processed.
  args: Vec<Arg<C>>,
}

/// An unprocessed argument.
#[derive(Clone)]
struct Arg<C> {
  /// The pattern.
  pat: Pat<C>,
  /// A description about the pattern, possibly computed by analyzing previous
  /// patterns in the match.
  desc: Desc<C>,
}

/// The work list. The back of the list is the next item to be processed (it's a
/// stack).
type Work<C> = Vec<WorkItem<C>>;

/// When we determine a pattern is reachable, we set its corresponding index to
/// true.
type Reachable = Vec<bool>;

/// The patterns being processed.
type Pats<'a, C> = std::iter::Enumerate<std::slice::Iter<'a, Pat<C>>>;

/// A determination of what the patterns were.
#[derive(Debug)]
pub enum Res {
  /// The patterns were exhaustive.
  Exhaustive,
  /// The patterns were not exhaustive.
  NonExhaustive,
  /// There was a pattern, at the given index, which can never be reached.
  Unreachable(usize),
}

/// Does the check.
///
/// Patterns are matched in order from first to last.
pub fn check<C: Con>(pats: &[Pat<C>]) -> Res {
  let mut r = vec![false; pats.len()];
  if fail(&mut r, Desc::default(), pats.iter().enumerate()) {
    match r.iter().position(|&x| !x) {
      None => Res::Exhaustive,
      Some(idx) => Res::Unreachable(idx),
    }
  } else {
    Res::NonExhaustive
  }
}

/// Augment the last element in the work list with a new Desc.
fn augment<C>(mut work: Work<C>, desc: Desc<C>) -> Work<C> {
  if let Some(item) = work.last_mut() {
    item.descs.push(desc);
  }
  work
}

/// Builds a `Desc` from a base `Desc` and a work list.
fn build_desc<C>(mut desc: Desc<C>, work: Work<C>) -> Desc<C> {
  // Since we take the from the end of `work`, reverse the iterator.
  for item in work.into_iter().rev() {
    // First the computed descriptions.
    let mut descs = item.descs;
    // Then this description.
    descs.push(desc);
    // Then the argument descriptions. We reverse because these are stored in
    // reverse, so reversing again will straighten it out.
    descs.extend(item.args.into_iter().rev().map(|x| x.desc));
    desc = Desc::Pos(item.con, descs)
  }
  desc
}

/// Statically match a `Con` against a `Desc`.
fn static_match<C: Con>(con: C, desc: &Desc<C>) -> StaticMatch<C> {
  match desc {
    Desc::Pos(c, _) => {
      if *c == con {
        StaticMatch::Yes
      } else {
        StaticMatch::No
      }
    }
    Desc::Neg(cons) => {
      if cons.contains(&con) {
        StaticMatch::No
      } else if con.span() == Span::Finite(cons.len() + 1) {
        // This is the last con.
        StaticMatch::Yes
      } else {
        StaticMatch::Maybe(cons.clone())
      }
    }
  }
}

/// Tries to pass the next pattern in `pats` to a fresh call to `do_match`.
/// Returns whether the match was exhaustive.
fn fail<C: Con>(
  r: &mut Reachable,
  desc: Desc<C>,
  mut pats: Pats<'_, C>,
) -> bool {
  match pats.next() {
    None => false,
    Some((idx, pat)) => do_match(r, idx, pat.clone(), desc, Vec::new(), pats),
  }
}

/// Tries to prove a pat located at the index is reachable. Sets that index to
/// true if it can prove this. Returns whether the match was exhaustive.
fn succeed<C: Con>(
  r: &mut Reachable,
  idx: usize,
  mut work: Work<C>,
  pats: Pats<'_, C>,
) -> bool {
  loop {
    match work.pop() {
      None => {
        r[idx] = true;
        return true;
      }
      Some(mut item) => match item.args.pop() {
        None => work = augment(work, Desc::Pos(item.con, item.descs)),
        Some(arg) => {
          work.push(item);
          return do_match(r, idx, arg.pat, arg.desc, work, pats);
        }
      },
    }
  }
}

/// Updates the work list with new work for the pattern at the index, then
/// continues on to `succeed`. Returns whether the match was exhaustive.
fn succeed_with<C: Con>(
  r: &mut Reachable,
  idx: usize,
  mut work: Work<C>,
  con: C,
  arg_pats: Vec<Pat<C>>,
  desc: Desc<C>,
  pats: Pats<'_, C>,
) -> bool {
  let arg_descs = match desc {
    Desc::Neg(_) => arg_pats.iter().map(|_| Desc::default()).collect(),
    Desc::Pos(_, descs) => descs,
  };
  assert_eq!(arg_pats.len(), arg_descs.len());
  work.push(WorkItem {
    con,
    descs: Vec::new(),
    args: arg_pats
      .into_iter()
      .zip(arg_descs)
      .rev()
      .map(|(pat, desc)| Arg { pat, desc })
      .collect(),
  });
  succeed(r, idx, work, pats)
}

/// Tries to match the `Pat` against the `Desc` using the other helpers. Returns
/// whether the match was exhaustive.
fn do_match<C: Con>(
  r: &mut Reachable,
  idx: usize,
  pat: Pat<C>,
  desc: Desc<C>,
  work: Work<C>,
  pats: Pats<'_, C>,
) -> bool {
  match pat {
    Pat::Any => succeed(r, idx, augment(work, desc), pats),
    Pat::Con(con, args) => match static_match(con.clone(), &desc) {
      StaticMatch::Yes => succeed_with(r, idx, work, con, args, desc, pats),
      StaticMatch::No => fail(r, build_desc(desc, work), pats),
      StaticMatch::Maybe(mut cons) => {
        cons.insert(con.clone());
        succeed_with(r, idx, work.clone(), con, args, desc, pats.clone())
          && fail(r, build_desc(Desc::Neg(cons), work), pats)
      }
    },
  }
}
