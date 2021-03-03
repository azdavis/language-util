use crate::{get, CycleError, Graph};
use maplit::{btreemap, btreeset};

fn check(graph: Graph<u32>, order: &[u32]) {
  assert_eq!(get(&graph).unwrap(), order);
}

fn check_cycle(graph: Graph<u32>) {
  let err = get(&graph).unwrap_err();
  assert!(matches!(err, CycleError(_)));
}

#[test]
fn empty() {
  check(btreemap![], &[]);
}

#[test]
fn one() {
  let graph = btreemap![
    1 => btreeset![],
  ];
  check(graph, &[1]);
}

#[test]
fn separate() {
  let graph = btreemap![
    1 => btreeset![],
    2 => btreeset![],
  ];
  check(graph, &[2, 1]);
}

#[test]
fn simple() {
  let graph = btreemap![
    1 => btreeset![2],
    2 => btreeset![],
  ];
  check(graph, &[2, 1]);
}

#[test]
fn cycle() {
  let graph = btreemap![
    2 => btreeset![1],
    1 => btreeset![2],
  ];
  check_cycle(graph);
}

#[test]
fn bigger() {
  let graph = btreemap![
    1 => btreeset![4],
    2 => btreeset![1, 7],
    3 => btreeset![4, 6, 8],
    4 => btreeset![5],
    5 => btreeset![6, 8],
    6 => btreeset![],
    7 => btreeset![3, 8, 9],
    8 => btreeset![9],
    9 => btreeset![],
  ];
  check(graph, &[9, 8, 6, 5, 4, 3, 7, 1, 2]);
}

#[test]
fn bigger_cycle() {
  let graph = btreemap![
    1 => btreeset![2],
    2 => btreeset![],
    3 => btreeset![6],
    4 => btreeset![5],
    5 => btreeset![3, 2],
    6 => btreeset![1, 4],
  ];
  check_cycle(graph);
}

#[test]
fn hm_cycle() {
  let graph = btreemap![
    1 => btreeset![2],
    2 => btreeset![1],
    3 => btreeset![1],
  ];
  check_cycle(graph);
}
