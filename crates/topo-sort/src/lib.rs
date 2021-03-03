//! Topological sorting.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A graph, represented as a map between nodes and their neighbors.
pub type Graph<T> = BTreeMap<T, BTreeSet<T>>;

/// Returns a reverse topological ordering of the graph, or an error if the
/// graph has a cycle.
pub fn get<T>(graph: &Graph<T>) -> Result<Vec<T>, CycleError<T>>
where
  T: Copy + Eq + Ord,
{
  let mut active = BTreeSet::new();
  let mut done = BTreeSet::new();
  let mut ret = Vec::with_capacity(graph.len());
  let mut stack: Vec<_> = graph.keys().map(|&x| (Action::Start, x)).collect();
  while let Some((ac, cur)) = stack.pop() {
    match ac {
      Action::Start => {
        if done.contains(&cur) {
          continue;
        }
        if !active.insert(cur) {
          return Err(CycleError(cur));
        }
        stack.push((Action::Finish, cur));
        if let Some(ns) = graph.get(&cur) {
          stack.extend(ns.iter().map(|&x| (Action::Start, x)));
        }
      }
      Action::Finish => {
        assert!(active.remove(&cur));
        assert!(done.insert(cur));
        ret.push(cur);
      }
    }
  }
  Ok(ret)
}

/// An error when the graph contained a cycle.
#[derive(Debug)]
pub struct CycleError<T>(T);

impl<T> CycleError<T> {
  /// Returns one of the `T` involved in the cycle.
  pub fn witness(self) -> T {
    self.0
  }
}

impl<T> fmt::Display for CycleError<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "graph has a cycle")
  }
}

impl<T> std::error::Error for CycleError<T> where T: std::fmt::Debug {}

enum Action {
  Start,
  Finish,
}
