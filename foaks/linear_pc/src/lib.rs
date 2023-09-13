use std::{collections::HashMap, time::Instant};

use infrastructure::{
  merkle_tree::{self, create_tree},
  my_hash::HashDigest,
  utility::my_log,
};
use global::constants::*;
use linear_code::{
  linear_code_encode::LinearCodeEncodeContext,
  parameter::{CN, COLUMN_SIZE, DISTANCE_THRESHOLD, DN, TARGET_DISTANCE},
};
use linear_gkr::{
  circuit_fast_track::{Gate, Layer},
  verifier::ZkVerifier,
};
use prime_field::FieldElement;

use crate::parameters::*;

mod parameters;

#[derive(Default)]
pub struct LinearPC {
  encoded_codeword: Vec<Vec<FieldElement>>,
  coef: Vec<Vec<FieldElement>>,
  codeword_size: Vec<usize>,
  mt: Vec<HashDigest>,
  verifier: ZkVerifier,
  lce_ctx: LinearCodeEncodeContext,
  gates_count: HashMap<usize, usize>,
}

impl LinearPC {
  pub fn init(n: usize) -> Self {
    let mut lce_ctx = LinearCodeEncodeContext::init();
    lce_ctx.expander_init(n / COLUMN_SIZE, None);
    Self {
      lce_ctx,
      codeword_size: Vec::with_capacity(COLUMN_SIZE),
      encoded_codeword: Vec::with_capacity(COLUMN_SIZE),
      coef: Vec::with_capacity(COLUMN_SIZE),
      ..Default::default()
    }
  }
  pub fn commit(&mut self, src: &[FieldElement]) -> Vec<HashDigest> {
    //Todo: Refactor, delete self.codeword_size field
    let n: usize = src.len();

    assert_eq!(n % COLUMN_SIZE, 0);

    let segment = n / COLUMN_SIZE;
    for i in 0..COLUMN_SIZE {
      let begin = i * segment;
      let end = (i + 1) * segment;
      let src_slice = &src[begin..end];
      self.coef.push(src_slice.to_vec());

      let mut dst = self.lce_ctx.encode(src_slice);
      let size = dst.len();
      self.codeword_size.push(size);
      dst.resize(2 * segment, FE_ZERO);
      self.encoded_codeword.push(dst);
    }

    let stash: Vec<HashDigest> = (0..(segment * 2))
      .map(|i| {
        (0..(COLUMN_SIZE / 2)).fold(HashDigest::default(), |acc, j| {
          merkle_tree::hash_double_field_element_merkle_damgard(
            self.encoded_codeword[2 * j][i],
            self.encoded_codeword[2 * j + 1][i],
            acc,
          )
        })
      })
      .collect();

    create_tree(&mut self.mt, &stash, true);
    self.mt.clone()
  }

