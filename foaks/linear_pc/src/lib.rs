use infrastructure::my_hash::HashDigest;
use linear_code::linear_code_encode::Graph;
use linear_code::parameter::{ALPHA, CN, COLUMN_SIZE, DISTANCE_THRESHOLD, DN, R};
use linear_gkr::prover::ZkProver;
use linear_gkr::verifier::ZkVerifier;
use prime_field::FieldElement;

//commit
pub struct LinearPC {
  encoded_codeword: Vec<Vec<FieldElement>>,
  coef: Vec<Vec<FieldElement>>,
  codeword_size: Vec<u64>,
  mt: Vec<HashDigest>,
  p: ZkProver,
  v: ZkVerifier,
}

impl LinearPC {
  pub fn commit(
    &mut self,
    src: Vec<FieldElement>,
    n: u64,
    linear_code_encode: &mut LinearCodeEncodeContext,
  ) -> Vec<HashDigest> {
    let mut stash = vec![HashDigest::new(); (n / COLUMN_SIZE * 2).try_into().unwrap()];
    self.codeword_size = vec![0; COLUMN_SIZE.try_into().unwrap()];
    assert_eq!(n % COLUMN_SIZE, 0);
    self.encoded_codeword = vec![vec![FieldElement::zero()]; COLUMN_SIZE.try_into().unwrap()];
    self.coef = vec![vec![FieldElement::zero()]; COLUMN_SIZE.try_into().unwrap()];

    //new code
    for i in 0..COLUMN_SIZE as usize {
      self.encoded_codeword[i] =
        vec![FieldElement::zero(); (n / COLUMN_SIZE * 2).try_into().unwrap()];
      self.coef[i] = vec![FieldElement::zero(); COLUMN_SIZE.try_into().unwrap()];
      // Todo: Debug, could use std::ptr::copy_nonoverlapping() instead
      self.coef[i] = (src[i * (n / COLUMN_SIZE) as usize..]).to_vec();

      //memset(encoded_codeword[i], 0, sizeof(prime_field::field_element) * n / COLUMN_SIZE * 2);

      self.codeword_size[i] = linear_code_encode.encode(
        (src[i * (n / COLUMN_SIZE) as usize..]).to_vec(),
        &mut self.encoded_codeword[i],
        n / COLUMN_SIZE,
        Some(0),
      );
    }

    for i in 0..(n / COLUMN_SIZE * 2) as usize {
      //memset(&stash[i], 0, sizeof(__hhash_digest));
      stash[i] = HashDigest::default();
      for j in 0..(COLUMN_SIZE / 2) as usize {}
    }
    unimplemented!();
  }
}

pub struct LinearCodeEncodeContext {
  pub scratch: Vec<Vec<Vec<FieldElement>>>,
  pub encode_initialized: bool,
  pub c: Vec<Graph>,
  pub d: Vec<Graph>,
}

impl LinearCodeEncodeContext {
  pub fn init() -> Self {
    let scratch = vec![vec![vec![FieldElement::zero()]; 100]; 2];
    let encode_initialized = false;
    let c = vec![Graph::default(); 100];
    let d = vec![Graph::default(); 100];

    Self {
      scratch,
      encode_initialized,
      c,
      d,
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
    let r = (ALPHA * (n as f64)) as u64; //chech here
    for j in 0..r as usize {
      self.scratch[1][dep][j] = FieldElement::zero();
    }
    //expander mult
    for i in 0..(n as usize) {
      let ref val = src[i];
      for d in 0..self.c[dep].degree as usize {
        let target = self.c[dep].neighbor[i][d] as usize;
        self.scratch[1][dep][target] =
          self.scratch[1][dep][target] + self.c[dep].weight[i][d] * *val;
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
    let r = self.d[dep].r;
    for i in 0..(R as usize) {
      self.scratch[0][dep][(n + l + r) as usize] = FieldElement::from_real(0);
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

  pub fn expander_init(&mut self, n: u64, dep: Option<i32>) -> u64 {
    // random Graph
    if n <= DISTANCE_THRESHOLD.try_into().unwrap() {
      n
    } else {
      let mut dep_ = dep.unwrap_or(0i32);
      self.c[dep_ as usize] = generate_random_expander(n, (ALPHA * (n as f64)) as u64, CN as u64);
      let L = self.expander_init((ALPHA * (n as f64)) as u64, Some(dep_ + 1i32));
      self.d[dep_ as usize] =
        generate_random_expander(L, ((n as f64) * (R - 1f64) - (L as f64)) as u64, DN as u64);
      n + L + (((n as f64) * (R - 1.0) - (L as f64)) as u64)
    }
  }
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

//open

//verify
