//! Topological sorting.

#![allow(missing_docs)]

#[cfg(test)]
mod tests;

pub mod graph;

use always::always;
use std::collections::{BTreeSet, HashSet};
use std::hash::{BuildHasherDefault, Hash, Hasher};

#[derive(Debug)]
enum ActionKind {
  Start,
  End,
}

#[derive(Debug)]
struct Action<T>(T, ActionKind);

impl<T> Action<T> {
  const fn start(value: T) -> Self {
    Self(value, ActionKind::Start)
  }

  const fn end(value: T) -> Self {
    Self(value, ActionKind::End)
  }
}

#[derive(Debug)]
pub struct Work<T>(Vec<Action<T>>);

impl<T> Work<T> {
  pub fn push(&mut self, value: T) {
    self.0.push(Action::start(value));
  }
}

impl<T> Default for Work<T> {
  fn default() -> Self {
    Self(Vec::new())
  }
}

impl<T> Extend<T> for Work<T> {
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    for x in iter {
      self.push(x);
    }
  }
}

impl<T> FromIterator<T> for Work<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut ret = Work::default();
    ret.extend(iter);
    ret
  }
}

pub trait Visitor {
  type Elem: Copy;
  type Data;
  type Set: Set<Self::Elem>;
  fn enter(&self, value: Self::Elem) -> Option<Self::Data>;
  fn process(&mut self, value: Self::Elem, data: Self::Data, work: &mut Work<Self::Elem>);
  fn exit(&mut self, value: Self::Elem, level_idx: usize);
}

pub trait Set<T> {
  fn new() -> Self;
  fn contains(&self, value: T) -> bool;
  fn insert(&mut self, value: T) -> bool;
  fn remove(&mut self, value: T) -> bool;
  fn is_empty(&self) -> bool;
}

impl<T> Set<T> for BTreeSet<T>
where
  T: Ord,
{
  fn new() -> Self {
    BTreeSet::new()
  }

  fn contains(&self, value: T) -> bool {
    self.contains(&value)
  }

  fn insert(&mut self, value: T) -> bool {
    self.insert(value)
  }

  fn remove(&mut self, value: T) -> bool {
    self.remove(&value)
  }

  fn is_empty(&self) -> bool {
    self.is_empty()
  }
}

impl<T, S> Set<T> for HashSet<T, BuildHasherDefault<S>>
where
  T: Hash + Eq,
  S: Hasher + Default,
{
  fn new() -> Self {
    HashSet::default()
  }

  fn contains(&self, value: T) -> bool {
    self.contains(&value)
  }

  fn insert(&mut self, value: T) -> bool {
    self.insert(value)
  }

  fn remove(&mut self, value: T) -> bool {
    self.remove(&value)
  }

  fn is_empty(&self) -> bool {
    self.is_empty()
  }
}

impl<T> Set<T> for HashSet<T, rustc_hash::FxBuildHasher>
where
  T: Hash + Eq,
{
  fn new() -> Self {
    HashSet::default()
  }

  fn contains(&self, value: T) -> bool {
    self.contains(&value)
  }

  fn insert(&mut self, value: T) -> bool {
    self.insert(value)
  }

  fn remove(&mut self, value: T) -> bool {
    self.remove(&value)
  }

  fn is_empty(&self) -> bool {
    self.is_empty()
  }
}

pub fn run<V>(visitor: &mut V, mut work: Work<V::Elem>) -> TopoSort<V::Set, V::Elem>
where
  V: Visitor,
{
  let mut cur = V::Set::new();
  let mut done = V::Set::new();
  // INVARIANT: `level_idx` == how many `End`s are in `work`.
  let mut level_idx = 0usize;
  let mut cycle = None::<V::Elem>;
  while let Some(Action(value, kind)) = work.0.pop() {
    match kind {
      ActionKind::Start => {
        if done.contains(value) {
          continue;
        }
        let Some(data) = visitor.enter(value) else { continue };
        if !cur.insert(value) {
          if cycle.is_none() {
            cycle = Some(value);
          }
          continue;
        }
        work.0.push(Action::end(value));
        level_idx += 1;
        visitor.process(value, data, &mut work);
      }
      ActionKind::End => {
        level_idx = match level_idx.checked_sub(1) {
          None => {
            always!(false, "`End` should have a matching `Start`");
            continue;
          }
          Some(x) => x,
        };
        always!(cur.remove(value), "should only `End` when in `cur`");
        always!(done.insert(value), "should not `End` if already done");
        visitor.exit(value, level_idx);
      }
    }
  }
  always!(level_idx == 0, "should return to starting level");
  always!(cur.is_empty(), "should have no progress when done");
  TopoSort { done, cycle }
}

#[derive(Debug)]
pub struct TopoSort<S, T> {
  pub done: S,
  pub cycle: Option<T>,
}
