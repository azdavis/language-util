use crate::{get, CycleError, Graph};
use std::collections::{BTreeMap, BTreeSet};

fn check(graph: &Graph<u32>, order: &[u32]) {
  assert_eq!(get(graph).unwrap(), order);
}

fn check_cycle(graph: &Graph<u32>) {
  let err = get(graph).unwrap_err();
  assert!(matches!(err, CycleError(_)));
}

#[test]
fn empty() {
  check(&BTreeMap::new(), &[]);
}

#[test]
fn one() {
  let graph = BTreeMap::from([(1, BTreeSet::new())]);
  check(&graph, &[1]);
}

#[test]
fn separate() {
  let graph = BTreeMap::from([(1, BTreeSet::new()), (2, BTreeSet::new())]);
  check(&graph, &[2, 1]);
}

#[test]
fn simple() {
  let graph = BTreeMap::from([(1, BTreeSet::from([2])), (2, BTreeSet::new())]);
  check(&graph, &[2, 1]);
}

#[test]
fn bigger() {
  let graph = BTreeMap::from([
    (1, BTreeSet::from([4])),
    (2, BTreeSet::from([1, 7])),
    (3, BTreeSet::from([4, 6, 8])),
    (4, BTreeSet::from([5])),
    (5, BTreeSet::from([6, 8])),
    (6, BTreeSet::new()),
    (7, BTreeSet::from([3, 8, 9])),
    (8, BTreeSet::from([9])),
    (9, BTreeSet::new()),
  ]);
  check(&graph, &[9, 8, 6, 5, 4, 3, 7, 1, 2]);
}

#[test]
fn small_cycle() {
  let graph = BTreeMap::from([(2, BTreeSet::from([1])), (1, BTreeSet::from([2]))]);
  check_cycle(&graph);
}

#[test]
fn bigger_cycle() {
  let graph = BTreeMap::from([
    (1, BTreeSet::from([2])),
    (2, BTreeSet::new()),
    (3, BTreeSet::from([6])),
    (4, BTreeSet::from([5])),
    (5, BTreeSet::from([3, 2])),
    (6, BTreeSet::from([1, 4])),
  ]);
  check_cycle(&graph);
}

#[test]
fn small_cycle_with_extra() {
  let graph =
    BTreeMap::from([(1, BTreeSet::from([2])), (2, BTreeSet::from([1])), (3, BTreeSet::from([1]))]);
  check_cycle(&graph);
}
