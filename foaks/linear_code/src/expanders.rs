use prime_field::FieldElement;
use std::vec::Vec;

use crate::parameter::{ALPHA, CN, DISTANCE_THRESHOLD, DN, R};

#[derive(Default)]
pub struct Graph {
  pub degree: i32,
  pub neighbor: Vec<Vec<u64>>,
  pub r_neighbor: Vec<Vec<u64>>,
  pub weight: Vec<Vec<FieldElement>>,
  pub r_weight: Vec<Vec<FieldElement>>,
  pub l: u64,
  pub r: u64,
}

// TODO

// pub static mut C: [Graph; 100] = [Graph::default(); 100];
// pub static mut D: [Graph; 100] = [Graph::default(); 100];

// TODO this can be something like
// this https://crates.io/crates/lazy_static or this https://github.com/matklad/once_cell

pub static mut C: Vec<Graph> = Vec::new(); // TODO this can actually be lazy_cell
pub static mut D: Vec<Graph> = Vec::new(); // TODO this can actually be lazy_cell

#[inline]
pub fn generate_random_expander(l: u64, r: u64, d: u64) -> Graph {
  let mut ret: Graph = Graph::default();
  ret.degree = i32::try_from(d).unwrap();
  ret.neighbor.truncate(l as usize);
  ret.weight.truncate(l as usize);

  ret.r_neighbor.truncate(r as usize);
  ret.r_weight.truncate(r as usize);

  for i in 0..(l as usize) {
    ret.neighbor[i].truncate(d as usize);
    ret.weight[i].truncate(d as usize);
    for j in 0..(d as usize) {
      let target = rand::random::<u64>() % r;
      // TODO
      // let weight: FieldElement = prime_field::random();
      let weight = FieldElement::default();
      ret.neighbor[i][j] = target;
      ret.r_neighbor[target as usize].push(i as u64);
      ret.r_weight[target as usize].push(weight);
      ret.weight[i][j] = weight;
    }
  }

  ret.l = l;
  ret.r = r;
  ret
}

#[inline]
pub unsafe fn expander_init(n: u64, dep: Option<i32>) -> u64 {
  // random Graph
  if n <= DISTANCE_THRESHOLD.try_into().unwrap() {
    n
  } else {
    let mut dep_ = dep.unwrap_or(0i32);
    C[dep_ as usize] = generate_random_expander(n, (ALPHA * (n as f64)) as u64, CN as u64);
    let L = expander_init((ALPHA * (n as f64)) as u64, Some(dep_ + 1i32));
    D[dep_ as usize] =
      generate_random_expander(L, ((n as f64) * (R - 1f64) - (L as f64)) as u64, DN as u64);
    n + L + (((n as f64) * (R - 1.0) - (L as f64)) as u64)
  }
}
