use prime_field::FieldElement;
use std::vec::Vec;

use crate::{
  expanders::{C, D},
  parameter::{ALPHA, DISTANCE_THRESHOLD},
};

// TODO: check initialization
pub struct LinearCodeEncode {
  scratch: Vec<Vec<Vec<FieldElement>>>,
  encode_initialized: bool,
}

impl LinearCodeEncode {
  pub fn init() -> Self {
    let scratch = vec![vec![vec![FieldElement::zero()]; 100]; 2];
    let encode_initialized = false;
    Self {
      scratch,
      encode_initialized,
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
    // if self.scratch.len() < 2 {
    //   self.scratch = vec![Vec::new(), Vec::new()];
    //}
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
        // TODO: check out-of-range
        dst[i] = src[i];
      }
      return n;
    }
    for i in 0..(n as usize) {
      self.scratch[0][dep][i] = src[i];
    }
    let R = (ALPHA * (n as f64)) as u64;
    for j in 0..(R as usize) {
      self.scratch[1][dep][j] = FieldElement::from_real(0);
    }
    for i in 0..(n as usize) {
      let ref val = src[i];
      for d in 0..C[dep].degree as usize {
        let target = C[dep].neighbor[i][d] as usize;
        self.scratch[1][dep][target] = self.scratch[1][dep][target] + C[dep].weight[i][d] * *val;
      }
    }
    // recursion
    // TODO
    let L = encode(
      &mut scratch[1][dep],
      &mut scratch[0][dep][(n as usize)..((n + R) as usize)],
      R,
      Some(dep + 1),
    );
    assert_eq![D[dep].l, L];
    // R consumed
    let R = D[dep].r;
    for i in 0..(R as usize) {
      scratch[0][dep][(n + L + R) as usize] = FieldElement::from_real(0);
    }
    for i in 0..(L as usize) {
      let ref val = src[i];
      for d in 0..(D[dep].degree as usize) {
        let target = D[dep].neighbor[i][d];
        scratch[0][dep][(n + L + target) as usize] =
          scratch[0][dep][(n + L + target) as usize] + *val * D[dep].weight[i][d];
      }
    }
    for i in 0..((n + L + R) as usize) {
      dst[i] = scratch[0][dep][i];
    }
    // return
    return n + L + R;
  }
}
