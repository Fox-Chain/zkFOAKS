use crate::parameter::DISTANCE_THRESHOLD;
use prime_field::FieldElement;
use std::collections::HashMap;
use std::vec::Vec;

use crate::parameter::*;

#[derive(Default, Clone)]
pub struct Graph {
  pub degree: i32,
  pub neighbor: Vec<Vec<u64>>,
  pub r_neighbor: Vec<Vec<u64>>,
  pub weight: Vec<Vec<FieldElement>>,
  pub r_weight: Vec<Vec<FieldElement>>,
  pub l: u64,
  pub r: u64,
}

// TODO this can be something like
// this https://crates.io/crates/lazy_static or this https://github.com/matklad/once_cell

// pub static mut D: Vec<Graph> = Vec::new(); // TODO this can actually be lazy_cell

#[derive(Default)]
pub struct LinearCodeEncodeContext {
  pub scratch: Vec<Vec<Vec<FieldElement>>>,
  pub c: Vec<Graph>,
  pub d: Vec<Graph>,
  pub encode_initialized: bool,
}

impl LinearCodeEncodeContext {
  pub fn init() -> Self {
    let scratch = vec![vec![vec![FieldElement::zero()]; 100]; 2];
    //let encode_initialized = false;
    let c = vec![Graph::default(); 100];
    let d = vec![Graph::default(); 100];

    Self {
      scratch,
      c,
      d,
      ..Default::default()
    }
  }

  pub fn encode(
    &mut self,
    src: Vec<FieldElement>,
    dst: &mut Vec<FieldElement>,
    n: u64,
    dep_: Option<usize>,
  ) -> u64 {
    let dep = dep_.unwrap_or(0);
    if !self.encode_initialized {
      self.encode_initialized = true;
      let mut i = 0u64;
      while (n >> i) > 1 {
        let size = ((2 * n) >> i) as usize;
        self.scratch[0][i as usize] = vec![FieldElement::default(); size];
        self.scratch[1][i as usize] = vec![FieldElement::default(); size];
        i = i + 1;
      }
    }
    if n <= DISTANCE_THRESHOLD.try_into().unwrap() {
      for i in 0..(n as usize) {
        dst[i] = src[i];
      }
      return n;
    }
    for i in 0..(n as usize) {
      self.scratch[0][dep][i] = src[i];
    }
    let mut r = (ALPHA * (n as f64)) as u64; //chech here
    for j in 0..r as usize {
      self.scratch[1][dep][j] = FieldElement::zero();
    }
    //expander mult
    for i in 0..(n as usize) {
      let val = src[i];
      for d in 0..self.c[dep].degree as usize {
        let target = self.c[dep].neighbor[i][d] as usize;
        self.scratch[1][dep][target] =
          self.scratch[1][dep][target] + self.c[dep].weight[i][d] * val;
      }
    }
    // TODO
    let l = self.encode(
      self.scratch[1][dep].clone(),
      &mut (self.scratch[0][dep][(n as usize)..]).to_vec(),
      r,
      Some(dep + 1),
    );
    assert_eq![self.d[dep].l, l];
    // R consumed
    r = self.d[dep].r;
    for i in 0..(r as usize) {
      self.scratch[0][dep][(n + l) as usize + i] = FieldElement::from_real(0);
    }
    for i in 0..(l as usize) {
      let ref val = src[i];
      for d in 0..(self.d[dep].degree as usize) {
        let target = self.d[dep].neighbor[i][d];
        self.scratch[0][dep][(n + l + target) as usize] =
          self.scratch[0][dep][(n + l + target) as usize] + *val * self.d[dep].weight[i][d];
      }
    }
    for i in 0..((n + l + r) as usize) {
      dst[i] = self.scratch[0][dep][i];
    }
    // return
    return n + l + r;
  }

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

  pub unsafe fn expander_init(&mut self, n: u64, dep: Option<usize>) -> u64 {
    // random Graph
    if n <= DISTANCE_THRESHOLD as u64 {
      n
    } else {
      let dep = dep.unwrap_or(0);
      self.c[dep as usize] = Self::generate_random_expander(n, (ALPHA * (n as f64)) as u64, CN);
      let l = self.expander_init((ALPHA * (n as f64)) as u64, Some(dep + 1));
      self.d[dep as usize] =
        Self::generate_random_expander(l, ((n as f64) * (R - 1f64) - (l as f64)) as u64, DN);
      n + l + (((n as f64) * (R - 1.0) - (l as f64)) as u64)
    }
  }
}