  fn generate_circuit(&mut self, query: &mut [usize], n: usize, input: &[FieldElement]) {
    let query_count = query.len();
    query.sort();
    self.prepare_gates_count(n, query_count);
    println!("Depth {}", self.gates_count.len());
    assert_eq!((1 << my_log(n).expect("Failed to compute logarithm")), n);

    self.verifier.a_c.inputs = Vec::with_capacity(n);
    self.verifier.a_c.total_depth = self.gates_count.len() + 1;
    self.verifier.a_c.circuit = Vec::with_capacity(self.verifier.a_c.total_depth);
    let bit_length = my_log(n).expect("Failed to compute bit_length");
    let layer_0 = Layer {
      bit_length,
      gates: vec![Gate::new(); 1 << bit_length],
      ..Default::default()
    };

    self.verifier.a_c.circuit.push(layer_0);

    //Refactored code, add - 1 to the bucle to avoid the last layer
    for i in 0..self.gates_count.len() - 1 {
      let bit_length = my_log(smallest_pow2_larger_or_equal_to(
        *self
          .gates_count
          .get(&i)
          .expect("Failed to retrieve gates_count value"),
      ))
      .expect("Failed to compute bit_length");

      let gates = vec![Gate::new(); 1 << bit_length];

      let layer_i = Layer {
        bit_length,
        gates,
        ..Default::default()
      };

      self.verifier.a_c.circuit.push(layer_i);
    }

    let bit_length =
      my_log(smallest_pow2_larger_or_equal_to(query_count)).expect("Failed to compute bit_length");

    let final_layer = Layer {
      bit_length,
      gates: vec![Gate::new(); 1 << bit_length],
      ..Default::default()
    };

    self.verifier.a_c.circuit.push(final_layer);

    for (i, elem) in input.iter().enumerate().take(n) {
      self.verifier.a_c.inputs.push(*elem);
      self.verifier.a_c.circuit[0].gates[i] = Gate::from_params(INPUT, 0, 0);
      self.verifier.a_c.circuit[1].gates[i] = Gate::from_params(DIRECT_RELAY, i, 0);
    }

    for i in 0..n {
      self.verifier.a_c.circuit[2].gates[i] = Gate::from_params(RELAY, i, 0);
    }

    self.verifier.a_c.circuit[2].src_expander_c_mempool = vec![0; CN * self.lce_ctx.c[0].l];
    self.verifier.a_c.circuit[2].weight_expander_c_mempool =
      vec![FE_ZERO; CN * self.lce_ctx.c[0].l];
    let mut c_mempool_ptr = 0;
    let mut d_mempool_ptr = 0;
    for i in 0..self.lce_ctx.c[0].r {
      self.verifier.a_c.circuit[2].gates[i + n] = Gate::from_params(CUSTOM_LINEAR_COMB, 0, 0);
      self.verifier.a_c.circuit[2].gates[i + n].parameter_length =
        self.lce_ctx.c[0].r_neighbor[i].len();
      self.verifier.a_c.circuit[2].gates[i + n].src =
        self.verifier.a_c.circuit[2].src_expander_c_mempool[c_mempool_ptr..].to_vec();
      self.verifier.a_c.circuit[2].gates[i + n].weight =
        self.verifier.a_c.circuit[2].weight_expander_c_mempool[c_mempool_ptr..].to_vec();
      c_mempool_ptr += self.lce_ctx.c[0].r_neighbor[i].len();
      for j in 0..self.lce_ctx.c[0].r_neighbor[i].len() {
        let l = self.lce_ctx.c[0].r_neighbor[i][j];
        let r = i;
        let weight = self.lce_ctx.c[0].r_weight[r][j];
        self.verifier.a_c.circuit[2].gates[i + n].src[j] = l;
        self.verifier.a_c.circuit[2].gates[i + n].weight[j] = weight;
      }
    }

    let output_depth_output_size =
      self.generate_enc_circuit(self.lce_ctx.c[0].r, n + self.lce_ctx.c[0].r, 1, 2);
    // add final output
    let final_output_depth = output_depth_output_size.0 + 1;

    for (i, elem) in self.verifier.a_c.circuit[final_output_depth]
      .gates
      .iter_mut()
      .enumerate()
      .take(output_depth_output_size.1)
    {
      *elem = Gate::from_params(RELAY, i, 0);
    }

    //let d_input_offset = n; //Never used
    let output_so_far = output_depth_output_size.1;
    self.verifier.a_c.circuit[final_output_depth].src_expander_d_mempool =
      vec![0; DN * self.lce_ctx.d[0].l];
    self.verifier.a_c.circuit[final_output_depth].weight_expander_d_mempool =
      vec![FE_ZERO; DN * self.lce_ctx.d[0].l];

    for i in 0..self.lce_ctx.d[0].r {
      self.verifier.a_c.circuit[final_output_depth].gates[output_so_far + i].ty =
        CUSTOM_LINEAR_COMB;
      self.verifier.a_c.circuit[final_output_depth].gates[output_so_far + i].parameter_length =
        self.lce_ctx.d[0].r_neighbor[i].len();

      self.verifier.a_c.circuit[final_output_depth].gates[output_so_far + i].src =
        self.verifier.a_c.circuit[final_output_depth].src_expander_d_mempool[d_mempool_ptr..]
          .to_vec();
      self.verifier.a_c.circuit[final_output_depth].gates[output_so_far + i].weight =
        self.verifier.a_c.circuit[final_output_depth].weight_expander_d_mempool[d_mempool_ptr..]
          .to_vec();
      d_mempool_ptr += self.lce_ctx.d[0].r_neighbor[i].len();

      for (j, (neighbor, weight)) in self.lce_ctx.d[0].r_neighbor[i]
        .iter()
        .zip(self.lce_ctx.d[0].r_weight[i].iter())
        .enumerate()
      {
        self.verifier.a_c.circuit[final_output_depth].gates[output_so_far + i].src[j] =
          *neighbor + n;
        self.verifier.a_c.circuit[final_output_depth].gates[output_so_far + i].weight[j] = *weight
      }
    }
    for (i, elem) in query.iter().enumerate() {
      self.verifier.a_c.circuit[final_output_depth + 1].gates[i] =
        Gate::from_params(RELAY, *elem, 0);
    }
    assert_eq!(c_mempool_ptr, CN * self.lce_ctx.c[0].l);
  }

