use crate::parameter::DISTANCE_THRESHOLD;
#[allow(unused)]
use crate::parameter::*;
use prime_field::FieldElement;
use std::{
  fs::read_to_string,
  str::{Lines, Split},
  vec::Vec,
};

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

  pub fn encode(
    &mut self,
    src: Vec<FieldElement>,
    dst: &mut Vec<FieldElement>,
    n: usize,
    dep_: Option<usize>,
  ) -> usize {
    let dep = dep_.unwrap_or(0);
    if !self.encode_initialized {
      self.encode_initialized = true;
      let mut i = 0usize;
      while (n >> i) > 1 {
        let size = (2 * n) >> i;
        self.scratch[0][i] = vec![FieldElement::default(); size];
        self.scratch[1][i] = vec![FieldElement::default(); size];
        i = i + 1;
      }
    }
    if n <= DISTANCE_THRESHOLD.try_into().unwrap() {
      for i in 0..n {
        dst[i] = src[i];
      }
      return n;
    }
    for i in 0..n {
      self.scratch[0][dep][i] = src[i];
    }
    let mut r = (ALPHA * (n as f64)) as usize; //chech here
    for j in 0..r {
      self.scratch[1][dep][j] = FieldElement::zero();
    }
    //expander mult
    for i in 0..n {
      let val = src[i];
      for d in 0..self.c[dep].degree {
        let target = self.c[dep].neighbor[i][d];
        self.scratch[1][dep][target] =
          self.scratch[1][dep][target] + self.c[dep].weight[i][d] * val;
      }
    }
    // TODO
    let l = self.encode(
      self.scratch[1][dep].clone(),
      &mut (self.scratch[0][dep][n..]).to_vec(),
      r,
      Some(dep + 1),
    );
    assert_eq!(self.d[dep].l, l);
    // R consumed
    r = self.d[dep].r;
    for i in 0..r {
      self.scratch[0][dep][n + l + i] = FieldElement::from_real(0);
    }
    for i in 0..l {
      let ref val = src[i];
      for d in 0..self.d[dep].degree {
        let target = self.d[dep].neighbor[i][d];
        self.scratch[0][dep][n + l + target] =
          self.scratch[0][dep][n + l + target] + *val * self.d[dep].weight[i][d];
      }
    }
    for i in 0..(n + l + r) {
      dst[i] = self.scratch[0][dep][i];
    }
    // return
    return n + l + r;
  }

  pub unsafe fn expander_init(&mut self, n: usize, dep: Option<usize>) -> usize {
    // random Graph
    if n <= DISTANCE_THRESHOLD {
      n
    } else {
      let dep = dep.unwrap_or(0);
      self.c[dep] = generate_random_expander(n, (ALPHA * (n as f64)) as usize, CN);
      let l = self.expander_init((ALPHA * (n as f64)) as usize, Some(dep + 1));
      self.d[dep] =
        generate_random_expander(l, ((n as f64) * (R - 1f64) - (l as f64)) as usize, DN);
      n + l + (((n as f64) * (R - 1.0) - (l as f64)) as usize)
    }
  }
}
pub fn generate_random_expander(l: usize, r: usize, d: usize) -> Graph {
  let mut ret: Graph = Graph::default();
  ret.degree = d;
  ret.neighbor = vec![vec![]; l];
  ret.weight = vec![vec![]; l];

  ret.r_neighbor = vec![vec![]; r];
  ret.r_weight = vec![vec![]; r];

  //Comment this block of code for now. For testing purpose we use data from Orion C++

  // for i in 0..l {
  //   ret.neighbor[i] = vec![0; d];
  //   ret.weight[i] = vec![FieldElement::zero(); d];
  //   for j in 0..d {
  //     let target = rand::random::<usize>() % r;
  //     let weight = FieldElement::new_random();
  //     ret.neighbor[i][j] = target;
  //     ret.r_neighbor[target].push(i);
  //     ret.r_weight[target].push(weight);
  //     ret.weight[i][j] = weight;
  //   }
  // }
  //Improve this for later, hardocoded 10
  if l == 128 {
    ret.neighbor = read_neighbor_graph_file("c_0_neighbor.txt");
    ret.r_neighbor = read_neighbor_graph_file("c_0_r_neighbor.txt");
    ret.r_weight = read_weight_graph_file("c_0_r_weight.txt");
    ret.weight = read_weight_graph_file("c_0_weight.txt");
  } else if l == 30 {
    ret.neighbor = read_neighbor_graph_file("c_1_neighbor.txt");
    ret.r_neighbor = read_neighbor_graph_file("c_1_r_neighbor.txt");
    ret.r_weight = read_weight_graph_file("c_1_r_weight.txt");
    ret.weight = read_weight_graph_file("c_1_weight.txt");
  } else if l == 7 {
    //todo for Edu leer d_1_.....
    todo!();
  } else if l == 51 {
    //todo for Edu leer d_0_.....
    todo!();
  }
  ret.l = l;
  ret.r = r;
  ret
}
pub fn read_weight_graph_file(path: &str) -> Vec<Vec<FieldElement>> {
  let result_content = read_to_string(path).unwrap();
  let result_lines = result_content.lines();
  let res: Vec<Vec<FieldElement>> = result_lines
    .into_iter()
    .map(|x| {
      let mut vec = Vec::new();
      let mut block_line = x.split_whitespace();
      let size = block_line.clone().count();
      for _i in 0..size / 2 {
        let real: u64 = block_line.next().unwrap().parse().unwrap();
        let img: u64 = block_line.next().unwrap().parse().unwrap();
        vec.push(FieldElement::new(real, img));
      }
      vec
    })
    .collect();
  res
}

pub fn read_neighbor_graph_file(path: &str) -> Vec<Vec<usize>> {
  let result_content = read_to_string(path).unwrap();
  let result_lines = result_content.lines();
  let res: Vec<Vec<usize>> = result_lines
    .into_iter()
    .map(|x| {
      let mut vec = Vec::new();
      let mut block_line = x.split_whitespace();
      let size = block_line.clone().count();
      for _i in 0..size {
        let elem: usize = block_line.next().unwrap().parse().unwrap();
        vec.push(elem);
      }
      vec
    })
    .collect();
  res
}
