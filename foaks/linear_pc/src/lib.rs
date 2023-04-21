use std::collections::HashMap;
use std::default;
use std::mem::size_of_val;
use std::time::Instant;

use infrastructure::merkle_tree::{self, create_tree};
use infrastructure::my_hash::HashDigest;
use infrastructure::utility::my_log;
use linear_code::linear_code_encode::Graph;
use linear_code::linear_code_encode::LinearCodeEncodeContext;
use linear_code::parameter::{ALPHA, CN, COLUMN_SIZE, DISTANCE_THRESHOLD, DN, R, TARGET_DISTANCE};
use linear_gkr::circuit_fast_track::{Gate, Layer};
use linear_gkr::prover::ZkProver;
use linear_gkr::verifier::ZkVerifier;
use prime_field::FieldElement;

//commit
pub struct LinearPC {
  encoded_codeword: Vec<Vec<FieldElement>>,
  coef: Vec<Vec<FieldElement>>,
  codeword_size: Vec<u64>,
  mt: Vec<HashDigest>,
  prover: ZkProver,
  verifier: ZkVerifier,
  lcectx: LinearCodeEncodeContext,
  gates_count: HashMap<u64, u64>,
}

impl LinearPC {
  pub unsafe fn commit(&mut self, src: Vec<FieldElement>, n: u64) -> Vec<HashDigest> {
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

      self.codeword_size[i] = self.lcectx.encode(
        (src[i * (n / COLUMN_SIZE) as usize..]).to_vec(),
        &mut self.encoded_codeword[i],
        n / COLUMN_SIZE,
        Some(0),
      );
    }