  pub fn tensor_product_protocol(
    &mut self,
    r0: &[FieldElement],
    r1: &[FieldElement],
    n: usize,
    com_mt: Vec<HashDigest>,
  ) -> (FieldElement, bool) {
    let size_r0 = r0.len();
    let size_r1 = r1.len();
    assert_eq!(size_r0 * size_r1, n);

    let segment = n / COLUMN_SIZE;
    let mut visited_com = vec![false; segment * 4];
    let mut visited_combined_com = vec![false; segment * 4];

    let mut proof_size = 0;
    let query_count = (-128f32 / (1f32 - TARGET_DISTANCE).log2()) as usize;
    println!("Query count: {}", query_count);
    println!("Column size: {}", COLUMN_SIZE);
    println!("Number of merkle pathes: {}", query_count);
    println!("Number of field elements: {}", query_count * COLUMN_SIZE);

    //prover construct the combined codeword

    let codeword_size_0 = self.codeword_size[0];
    let mut combined_codeword = vec![FE_ZERO; codeword_size_0];
    let mut combined_codeword_hash = Vec::with_capacity(segment * 2);
    let mut combined_codeword_mt = vec![HashDigest::default(); segment * 4];
    for (i, elem_r0) in r0.iter().enumerate().take(COLUMN_SIZE) {
      for (j, elem_c_c) in combined_codeword.iter_mut().enumerate() {
        *elem_c_c = *elem_c_c + *elem_r0 * self.encoded_codeword[i][j];
      }
    }

    for elem in combined_codeword.iter() {
      combined_codeword_hash.push(merkle_tree::hash_single_field_element(*elem));
    }
    let hash_zero_field_element = merkle_tree::hash_single_field_element(FE_ZERO);
    let hash_zeros = vec![hash_zero_field_element; segment * 2 - codeword_size_0];
    combined_codeword_hash.extend_from_slice(&hash_zeros);

    //merkle commit to combined_codeword
    create_tree(&mut combined_codeword_mt, &combined_codeword_hash, false);

    //prover construct the combined original message
    let mut combined_message = vec![FE_ZERO; n];

    for (i, coef_i) in self.coef.iter().enumerate().take(COLUMN_SIZE) {
      for (j, &coef_ij) in coef_i.iter().enumerate().take(codeword_size_0) {
        combined_message[j] = combined_message[j] + r0[i] * coef_ij;
      }
    }

    //check for encode
    {
      let sliced_message = &combined_message[..segment];
      let test_codeword = self.lce_ctx.encode(sliced_message);
      let test_codeword_size = test_codeword.len();
      assert_eq!(test_codeword_size, codeword_size_0);
      assert_eq!(test_codeword, combined_codeword);
    }

    //verifier random check columns
    let v_t0 = Instant::now();

    for _ in 0..query_count {
      let q = rand::random::<usize>() % codeword_size_0;
      let mut sum = FE_ZERO;
      for (j, elem) in r0.iter().enumerate().take(COLUMN_SIZE) {
        sum = sum + *elem * self.encoded_codeword[j][q];
      }
      proof_size += std::mem::size_of::<FieldElement>() * COLUMN_SIZE;

      //calc hash
      let mut column_hash = HashDigest::default();

      for j in 0..COLUMN_SIZE / 2 {
        column_hash = merkle_tree::hash_double_field_element_merkle_damgard(
          self.encoded_codeword[2 * j][q],
          self.encoded_codeword[2 * j + 1][q],
          column_hash,
        );
      }

      assert!(merkle_tree::verify_claim(
        com_mt[1],
        &com_mt,
        column_hash,
        q,
        segment * 2,
        &mut visited_com,
        &mut proof_size,
      ));
      assert!(merkle_tree::verify_claim(
        combined_codeword_mt[1],
        &combined_codeword_mt,
        merkle_tree::hash_single_field_element(combined_codeword[q]),
        q,
        segment * 2,
        &mut visited_combined_com,
        &mut proof_size,
      ));
      assert_eq!(sum, combined_codeword[q]);
      //add merkle tree open
    }
    let time_span = v_t0.elapsed();
    let mut verification_time = time_span.as_secs_f64();

    // setup code-switching
    let mut answer = FE_ZERO;
    for i in 0..(segment) {
      answer = answer + r1[i] * combined_message[i];
    }

    // prover commit private input
    let mut q = Vec::with_capacity(query_count);
    for _ in 0..query_count {
      q.push(rand::random::<usize>() % self.codeword_size[0]);
    }

    // generate circuit
    self.generate_circuit(&mut q, segment, &combined_message);
    //self.verifier.get_prover(&p); //Refactored, inside of zk_verifier has not
    // zk_prover self.prover.get_circuit(self.verifier.aritmetic_circuit);
    // Refactored, inside of zk_prover.init_array()
    let max_bit_length = self.verifier.a_c.circuit.iter().map(|c| c.bit_length).max();
    let max_bit_length = max_bit_length.expect("Failed to retrieve max_bit_length");
    // p.init_array(max_bit_length); Refactored inside verifier.verify()
    self.verifier.init_array(max_bit_length);

    // p.get_witness(combined_message, N / column_size); Refactored inside
    // verifier.verify()

    let witness = combined_message[..segment].to_vec();

    let (result, time_diff) =
      self
        .verifier
        .verify(max_bit_length, witness, query_count, combined_codeword, q);

    verification_time += time_diff;
    verification_time += self.verifier.v_time;
    proof_size += query_count * std::mem::size_of::<FieldElement>();

    println!(
      "Proof size for tensor IOP {} bytes",
      proof_size + self.verifier.proof_size
    );
    println!("Verification time {}", verification_time);

    (answer, result)
  }

