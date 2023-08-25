use std::vec::Vec;

use prime_field::FieldElement;

use crate::parameter::DISTANCE_THRESHOLD;
use crate::parameter::*;

use infrastructure::constants::REAL_ZERO;

#[derive(Default, Clone)]
pub struct Graph {
  pub degree: usize,
  pub neighbor: Vec<Vec<usize>>,
  pub r_neighbor: Vec<Vec<usize>>,
  pub weight: Vec<Vec<FieldElement>>,
  pub r_weight: Vec<Vec<FieldElement>>,
  pub l: usize,
  pub r: usize,
}

#[derive(Default)]
pub struct LinearCodeEncodeContext {
  pub scratch: Vec<Vec<Vec<FieldElement>>>,
  pub c: Vec<Graph>,
  pub d: Vec<Graph>,
  pub encode_initialized: bool,
}

impl LinearCodeEncodeContext {
  // Todo: Refactor, use Vec::with_capacity(100) instead of vec![vec![]; 100]
  pub fn init() -> Self {
    let scratch = vec![vec![vec![]; 100]; 2];
    let c = vec![Graph::default(); 100];
    let d = vec![Graph::default(); 100];

    Self {
      scratch,
      c,
      d,
      ..Default::default()
    }
  }

  pub fn encode(&mut self, src: &[FieldElement]) -> Vec<FieldElement> {
    // Todo Refactor, reuse code from encode_scratch
    // Todo Refactor, delete self.c[dep].degree, self.d[dep].degree
    let n = src.len();
    let dep = 0;
    if !self.encode_initialized {
      self.encode_initialized = true;
      let mut i = 0;
      while (n >> i) > 1 {
        let size = (2 * n) >> i;
        self.scratch[0][i] = vec![FieldElement::default(); size];
        self.scratch[1][i] = vec![FieldElement::default(); size];
        i += 1
      }
    }

    self.scratch[0][dep][..n].copy_from_slice(src);
    let mut r: usize = (ALPHA * (n as f64)) as usize;

    self.scratch[1][dep].fill(REAL_ZERO);

    //expander mult
    for (i, elem) in src.iter().enumerate() {
      for d in 0..self.c[dep].degree {
        let target = self.c[dep].neighbor[i][d];
        self.scratch[1][dep][target] =
          self.scratch[1][dep][target] + self.c[dep].weight[i][d] * *elem;
      }
    }
    let l: usize = self.encode_scratch(r, n, dep + 1);

    assert_eq!(self.d[dep].l, l);
    // R consumed
    r = self.d[dep].r;
    let zeros = vec![FieldElement::from_real(0); r];
    self.scratch[0][dep][n + l..n + l + r].copy_from_slice(&zeros);

    for (i, val) in self.scratch[0][dep][n..n + l].to_owned().iter().enumerate() {
      for d in 0..self.d[dep].degree {
        let target = self.d[dep].neighbor[i][d];
        self.scratch[0][dep][n + l + target] =
          self.scratch[0][dep][n + l + target] + *val * self.d[dep].weight[i][d];
      }
    }

    let dst = &self.scratch[0][dep][..n + l + r];
    dst.to_owned()
  }

  pub fn encode_scratch(&mut self, n: usize, pre_n: usize, dep: usize) -> usize {
    if n <= DISTANCE_THRESHOLD {
      let slc = self.scratch[1][dep - 1][..n].to_owned();
      self.scratch[0][dep - 1][pre_n..pre_n + n].copy_from_slice(&slc);
      return n;
    }
    let slc = self.scratch[1][dep - 1][..n].to_owned();
    self.scratch[0][dep][..n].copy_from_slice(&slc);

    let mut r = (ALPHA * (n as f64)) as usize;
    self.scratch[1][dep].fill(REAL_ZERO);

    //expander mult
    for (i, val) in self.scratch[1][dep - 1]
      .to_owned()
      .iter()
      .enumerate()
      .take(n)
    {
      for d in 0..self.c[dep].degree {
        let target = self.c[dep].neighbor[i][d];
        self.scratch[1][dep][target] =
          self.scratch[1][dep][target] + self.c[dep].weight[i][d] * *val;
      }
    }
    let l: usize = self.encode_scratch(r, n, dep + 1);
    assert_eq!(self.d[dep].l, l);
    // R consumed
    r = self.d[dep].r;
    let zeros = vec![REAL_ZERO; r];
    self.scratch[0][dep][n + l..n + l + r].copy_from_slice(&zeros);

    for (i, val) in self.scratch[0][dep][n..n + l].to_owned().iter().enumerate() {
      for d in 0..self.d[dep].degree {
        let target = self.d[dep].neighbor[i][d];
        self.scratch[0][dep][n + l + target] =
          self.scratch[0][dep][n + l + target] + *val * self.d[dep].weight[i][d];
      }
    }

    let slc = self.scratch[0][dep][..n + l + r].to_owned();
    self.scratch[0][dep - 1][pre_n..pre_n + n + l + r].copy_from_slice(&slc);

    n + l + r
  }

  pub fn expander_init(&mut self, n: usize, dep: Option<usize>) -> usize {
    match n <= DISTANCE_THRESHOLD {
      true => n,
      false => {
        let dep = dep.unwrap_or(0);
        let alpha_n = (ALPHA * n as f64) as usize;
        let (l, expander_size) = {
          let l = self.expander_init(alpha_n, Some(dep + 1));
          let expander_size = ((n as f64) * (R - 1.0) - l as f64) as usize;
          (l, expander_size)
        };

        self.c[dep] = generate_random_expander(n, alpha_n, CN);
        self.d[dep] = generate_random_expander(l, expander_size, DN);

        n + l + expander_size
      }
    }
  }
}

pub fn generate_random_expander(l: usize, r: usize, degree: usize) -> Graph {
  let mut neighbor = Vec::with_capacity(l);
  let mut weight = Vec::with_capacity(l);

  let mut r_neighbor = vec![vec![]; r];
  let mut r_weight = vec![vec![]; r];

  for i in 0..l {
    neighbor.push(Vec::with_capacity(degree));
    weight.push(Vec::with_capacity(degree));
    for _ in 0..degree {
      let target = rand::random::<usize>() % r;
      let tmp_weight = FieldElement::new_random();
      neighbor[i].push(target);
      r_neighbor[target].push(i);
      r_weight[target].push(tmp_weight);
      weight[i].push(tmp_weight);
    }
  }

  Graph {
    degree,
    neighbor,
    weight,
    l,
    r,
    r_neighbor,
    r_weight,
  }
}
