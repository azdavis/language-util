//! Topological sorting.

#[cfg(test)]
mod tests;

pub mod graph;

use always::always;
use std::collections::{BTreeSet, HashSet};
use std::hash::{BuildHasherDefault, Hash, Hasher};

/// The work to do.
#[derive(Debug)]
pub struct Work<T>(Vec<Action<T>>);

impl<T> Work<T> {
  /// Adds an element to be processed.
  pub fn push(&mut self, value: T) {
    self.0.push(Action::start(value));
  }

  /// Runs the sort on the elements with the visitor.
  pub fn run<V>(mut self, visitor: &mut V) -> Ret<V::Set, T>
  where
    T: Clone,
    V: Visitor<Elem = T>,
    V::Set: Set<T>,
  {
    let mut cur = V::Set::default();
    let mut ret = Ret { done: V::Set::default(), cycle: None::<T> };
    // INVARIANT: `level` == how many `End`s are in `work`.
    let mut level = 0usize;
    while let Some(Action(value, kind)) = self.0.pop() {
      match kind {
        ActionKind::Start => {
          if ret.done.contains(&value) {
            continue;
          }
          let Some(data) = visitor.enter(value.clone()) else { continue };
          if !cur.insert(value.clone()) {
            if ret.cycle.is_none() {
              ret.cycle = Some(value);
            }
            continue;
          }
          self.0.push(Action::end(value.clone()));
          level += 1;
          visitor.process(value, data, &mut self);
        }
        ActionKind::End => {
          level = level.checked_sub(1).unwrap_or_else(|| {
            always!(false, "`End` should have a matching `Start`");
            0
          });
          always!(cur.remove(&value), "should only `End` when in `cur`");
          always!(ret.done.insert(value.clone()), "should not `End` if already done");
          visitor.exit(value, level);
        }
      }
    }
    always!(level == 0, "should return to starting level");
    always!(cur.is_empty(), "should have no progress when done");
    ret
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

/// The output of sorting.
#[derive(Debug)]
pub struct Ret<S, T> {
  /// The set of all the elements we visited.
  pub done: S,
  /// If there was a cycle, a participant in the cycle is here.
  pub cycle: Option<T>,
}

/// A visitor when sorting.
pub trait Visitor {
  /// The type of elements we sort.
  type Elem;
  /// Data about an element.
  type Data;
  /// A set of elements.
  type Set;
  /// Begins processing an element by looking up its data. Or, to skip this element, return `None`.
  fn enter(&self, value: Self::Elem) -> Option<Self::Data>;
  /// Processes the element and its data. Can add more things to visit to the work.
  fn process(&mut self, value: Self::Elem, data: Self::Data, work: &mut Work<Self::Elem>);
  /// Finishes processing an element. The number of other things still left to process is given as
  /// `level`.
  fn exit(&mut self, value: Self::Elem, level: usize);
}

/// A set of T.
pub trait Set<T>: Default {
  /// Returns whether the value is in the set.
  fn contains(&self, value: &T) -> bool;
  /// Inserts the value into the set. Returns whether the value was newly inserted.
  fn insert(&mut self, value: T) -> bool;
  /// Removes the value into the set. Returns whether the value was previously in the set.
  fn remove(&mut self, value: &T) -> bool;
  /// Returns whether the set is empty.
  fn is_empty(&self) -> bool;
}

impl<T> Set<T> for BTreeSet<T>
where
  T: Ord,
{
  fn contains(&self, value: &T) -> bool {
    self.contains(value)
  }

  fn insert(&mut self, value: T) -> bool {
    self.insert(value)
  }

  fn remove(&mut self, value: &T) -> bool {
    self.remove(value)
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
  fn contains(&self, value: &T) -> bool {
    self.contains(value)
  }

  fn insert(&mut self, value: T) -> bool {
    self.insert(value)
  }

  fn remove(&mut self, value: &T) -> bool {
    self.remove(value)
  }

  fn is_empty(&self) -> bool {
    self.is_empty()
  }
}

impl<T> Set<T> for HashSet<T, rustc_hash::FxBuildHasher>
where
  T: Hash + Eq,
{
  fn contains(&self, value: &T) -> bool {
    self.contains(value)
  }

  fn insert(&mut self, value: T) -> bool {
    self.insert(value)
  }

  fn remove(&mut self, value: &T) -> bool {
    self.remove(value)
  }

  fn is_empty(&self) -> bool {
    self.is_empty()
  }
}

#[derive(Debug)]
enum ActionKind {
  Start,
  End,
}

#[derive(Debug)]
struct Action<T>(T, ActionKind);

impl<T> Action<T> {
  fn start(value: T) -> Self {
    Self(value, ActionKind::Start)
  }

  fn end(value: T) -> Self {
    Self(value, ActionKind::End)
  }
}