  // Original code use "query" input, but never used it, so I removed it
  fn prepare_gates_count(&mut self, n: usize, query_count: usize) {
    //long long query_ptr = 0;
    // input layer
    self.gates_count.insert(0, n);
    // expander part
    self.gates_count.insert(1, n + self.lce_ctx.c[0].r);
    let output_depth_output_size = self.prepare_enc_count(self.lce_ctx.c[0].r, n, 1);
    self
      .gates_count
      .entry(output_depth_output_size.0 + 1)
      .or_insert(n + output_depth_output_size.1 + self.lce_ctx.d[0].r);

    self
      .gates_count
      .entry(output_depth_output_size.0 + 2)
      .or_insert(query_count);
  }

  fn prepare_enc_count(
    &mut self,
    input_size: usize,
    output_size_so_far: usize,
    depth: usize,
  ) -> (usize, usize) {
    let mut depth = depth;
    let mut input_size = input_size;
    let mut output_size_so_far = output_size_so_far;

    while input_size > DISTANCE_THRESHOLD {
      // Calculate output size for the current depth
      let output_size = output_size_so_far + input_size + self.lce_ctx.c[depth].r;
      self.gates_count.insert(depth + 1, output_size);

      // Prepare for the next depth
      output_size_so_far = output_size;
      input_size = self.lce_ctx.c[depth].r;

      depth += 1;
    }

    let output_depth = depth;
    let output_size = output_size_so_far + self.lce_ctx.d[output_depth].r;
    self.gates_count.insert(output_depth + 1, output_size);

    (
      output_depth + 1,
      *self
        .gates_count
        .get(&(output_depth + 1))
        .expect("Failed to retrieve gates_count value"),
    )
  }

