use crate::expanders::ALPHA;
use crate::expanders::C;
use crate::expanders::D;
use crate::expanders::DISTANCE_THRESHOLD;
use prime_field::FieldElement;
use std::vec::Vec;

// TODO: check initialization
pub static mut SCRATCH: Vec<Vec<Vec<FieldElement>>> = Vec::new();
pub static mut __ENCODE_INITIALIZED: bool = false;

#[inline]
pub unsafe fn encode(
  src: &mut Vec<FieldElement>,
  dst: &mut [FieldElement],
  n: i64,
  dep_: Option<usize>,
) -> i64 {
  let dep = dep_.unwrap_or(0);
  if SCRATCH.len() < 2 {
    SCRATCH = vec![Vec::new(), Vec::new()];
  }
  if !__ENCODE_INITIALIZED {
    __ENCODE_INITIALIZED = true;
    let mut i = 0i64;
    while (n >> i) > 1i64 {
      let size: usize = ((2 * n) >> i) as usize;
      let entry_0 = vec![FieldElement::default(); size];
      SCRATCH[0].push(entry_0);
      let entry_1 = vec![FieldElement::default(); size];
      SCRATCH[1].push(entry_1);
      i = i + 1i64;
    }
  }
  if n <= DISTANCE_THRESHOLD as i64 {
    for i in 0..(n as usize) {
      // TODO: check out-of-range
      dst[i] = src[i];
    }
    // return
    return n;
  }
  for i in 0..(n as usize) {
    SCRATCH[0][dep][i] = src[i];
  }
  let R: i64 = (ALPHA * (n as f64)) as i64;
  for j in 0..(R as usize) {
    SCRATCH[1][dep][j] = FieldElement::from_real(0);
  }
  for i in 0..(n as usize) {
    let ref val = src[i];
    for d in (0..C[dep].degree as usize) {
      let target = C[dep].neighbor[i][d] as usize;
      SCRATCH[1][dep][target] = SCRATCH[1][dep][target] + C[dep].weight[i][d] * *val;
    }
  }
  // recursion
  // TODO
  let L = encode(
    &mut SCRATCH[1][dep],
    &mut SCRATCH[0][dep][(n as usize)..((n + R) as usize)],
    R,
    Some(dep + 1),
  );
  assert_eq![D[dep].L, L];
  // R consumed
  let R = D[dep].R;
  for i in 0..(R as usize) {
    SCRATCH[0][dep][(n + L + R) as usize] = FieldElement::from_real(0);
  }
  for i in 0..(L as usize) {
    let ref val = src[i];
    for d in 0..(D[dep].degree as usize) {
      let target = D[dep].neighbor[i][d];
      SCRATCH[0][dep][(n + L + target) as usize] =
        SCRATCH[0][dep][(n + L + target) as usize] + *val * D[dep].weight[i][d];
    }
  }
  for i in 0..((n + L + R) as usize) {
    dst[i] = SCRATCH[0][dep][i];
  }
  // return
  return n + L + R;
}

// TODO this can be something like
// this https://crates.io/crates/lazy_static or this https://github.com/matklad/once_cell

//#[derive(Default)]
// pub struct ExpanderContext {
//   pub c: Vec<Graph>,
//   pub d: Vec<Graph>,
// }

// impl ExpanderContext {
//   pub fn default() -> Self {
//     Self {
//       c: vec![Graph::default(); 100],
//       d: vec![Graph::default(); 100],
//     }
//   }
