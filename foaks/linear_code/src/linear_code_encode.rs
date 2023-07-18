use std::vec::Vec;

use prime_field::FieldElement;

use crate::parameter::DISTANCE_THRESHOLD;
use crate::parameter::*;

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

// TODO this can be something like
// this https://crates.io/crates/lazy_static or this https://github.com/matklad/once_cell

// pub static mut D: Vec<Graph> = Vec::new(); // TODO this can actually be
// lazy_cell

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

  pub fn encode(&mut self, src: Vec<FieldElement>, n: usize) -> (usize, Vec<FieldElement>) {
    let dep = 0;
    if !self.encode_initialized {
      self.encode_initialized = true;
      let mut i = 0usize;
      while (n >> i) > 1 {
        let size = (2 * n) >> i;
        self.scratch[0][i] = vec![FieldElement::default(); size];
        self.scratch[1][i] = vec![FieldElement::default(); size];
        i += 1
      }
    }
    self.scratch[0][dep][..n].copy_from_slice(&src[..n]);
    let mut r: usize = (ALPHA * (n as f64)) as usize;

    self.scratch[1][dep].iter_mut().for_each(|item| {
      *item = FieldElement::zero();
    });
    //expander mult
    for i in 0..n {
      for d in 0..self.c[dep].degree {
        let target = self.c[dep].neighbor[i][d];
        self.scratch[1][dep][target] =
          self.scratch[1][dep][target] + self.c[dep].weight[i][d] * src[i];
      }
    }
    let l: usize = self.encode_scratch(r, Some((n, dep + 1)));

    assert_eq!(self.d[dep].l, l);
    // R consumed
    r = self.d[dep].r;
    for i in 0..r {
      self.scratch[0][dep][n + l + i] = FieldElement::from_real(0);
    }
    for i in 0..l {
      let val = self.scratch[0][dep][n + i];
      for d in 0..self.d[dep].degree {
        let target = self.d[dep].neighbor[i][d];
        self.scratch[0][dep][n + l + target] =
          self.scratch[0][dep][n + l + target] + val * self.d[dep].weight[i][d];
      }
    }

    let dst = &self.scratch[0][dep][..(n + l + r)];
    (n + l + r, dst.to_vec())
  }

  pub fn encode_scratch(&mut self, n: usize, pre_n_and_dep: Option<(usize, usize)>) -> usize {
    let (pre_n, dep) = match pre_n_and_dep {
      Some((pre_n, dep)) => (pre_n, dep),
      None => (0, 0),
    };
    if n <= DISTANCE_THRESHOLD {
      for i in 0..n {
        self.scratch[0][dep - 1][pre_n + i] = self.scratch[1][dep - 1][i];
      }
      return n;
    }
    for i in 0..n {
      self.scratch[0][dep][i] = self.scratch[1][dep - 1][i];
    }
    let mut r = (ALPHA * (n as f64)) as usize; //chech here
    for j in 0..r {
      self.scratch[1][dep][j] = FieldElement::zero();
    }
    //expander mult
    for i in 0..n {
      let val = self.scratch[1][dep - 1][i];
      for d in 0..self.c[dep].degree {
        let target = self.c[dep].neighbor[i][d];
        self.scratch[1][dep][target] =
          self.scratch[1][dep][target] + self.c[dep].weight[i][d] * val;
      }
    }
    let l: usize = self.encode_scratch(r, Some((n, dep + 1)));
    assert_eq!(self.d[dep].l, l);
    // R consumed
    r = self.d[dep].r;
    for i in 0..r {
      self.scratch[0][dep][n + l + i] = FieldElement::from_real(0);
    }
    for i in 0..l {
      let val = self.scratch[0][dep][n + i];
      for d in 0..self.d[dep].degree {
        let target = self.d[dep].neighbor[i][d];
        self.scratch[0][dep][n + l + target] =
          self.scratch[0][dep][n + l + target] + val * self.d[dep].weight[i][d];
      }
    }

    for i in 0..(n + l + r) {
      self.scratch[0][dep - 1][pre_n + i] = self.scratch[0][dep][i];
    }

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

pub fn generate_random_expander(l: usize, r: usize, d: usize) -> Graph {
  //let mut ret: Graph = Graph::default();
  let degree = d;
  let mut neighbor = vec![vec![]; l];
  let mut weight = vec![vec![]; l];

  let mut r_neighbor = vec![vec![]; r];
  let mut r_weight = vec![vec![]; r];

  for i in 0..l {
    neighbor[i] = vec![0; d];
    weight[i] = vec![FieldElement::zero(); d];
    for j in 0..d {
      let target = rand::random::<usize>() % r;
      let tmp_weight = FieldElement::new_random();
      neighbor[i][j] = target;
      r_neighbor[target].push(i);
      r_weight[target].push(tmp_weight);
      weight[i][j] = tmp_weight;
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
