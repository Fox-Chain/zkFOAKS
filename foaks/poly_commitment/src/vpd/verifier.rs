use std::mem;
use std::time::Instant;

use infrastructure::{
  constants::{LOG_SLICE_NUMBER, RS_CODE_RATE, SLICE_NUMBER},
  my_hash::{self, HashDigest},
};
use infrastructure::merkle_tree::create_tree;
use infrastructure::my_hash::my_hash;
use prime_field::FieldElement;

use crate::LdtCommitment;
use crate::vpd::fri::FRIContext;

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

  let mut current_hash: HashDigest = *merkle_path.last().expect("Merkle path is empty");

  let mut data: [HashDigest; 2];
  // don't mutate the current_hash, this is the output of the loop following

  for merkle_path_item in merkle_path.iter().take(len - 1) {
    data = [current_hash, *merkle_path_item];

    if (pow & 1_u128) != 0 {
      data = [*merkle_path_item, current_hash];
    }
    pow /= 2;

    current_hash = my_hash(data);
  }

  data = unsafe { mem::zeroed() };

  let mut value_hash = HashDigest::new();

  for value in values {
    data[0] = HashDigest::memcpy_from_field_elements([value.0, value.1]);
    data[1] = value_hash;
    value_hash = my_hash::my_hash(data);
  }

  hash_digest == current_hash && Some(&value_hash) == merkle_path.last()
}

impl FRIContext {
  /// Given fold parameter r, return the root of the merkle tree of next level.
  pub fn commit_phase_step(&mut self, r: FieldElement, slice_count: usize) -> HashDigest {
    let nxt_witness_size = (1 << self.log_current_witness_size_per_slice) / 2;
    if self.cpd.rs_codeword[self.current_step_no].is_empty() {
      self.cpd.rs_codeword[self.current_step_no] =
        vec![FieldElement::default(); nxt_witness_size * slice_count];
    }

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
      let pos = usize::min(qual_res_0, qual_res_1);

      let inv_mu = self.l_group[((1 << self.log_current_witness_size_per_slice) - i)
        & ((1 << self.log_current_witness_size_per_slice) - 1)];

      for j in 0..SLICE_NUMBER {
        let real_pos = previous_witness_mapping[(pos) << LOG_SLICE_NUMBER | j];
        assert!((i << LOG_SLICE_NUMBER | j) < nxt_witness_size * slice_count);
        self.cpd.rs_codeword[self.current_step_no][i << LOG_SLICE_NUMBER | j] = inv_2
          * ((previous_witness[real_pos] + previous_witness[real_pos | 1])
            + inv_mu * r * (previous_witness[real_pos] - previous_witness[real_pos | 1]));
      }
    }

    for i in 0..nxt_witness_size {
      self.l_group[i] = self.l_group[i * 2];
    }

    let mut tmp: Vec<FieldElement> =
      vec![FieldElement::new_random(); nxt_witness_size * slice_count];

    self.cpd.rs_codeword_mapping[self.current_step_no] = vec![0; nxt_witness_size * slice_count];

    let nxt_witness_size_div_2 = nxt_witness_size / 2;

    for i in 0..nxt_witness_size_div_2 {
      for j in 0..SLICE_NUMBER {
        let a = i << LOG_SLICE_NUMBER | j;
        let b = (i + nxt_witness_size_div_2) << LOG_SLICE_NUMBER | j;
        let c = i << log_leaf_size | (j << 1);
        let d = c | 1;

        let rs_codeword_mapping = &mut self.cpd.rs_codeword_mapping;
        let rs_codeword = &self.cpd.rs_codeword;

        rs_codeword_mapping[self.current_step_no][a] = c;
        rs_codeword_mapping[self.current_step_no][b] = c;

        tmp[c] = rs_codeword[self.current_step_no][a];
        tmp[d] = rs_codeword[self.current_step_no][b];

        assert!(a < nxt_witness_size * SLICE_NUMBER);
        assert!(b < nxt_witness_size * SLICE_NUMBER);
        assert!(c < nxt_witness_size * SLICE_NUMBER);
        assert!(d < nxt_witness_size * SLICE_NUMBER);
      }
    }
    self.cpd.rs_codeword[self.current_step_no] = tmp;

    self.visited[self.current_step_no] = vec![false; nxt_witness_size * 4 * slice_count];

    let mut htmp: HashDigest;
    let mut hash_val: Vec<HashDigest> = vec![HashDigest::default(); nxt_witness_size / 2];

    for (i, hash_val_item) in hash_val.iter_mut().enumerate().take(nxt_witness_size / 2) {
      let mut data = [HashDigest::default(), HashDigest::default()];
      htmp = HashDigest::default();
      for j in 0..(1 << LOG_SLICE_NUMBER) {
        let c = (i) << log_leaf_size | (j << 1);
        let d = (i) << log_leaf_size | (j << 1) | 1;

        let data_ele = [
          self.cpd.rs_codeword[self.current_step_no][c],
          self.cpd.rs_codeword[self.current_step_no][d],
        ];

        data[0] = HashDigest::memcpy_from_field_elements(data_ele);
        data[1] = htmp;
        htmp = my_hash(data);
      }
      *hash_val_item = htmp;
    }

    let current_step_no = self.cpd.merkle[self.current_step_no].clone();
    create_tree(
      &hash_val,
      self.cpd.merkle[self.current_step_no].as_mut(),
      current_step_no.is_empty(),
    );

    self.cpd.merkle_size[self.current_step_no] = nxt_witness_size / 2;
    self.log_current_witness_size_per_slice -= 1;

    self.current_step_no += 1;
    self.cpd.merkle[self.current_step_no - 1][1] // since we increment current_step_no up there
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
    let mut ret: Vec<HashDigest> = Vec::with_capacity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
    let mut randomness: Vec<FieldElement> =
      Vec::with_capacity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

    let mut ptr = 0;

    while codeword_size > 1 << RS_CODE_RATE {
      assert!(ptr < log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

      randomness.push(FieldElement::new_random());

      ret.push(self.commit_phase_step(randomness[ptr], slice_count));
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