    for i in 0..(n / COLUMN_SIZE * 2) as usize {
      stash[i] = HashDigest::default();
      for j in 0..(COLUMN_SIZE / 2) as usize {
        stash[i] = merkle_tree::hash_double_field_element_merkle_damgard(
          self.encoded_codeword[2 * j][i],
          self.encoded_codeword[2 * j + 1][i],
          stash[i],
        );
      }
    }
    merkle_tree::create_tree(
      stash,
      (n / COLUMN_SIZE * 2).try_into().unwrap(),
      &mut self.mt,
      //Some(std::mem::size_of::<HashDigest>()),
      Some(true),
    );
    self.mt.clone()
  }

  fn generate_circuit(
    &mut self,
    query: &mut Vec<u64>,
    n: u64,
    query_count: u64,
    input: Vec<FieldElement>,
  ) {
    query.sort();
    self.prepare_gates_count(query.to_vec(), n, query_count);
    println!("Depth {}", self.gates_count.len());
    assert_eq!((1 << my_log(n as usize).unwrap()), n);

    self.verifier.aritmetic_circuit.inputs = vec![FieldElement::zero(); n as usize];
    self.verifier.aritmetic_circuit.total_depth = self.gates_count.len() + 1;
    self.verifier.aritmetic_circuit.circuit =
      vec![Layer::default(); self.verifier.aritmetic_circuit.total_depth];
    self.verifier.aritmetic_circuit.circuit[0].bit_length = my_log(n.try_into().unwrap()).unwrap();
    self.verifier.aritmetic_circuit.circuit[0].gates =
      vec![Gate::new(); 1 << self.verifier.aritmetic_circuit.circuit[0].bit_length];

    for i in 0..self.gates_count.len() {
      self.verifier.aritmetic_circuit.circuit[i + 1].bit_length = my_log(
        smallest_pow2_larger_or_equal_to(*self.gates_count.get(&(i as u64)).unwrap())
          .try_into()
          .unwrap(),
      )
      .unwrap();
      self.verifier.aritmetic_circuit.circuit[i + 1].gates =
        vec![Gate::new(); 1 << self.verifier.aritmetic_circuit.circuit[i + 1].bit_length];
    }
    self.verifier.aritmetic_circuit.circuit[self.gates_count.len() + 1].bit_length = my_log(
      smallest_pow2_larger_or_equal_to(query_count)
        .try_into()
        .unwrap(),
    )
    .unwrap();
    self.verifier.aritmetic_circuit.circuit[self.gates_count.len() + 1].gates =
      vec![
        Gate::new();
        1 << self.verifier.aritmetic_circuit.circuit[self.gates_count.len() + 1].bit_length
      ];

    for i in 0..n as usize {
      self.verifier.aritmetic_circuit.inputs[i] = input[i];
      self.verifier.aritmetic_circuit.circuit[0].gates[i] = Gate::from_params(3, 0, 0);
      self.verifier.aritmetic_circuit.circuit[1].gates[i] = Gate::from_params(4, i, 0);
    }
    //Todo: improve gate_types::input with constant values
    for i in 0..n as usize {
      self.verifier.aritmetic_circuit.circuit[2].gates[i] = Gate::from_params(10, 0, 0);
    }

    self.verifier.aritmetic_circuit.circuit[2].src_expander_c_mempool =
      vec![0; (CN * self.lcectx.c[0].l).try_into().unwrap()];
    self.verifier.aritmetic_circuit.circuit[2].weight_expander_c_mempool =
      vec![FieldElement::zero(); (CN * self.lcectx.c[0].l).try_into().unwrap()];
    let mut c_mempool_ptr = 0;
    let mut d_mempool_ptr = 0;
    for i in 0..self.lcectx.c[0].r as usize {
      self.verifier.aritmetic_circuit.circuit[2].gates[i + (n as usize)] =
        Gate::from_params(14, 0, 0);
      self.verifier.aritmetic_circuit.circuit[2].gates[i + (n as usize)].parameter_length =
        self.lcectx.c[0].r_neighbor[i].len();
      //Todo: Check if this is correct
      self.verifier.aritmetic_circuit.circuit[2].gates[i + (n as usize)].src =
        self.verifier.aritmetic_circuit.circuit[2].src_expander_c_mempool[c_mempool_ptr..].to_vec();
      self.verifier.aritmetic_circuit.circuit[2].gates[i + (n as usize)].weight =
        self.verifier.aritmetic_circuit.circuit[2].weight_expander_c_mempool[c_mempool_ptr..]
          .to_vec();
      c_mempool_ptr += self.lcectx.c[0].r_neighbor[i].len();
    }

    unimplemented!();
  }
  pub unsafe fn tensor_product_protocol(
    &mut self,
    r0: Vec<FieldElement>,
    r1: Vec<FieldElement>,
    size_r0: u64,
    size_r1: u64,
    n: u64,
    com_mt: Vec<HashDigest>,
    linear_code_encode: &mut LinearCodeEncodeContext,
  ) //-> (FieldElement, bool)
  {
    //let mut verification_time = 0.0;
    assert_eq!(size_r0 * size_r1, n);

    let mut visited_com = vec![false; (n / COLUMN_SIZE * 4).try_into().unwrap()];
    let mut visited_combined_com = vec![false; (n / COLUMN_SIZE * 4).try_into().unwrap()];

    let mut proof_size = 0;
    // Todo: check log2
    let query_count = (-128.0 / (fast_math::log2(1f32 - TARGET_DISTANCE))) as u64;
    println!("Query count: {}", query_count);
    println!("Column size: {}", COLUMN_SIZE);
    println!("Number of merkle pathes: {}", query_count);
    println!("Number of field elements: {}", query_count * COLUMN_SIZE);

    //prover construct the combined codeword

    let mut combined_codeword = vec![FieldElement::zero(); self.codeword_size[0] as usize];
    let mut combined_codeword_hash =
      vec![HashDigest::default(); (n / COLUMN_SIZE * 2).try_into().unwrap()];
    let mut combined_codeword_mt =
      vec![HashDigest::default(); (n / COLUMN_SIZE * 4).try_into().unwrap()];
    for i in 0..COLUMN_SIZE as usize {
      for j in 0..self.codeword_size[0] as usize {
        combined_codeword[j] = combined_codeword[j] + r0[i] * self.encoded_codeword[i][j];
      }
    }

    let zero = FieldElement::zero();
    for i in 0..(n / COLUMN_SIZE * 2) as usize {
      if i < self.codeword_size[0] as usize {
        combined_codeword_hash[i] = merkle_tree::hash_single_field_element(combined_codeword[i]);
      } else {
        combined_codeword_hash[i] = merkle_tree::hash_single_field_element(zero);
      }
    }

    //merkle commit to combined_codeword
    create_tree(
      combined_codeword_hash,
      (n / COLUMN_SIZE * 2).try_into().unwrap(),
      &mut combined_codeword_mt,
      Some(false),
    );

    //prover construct the combined original message
    let mut combined_message = vec![FieldElement::zero(); n.try_into().unwrap()];
    for i in 0..COLUMN_SIZE as usize {
      for j in 0..self.codeword_size[0] as usize {
        combined_message[j] = combined_message[j] + r0[i] * self.coef[i][j];
      }
    }

    //check for encode
    {
      let mut test_codeword = vec![FieldElement::zero(); (n / COLUMN_SIZE * 2).try_into().unwrap()];
      let test_codeword_size = linear_code_encode.encode(
        combined_message.clone(),
        &mut test_codeword,
        n / COLUMN_SIZE,
        None,
      );
      assert_eq!(test_codeword_size, self.codeword_size[0]);
      for i in 0..test_codeword_size as usize {
        assert_eq!(test_codeword[i], combined_codeword[i]);
      }
    }

    //verifier random check columns
    let v_t0 = Instant::now();

    for i in 0..query_count {
      let q = rand::random::<usize>() % self.codeword_size[0] as usize;
      let mut sum = FieldElement::zero();
      for j in 0..COLUMN_SIZE as usize {
        sum = sum + r0[j] * self.encoded_codeword[j][q];
      }
      proof_size += std::mem::size_of::<FieldElement>() * COLUMN_SIZE as usize;

      //calc hash
      let mut column_hash = HashDigest::default();

      for j in 0..COLUMN_SIZE as usize / 2 {
        column_hash = merkle_tree::hash_double_field_element_merkle_damgard(
          self.encoded_codeword[2 * j][q],
          self.encoded_codeword[2 * j + 1][q],
          column_hash,
        );
      }

      assert!(merkle_tree::verify_claim(
        com_mt[1].clone(),
        com_mt.clone(),
        &mut column_hash,
        q,
        (n / COLUMN_SIZE * 2).try_into().unwrap(),
        &mut visited_com,
        &mut proof_size,
      ));
      assert!(merkle_tree::verify_claim(
        combined_codeword_mt[1].clone(),
        combined_codeword_mt.clone(),
        &mut merkle_tree::hash_single_field_element(combined_codeword[q]),
        q,
        (n / COLUMN_SIZE * 2).try_into().unwrap(),
        &mut visited_combined_com,
        &mut proof_size,
      ));

      assert_eq!(sum, combined_codeword[q]);
      //add merkle tree open
    }
    let time_span = v_t0.elapsed();
    //verification_time += time_span.as_secs_f64();
    let verification_time = time_span.as_secs_f64();

    // setup code-switching
    let mut answer = FieldElement::zero();
    for i in 0..(n / COLUMN_SIZE) as usize {
      answer = answer + r1[i] * combined_message[i];
    }

    // prover commit private input

    // verifier samples query
    let mut q = vec![0u64; query_count.try_into().unwrap()];
    for i in 0..query_count as usize {
      q[i] = rand::random::<u64>() % self.codeword_size[0];
    }
    // generate circuit

    // bool result = true;
    // p.evaluate();
    unimplemented!();
  }

  fn prepare_gates_count(&mut self, query: Vec<u64>, n: u64, query_count: u64) {
    //long long query_ptr = 0;
    // input layer
    self.gates_count.insert(0, n);
    // expander part
    self.gates_count.insert(1, n + self.lcectx.c[0].r);
    let output_depth_output_size = self.prepare_enc_count(self.lcectx.c[0].r, n, 1);

    self.gates_count.insert(
      output_depth_output_size.0 + 1,
      n + output_depth_output_size.1 + self.lcectx.d[0].r,
    );
    self
      .gates_count
      .insert(output_depth_output_size.0 + 2, query_count);
  }

  fn prepare_enc_count(
    &mut self,
    input_size: u64,
    output_size_so_far: u64,
    depth: u64,
  ) -> (u64, u64) {
    if input_size <= DISTANCE_THRESHOLD.try_into().unwrap() {
      return (depth, output_size_so_far);
    }
    // output
    self.gates_count.insert(
      depth + 1,
      output_size_so_far + input_size + self.lcectx.c[depth as usize].r,
    );
    let output_depth_output_size = self.prepare_enc_count(
      self.lcectx.c[depth as usize].r,
      output_size_so_far + input_size,
      depth + 1,
    );
    self.gates_count.insert(
      output_depth_output_size.0 + 1,
      output_depth_output_size.1 + self.lcectx.d[depth as usize].r,
    );
    (
      output_depth_output_size.0 + 1,
      self
        .gates_count
        .get(&(output_depth_output_size.0 + 1))
        .copied()
        .unwrap(),
    )
  }
}

