use std::mem;
use std::time::Instant;

use infrastructure::merkle_tree::{create_tree, hash_single_field_element};
use infrastructure::my_hash::my_hash;
use infrastructure::{
  constants::{LOG_SLICE_NUMBER, RS_CODE_RATE, SLICE_NUMBER},
  my_hash::{self, HashDigest},
};
use prime_field::FieldElement;

use crate::vpd::fri::FRIContext;
use crate::LdtCommitment;

pub fn verify_merkle(
  hash_digest: HashDigest,
  merkle_path: &Vec<HashDigest>,
  len: usize,
  pow: u128,
  values: &Vec<(FieldElement, FieldElement)>,
) -> bool {
  // We need to make sure the len is always smaller than the size of merklePath.
  assert!(merkle_path.len() >= len);

  let mut pow = pow;
  //Todo: Print hash_digest, merkle_path

  // println!("hash: {:?}", hash_digest);

  // for i in 0..merkle_path.len() {
  //   println!("merkle {i}: {:?}", merkle_path[i]);
  // }

  // println!("len: {}, pow: {}", len, pow);
  // for i in 0..values.len() {
  //   println!(
  //     "values[{}].0.real:{}, img:{}",
  //     i, values[i].0.real, values[i].0.img
  //   );
  //   println!(
  //     "values[{}].1.real:{}, img:{}",
  //     i, values[i].1.real, values[i].1.img
  //   );
  // }
  //panic!("stop here");
  let mut current_hash: HashDigest = *merkle_path.last().unwrap();

  let mut data: [HashDigest; 2];
  // don't mutate the current_hash, this is the output of the loop following

  for i in 0..(len - 1) {
    data = [current_hash, merkle_path[i]];

    if (pow & 1_u128) != 0 {
      data = [merkle_path[i], current_hash];
    }
    pow /= 2;

    current_hash = my_hash(data);
  }

  data = unsafe { mem::zeroed() };

  let mut value_hash = HashDigest::new();

  unsafe {
    for value in values {
      let data_ele = [value.0, value.1];
      let src = std::ptr::addr_of!(data_ele) as *const HashDigest;
      let dst = std::ptr::addr_of_mut!(data[0]);
      std::ptr::copy_nonoverlapping(src, dst, 1);
      data[1] = value_hash;
      value_hash = my_hash::my_hash(data);
    }
  }

  println!(
    "value_hash: {:?}\nmerkle: {:?}",
    value_hash,
    merkle_path.last()
  );

  hash_digest == current_hash && Some(&value_hash) == merkle_path.last()
}

