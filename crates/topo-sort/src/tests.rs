use crate::graph::{get, Graph};

fn check(graph: &Graph<u32>, order: &[u32]) {
  assert_eq!(get(graph).unwrap(), order);
}

fn check_cycle(graph: &Graph<u32>) {
  let err = get(graph).unwrap_err();
  let _ = err.witness();
}

fn mk_graph(g: &[(u32, &[u32])]) -> Graph<u32> {
  g.iter().map(|&(node, ns)| (node, ns.iter().copied().collect())).collect()
}

#[test]
fn empty() {
  let graph = mk_graph(&[]);
  check(&graph, &[]);
}

#[test]
fn one() {
  let graph = mk_graph(&[(1, &[])]);
  check(&graph, &[1]);
}

#[test]
fn separate() {
  let graph = mk_graph(&[
    // comment prevents rustfmt from doing all one line
    (1, &[]),
    (2, &[]),
  ]);
  check(&graph, &[2, 1]);
}

#[test]
fn simple() {
  let graph = mk_graph(&[
    //
    (1, &[2]),
    (2, &[]),
  ]);
  check(&graph, &[2, 1]);
}

#[test]
fn bigger() {
  let graph = mk_graph(&[
    (1, &[4]),
    (2, &[1, 7]),
    (3, &[4, 6, 8]),
    (4, &[5]),
    (5, &[6, 8]),
    (6, &[]),
    (7, &[3, 8, 9]),
    (8, &[9]),
    (9, &[]),
  ]);
  check(&graph, &[9, 8, 6, 5, 4, 3, 7, 1, 2]);
}

#[test]
fn small_cycle() {
  let graph = mk_graph(&[(2, &[1]), (1, &[2])]);
  check_cycle(&graph);
}

#[test]
fn bigger_cycle() {
  let graph = mk_graph(&[
    //
    (1, &[2]),
    (2, &[]),
    (3, &[6]),
    (4, &[5]),
    (5, &[3, 2]),
    (6, &[1, 4]),
  ]);
  check_cycle(&graph);
}

#[test]
fn small_cycle_with_extra() {
  let graph = mk_graph(&[
    //
    (1, &[2]),
    (2, &[1]),
    (3, &[1]),
  ]);
  check_cycle(&graph);
}
