//! Topological sorting to make a graph.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A graph, represented as a map between nodes and their neighbors.
pub type Graph<T> = BTreeMap<T, BTreeSet<T>>;

/// Returns a reverse topological ordering of the graph.
///
/// # Errors
///
/// If the graph has a cycle.
///
/// # Panics
///
/// On internal error.
pub fn get<T>(graph: &Graph<T>) -> Result<Vec<T>, CycleError<T>>
where
  T: Copy + Eq + Ord,
{
  let work: crate::Work<_> = graph.keys().copied().collect();
  let mut visitor = Visitor { graph, order: Vec::new() };
  let got = crate::run(&mut visitor, work);
  match got.cycle {
    Some(x) => Err(CycleError(x)),
    None => Ok(visitor.order),
  }
}

struct Visitor<'a, T> {
  graph: &'a Graph<T>,
  order: Vec<T>,
}

impl<T> crate::Visitor for Visitor<'_, T>
where
  T: Copy + Eq + Ord,
{
  type Elem = T;

  type Data = ();

  type Set = BTreeSet<T>;

  fn enter(&self, _: Self::Elem) -> Option<Self::Data> {
    Some(())
  }

  fn process(&mut self, value: Self::Elem, (): Self::Data, work: &mut crate::Work<Self::Elem>) {
    if let Some(ns) = self.graph.get(&value) {
      work.extend(ns.iter().copied());
    }
  }

  fn exit(&mut self, value: Self::Elem, _: usize) {
    self.order.push(value);
  }
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