  fn generate_enc_circuit(
    &mut self,
    input_size: usize,
    output_size_so_far: usize,
    recursion_depth: usize,
    input_depth: usize,
  ) -> (usize, usize) {
    if input_size <= DISTANCE_THRESHOLD {
      return (input_depth, output_size_so_far);
    }
    // relay the output
    for (i, gate) in self.verifier.a_c.circuit[input_depth + 1]
      .gates
      .iter_mut()
      .enumerate()
      .take(output_size_so_far)
    {
      *gate = Gate::from_params(RELAY, i, 0);
    }

    self.verifier.a_c.circuit[input_depth + 1].src_expander_c_mempool =
      vec![0; CN * self.lce_ctx.c[recursion_depth].l];
    self.verifier.a_c.circuit[input_depth + 1].weight_expander_c_mempool =
      vec![FE_ZERO; CN * self.lce_ctx.c[recursion_depth].l];
    let mut mempool_ptr = 0;

    for i in 0..self.lce_ctx.c[recursion_depth].r {
      let neighbor_size = self.lce_ctx.c[recursion_depth].r_neighbor[i].len();
      self.verifier.a_c.circuit[input_depth + 1].gates[output_size_so_far + i].ty = 14;
      self.verifier.a_c.circuit[input_depth + 1].gates[output_size_so_far + i].parameter_length =
        neighbor_size;

      self.verifier.a_c.circuit[input_depth + 1].gates[output_size_so_far + i].src =
        self.verifier.a_c.circuit[input_depth + 1].src_expander_c_mempool[mempool_ptr..].to_vec();
      self.verifier.a_c.circuit[input_depth + 1].gates[output_size_so_far + i].weight =
        self.verifier.a_c.circuit[input_depth + 1].weight_expander_c_mempool[mempool_ptr..]
          .to_vec();
      mempool_ptr += self.lce_ctx.c[recursion_depth].r_neighbor[i].len();
      let c_input_offset = output_size_so_far - input_size;
      for j in 0..neighbor_size {
        self.verifier.a_c.circuit[input_depth + 1].gates[output_size_so_far + i].src[j] =
          self.lce_ctx.c[recursion_depth].r_neighbor[i][j] + c_input_offset;
        self.verifier.a_c.circuit[input_depth + 1].gates[output_size_so_far + i].weight[j] =
          self.lce_ctx.c[recursion_depth].r_weight[i][j];
      }
    }

    let output_depth_output_size = self.generate_enc_circuit(
      self.lce_ctx.c[recursion_depth].r,
      output_size_so_far + self.lce_ctx.c[recursion_depth].r,
      recursion_depth + 1,
      input_depth + 1,
    );
    let d_input_offset = output_size_so_far;
    let final_output_depth = output_depth_output_size.0 + 1;
    let output_size_so_far = output_depth_output_size.1;
    mempool_ptr = 0;

    // relay the output
    for (i, gate) in self.verifier.a_c.circuit[final_output_depth]
      .gates
      .iter_mut()
      .enumerate()
      .take(output_size_so_far)
    {
      *gate = Gate::from_params(RELAY, i, 0);
    }

    self.verifier.a_c.circuit[final_output_depth].src_expander_d_mempool =
      vec![0; DN * self.lce_ctx.d[recursion_depth].l];
    self.verifier.a_c.circuit[final_output_depth].weight_expander_d_mempool =
      vec![FE_ZERO; DN * self.lce_ctx.d[recursion_depth].l];

    for i in 0..self.lce_ctx.d[recursion_depth].r {
      let neighbor_size = self.lce_ctx.d[recursion_depth].r_neighbor[i].len();
      self.verifier.a_c.circuit[final_output_depth].gates[output_size_so_far + i].ty =
        CUSTOM_LINEAR_COMB;
      self.verifier.a_c.circuit[final_output_depth].gates[output_size_so_far + i]
        .parameter_length = neighbor_size;
      self.verifier.a_c.circuit[final_output_depth].gates[output_size_so_far + i].src =
        self.verifier.a_c.circuit[final_output_depth].src_expander_d_mempool[mempool_ptr..]
          .to_vec();
      self.verifier.a_c.circuit[final_output_depth].gates[output_size_so_far + i].weight =
        self.verifier.a_c.circuit[final_output_depth].weight_expander_d_mempool[mempool_ptr..]
          .to_vec();
      mempool_ptr += self.lce_ctx.d[recursion_depth].r_neighbor[i].len();
      for j in 0..neighbor_size {
        self.verifier.a_c.circuit[final_output_depth].gates[output_size_so_far + i].src[j] =
          self.lce_ctx.d[recursion_depth].r_neighbor[i][j] + d_input_offset;
        self.verifier.a_c.circuit[final_output_depth].gates[output_size_so_far + i].weight[j] =
          self.lce_ctx.d[recursion_depth].r_weight[i][j];
      }
    }
    (
      final_output_depth,
      output_size_so_far + self.lce_ctx.d[recursion_depth].r,
    )
  }