fn open_and_verify(x: FieldElement, n: u64, com_mt: Vec<HashDigest>) //-> (FieldElement, bool)
{
  assert_eq!(n % COLUMN_SIZE, 0);
  //tensor product of r0 otimes r1
  let mut r0 = vec![FieldElement::zero(); COLUMN_SIZE as usize];
  let mut r1 = vec![FieldElement::zero(); COLUMN_SIZE as usize];

  let x_n = FieldElement::fast_pow(x, (n / COLUMN_SIZE).into());
  r0[0] = FieldElement::real_one();
  for j in 1..COLUMN_SIZE as usize {
    r0[j] = r0[j - 1] * x_n;
  }
  r1[0] = FieldElement::real_one();
  for j in 1..(n / COLUMN_SIZE) as usize {
    r1[j] = r1[j - 1] * x;
  }

  //tensor_product_protocol(r0, r1, COLUMN_SIZE, n / COLUMN_SIZE, n, com_mt);
}

fn smallest_pow2_larger_or_equal_to(x: u64) -> u64 {
  for i in 0..32 {
    if (1 << i) >= x {
      return 1 << i;
    }
  }
  panic!();
}

// enum GateTypes
// {
// 	Add = 0,
// 	Mult = 1,
// 	Dummy = 2,
// 	Sum = 5,
// 	ExpSum = 12,
// 	DirectRelay = 4,
// 	NotGate = 6,
// 	Minus = 7,
// 	XorGate = 8,
// 	BitTest = 13,
// 	Relay = 10,
// 	CustomLinearComb = 14,
// 	Input = 3isize
// }