impl FRIContext {
  /// Request two values w^{pow0} and w^{pow1}, with merkle tree proof, where w
  /// is the root of unity and w^{pow0} and w^{pow1} are quad residue. Repeat
  /// ldt_repeat_num times, storing all result in vector.
  pub fn request_init_value_with_merkle(
    &mut self,
    pow_0: usize,
    pow_1: usize,
    // new_size: &i64,
    oracle_indicator: usize,
  ) -> (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>) {
    // we swap pow_0 and pow_1 when pow_0 > pow_1
    let (pow_0, pow_1) = if pow_0 > pow_1 {
      (pow_1, pow_0)
    } else {
      (pow_0, pow_1)
    };

    assert_eq!(
      pow_0 + (1 << self.log_current_witness_size_per_slice) / 2,
      pow_1
    );

    let mut value: Vec<(FieldElement, FieldElement)> = vec![];
    let log_leaf_size = LOG_SLICE_NUMBER + 1;

    //let mut new_size: usize = 0;
    for i in 0..SLICE_NUMBER {
      let element_1 =
        self.witness_rs_codeword_interleaved[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 0];

      let element_2 =
        self.witness_rs_codeword_interleaved[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 1];

      value.push((element_1, element_2));

      if !self.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 0] {
        self.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 0] = true;
      }
      if !self.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 1] {
        self.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 1] = true;
      }
      //new_size += mem::size_of::<FieldElement>();
    }

    let depth = self.log_current_witness_size_per_slice - 1;
    let mut com_hhash: Vec<HashDigest> = Vec::with_capacity(depth);

    // minus 1 since each leaf have 2 values (qual resi)
    let mut pos = pow_0 + (1 << (self.log_current_witness_size_per_slice - 1));

    let mut test_hash = self.witness_merkle[oracle_indicator][pos];
    com_hhash[depth] = test_hash;

    for i in 0..depth {
      // if !self.visited_init[oracle_indicator][pos ^ 1] {
      //   new_size += mem::size_of::<HashDigest>();
      // }
      self.visited_init[oracle_indicator][pos] = true;
      self.visited_init[oracle_indicator][pos ^ 1] = true;

      let data = if (pos & 1) == 1 {
        [self.witness_merkle[oracle_indicator][pos ^ 1], test_hash]
      } else {
        [test_hash, self.witness_merkle[oracle_indicator][pos ^ 1]]
      };
      test_hash = my_hash::my_hash(data);

      com_hhash[i] = self.witness_merkle[oracle_indicator][pos ^ 1];
      pos /= 2;

      assert_eq!(test_hash, self.witness_merkle[oracle_indicator][pos]);
    }
    assert_eq!(pos, 1);
    (value, com_hhash)
  }

  /// Given fold parameter r, return the root of the merkle tree of next level.
  pub fn commit_phase_step(&mut self, r: FieldElement, slice_count: usize) -> HashDigest {
    let nxt_witness_size = (1 << self.log_current_witness_size_per_slice) / 2;
    if self.cpd.rs_codeword[self.current_step_no].is_empty() {
      self.cpd.rs_codeword[self.current_step_no] =
        vec![FieldElement::default(); nxt_witness_size * slice_count];
    }

    //let mut previous_witness: Vec<FieldElement> = vec![];
    //let mut previous_witness_mapping: Vec<usize> = vec![];

    let (previous_witness, previous_witness_mapping) = match self.current_step_no {
      0 => (
        self.virtual_oracle_witness.clone(),
        self.virtual_oracle_witness_mapping.clone(),
      ),
      _ => (
        self.cpd.rs_codeword[self.current_step_no - 1].clone(),
        self.cpd.rs_codeword_mapping[self.current_step_no - 1].clone(),
      ),
    };

    let inv_2 = FieldElement::from_real(2).inverse();

    let log_leaf_size = LOG_SLICE_NUMBER + 1;

    for i in 0..nxt_witness_size {
      let qual_res_0 = i;
      let qual_res_1 = ((1 << (self.log_current_witness_size_per_slice - 1)) + i) / 2;
      let pos = usize::min(qual_res_0, qual_res_1 as usize);

      let inv_mu = self.l_group[((1 << self.log_current_witness_size_per_slice) - i)
        & ((1 << self.log_current_witness_size_per_slice) - 1)];

      for j in 0..SLICE_NUMBER {
        let real_pos = previous_witness_mapping[(pos) << LOG_SLICE_NUMBER | j];
        assert!((i << LOG_SLICE_NUMBER | j) < nxt_witness_size * slice_count);
        // we should check this since the original code has BUG comment
        self.cpd.rs_codeword[self.current_step_no][i << LOG_SLICE_NUMBER | j] = inv_2
          * ((previous_witness[real_pos] + previous_witness[real_pos | 1])
            + inv_mu * r * (previous_witness[real_pos] - previous_witness[real_pos | 1]));
      }
    }

    for i in 0..nxt_witness_size {
      self.l_group[i] = self.l_group[i * 2];
    }

    // we assume poly_commit::slice_count is (1 << SLICE_NUMBER) here
    // NOTE: this assumption is solved by using slice_count from context
    let mut tmp: Vec<FieldElement> =
      vec![FieldElement::new_random(); nxt_witness_size * slice_count];
    println!(
      "nxt_witness_size: {}, slice_count: {}, len;{}",
      nxt_witness_size,
      slice_count,
      tmp.len()
    );
    self.cpd.rs_codeword_mapping[self.current_step_no] = vec![0; nxt_witness_size * slice_count];

    for i in 0..nxt_witness_size / 2 {
      for j in 0..SLICE_NUMBER {
        let a = i << LOG_SLICE_NUMBER | j;
        let b = (i + nxt_witness_size / 2) << LOG_SLICE_NUMBER | j;
        let c = (i) << log_leaf_size | (j << 1) | 0;
        let d = (i) << log_leaf_size | (j << 1) | 1;
        self.cpd.rs_codeword_mapping[self.current_step_no][a] = (i) << log_leaf_size | (j << 1) | 0;
        self.cpd.rs_codeword_mapping[self.current_step_no][b] = (i) << log_leaf_size | (j << 1) | 0;

        tmp[c] = self.cpd.rs_codeword[self.current_step_no][i << LOG_SLICE_NUMBER | j];
        tmp[d] = self.cpd.rs_codeword[self.current_step_no]
          [(i + nxt_witness_size / 2) << LOG_SLICE_NUMBER | j];

        // println!(
        //   "a:{}, tmp[{}].real:{}, img:{} ",
        //   a, c, tmp[c].real, tmp[c].img
        // );
        // println!(
        //   "b:{}, tmp[{}].real:{}, img:{} ",
        //   b, d, tmp[d].real, tmp[d].img
        // );
        assert!(a < nxt_witness_size * SLICE_NUMBER);
        assert!(b < nxt_witness_size * SLICE_NUMBER);
        assert!(c < nxt_witness_size * SLICE_NUMBER);
        assert!(d < nxt_witness_size * SLICE_NUMBER);
      }
    }
    //panic!("stop here");
    self.cpd.rs_codeword[self.current_step_no] = tmp;

    self.visited[self.current_step_no] = vec![false; nxt_witness_size * 4 * slice_count];

    let mut htmp: HashDigest = HashDigest::default();
    let mut hash_val: Vec<HashDigest> = vec![HashDigest::default(); nxt_witness_size / 2];

    unsafe {
      for i in 0..nxt_witness_size / 2 {
        let mut data = [HashDigest::default(), HashDigest::default()];
        for j in 0..(1 << LOG_SLICE_NUMBER) {
          let c = (i) << log_leaf_size | (j << 1) | 0;
          let d = (i) << log_leaf_size | (j << 1) | 1;

          let data_ele = [
            self.cpd.rs_codeword[self.current_step_no][c],
            self.cpd.rs_codeword[self.current_step_no][d],
          ];

          data[0] = hash_single_field_element(data_ele[0]); // TODO check
          data[1] = htmp;
          htmp = my_hash(data);
        }
        hash_val[i] = htmp;
      }

      // write merkle tree to self.cpd.merkle[self.current_step_no]
      let current_step_no = self.cpd.merkle[self.current_step_no].clone();
      create_tree(
        hash_val,
        nxt_witness_size / 2,
        self.cpd.merkle[self.current_step_no].as_mut(),
        //Some(std::mem::size_of::<HashDigest>()),
        Some(current_step_no.is_empty()),
      )
    }

    self.cpd.merkle_size[self.current_step_no] = nxt_witness_size / 2;
    self.log_current_witness_size_per_slice -= 1;

    self.current_step_no += 1;
    self.cpd.merkle[self.current_step_no - 1][1] // since we increment
                                                 // current_step_no up there
  }

  /// Return the final rs code since it is only constant size
  pub fn commit_phase_final(&self) -> Vec<FieldElement> {
    self.cpd.rs_codeword[self.current_step_no - 1].clone()
  }

  pub fn commit_phase(&mut self, log_length: usize, slice_count: usize) -> LdtCommitment {
    let t0 = Instant::now();

    let log_current_witness_size_per_slice_cp = self.log_current_witness_size_per_slice;
    let mut codeword_size = 1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
    // repeat until the codeword is constant
    let mut ret: Vec<HashDigest> =
      vec![HashDigest::default(); log_length + RS_CODE_RATE - LOG_SLICE_NUMBER];
    let mut randomness: Vec<FieldElement> =
      vec![FieldElement::default(); log_length + RS_CODE_RATE - LOG_SLICE_NUMBER];

    let mut ptr = 0;
    while codeword_size > 1 << RS_CODE_RATE {
      assert!(ptr < log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
      randomness[ptr] = FieldElement::new_random();
      ret[ptr] = self.commit_phase_step(randomness[ptr], slice_count);
      codeword_size /= 2;
      ptr += 1;
    }

    self.log_current_witness_size_per_slice = log_current_witness_size_per_slice_cp;

    let com = LdtCommitment {
      commitment_hash: ret,
      final_rs_code: self.commit_phase_final(),
      randomness,
      mx_depth: ptr,
    };

    println!("FRI commit time {}", t0.elapsed().as_secs_f64());

    com
  }
}