  pub fn open_and_verify(
    &mut self,
    x: FieldElement,
    n: usize,
    com_mt: Vec<HashDigest>,
  ) -> (FieldElement, bool) {
    assert_eq!(n % COLUMN_SIZE, 0);
    //tensor product of r0 otimes r1
    let mut r0 = Vec::with_capacity(COLUMN_SIZE);
    let mut r1 = Vec::with_capacity(n / COLUMN_SIZE);

    let x_n = x.fast_pow(
      (n / COLUMN_SIZE)
        .try_into()
        .expect("Failed to convert to u128"),
    );
    //Todo: Refactor parallel for loop
    r0.push(FE_REAL_ONE);
    for j in 1..COLUMN_SIZE {
      r0.push(r0[j - 1] * x_n);
    }
    r1.push(FE_REAL_ONE);
    for j in 1..(n / COLUMN_SIZE) {
      r1.push(r1[j - 1] * x);
    }
    self.tensor_product_protocol(&r0, &r1, n, com_mt)
  }

  //Refactored
  pub fn open_and_verify_multi(
    &mut self,
    r: &[FieldElement],
    n: usize,
    com_mt: Vec<HashDigest>,
  ) -> (FieldElement, bool) {
    println!("open_and_verify_multi");
    assert_eq!(n % COLUMN_SIZE, 0);
    //tensor product of r0 otimes r1
    let mut r0: [FieldElement; 128] = [FE_ZERO; COLUMN_SIZE];
    let mut r1 = vec![FE_ZERO; n / COLUMN_SIZE];

    let mut log_column_size = 0;

    loop {
      if (1 << log_column_size) == COLUMN_SIZE {
        break;
      }
      log_column_size += 1;
    }

    dfs(&mut r0, r, 0, FE_REAL_ONE);
    dfs(&mut r1, &r[log_column_size..], 0, FE_REAL_ONE);

    self.tensor_product_protocol(&r0, &r1, n, com_mt)
  }
}
fn smallest_pow2_larger_or_equal_to(x: usize) -> usize {
  for i in 0..32 {
    if (1 << i) >= x {
      return 1 << i;
    }
  }
  panic!();
}

fn dfs(dst: &mut [FieldElement], r: &[FieldElement], depth: usize, val: FieldElement) {
  if dst.len() == 1 {
    dst[0] = val;
  } else {
    let (left, right) = dst.split_at_mut(dst.len() / 2);
    let one_minus_r = FE_REAL_ONE - r[depth];
    dfs(left, r, depth + 1, val * one_minus_r);
    dfs(right, r, depth, val * r[depth]);
  }
}
