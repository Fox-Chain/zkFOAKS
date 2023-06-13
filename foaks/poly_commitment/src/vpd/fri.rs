#[allow(unused)]
use crate::PolyCommitContext;
use infrastructure::{
  constants::{LOG_SLICE_NUMBER, MAX_BIT_LENGTH, MAX_FRI_DEPTH, RS_CODE_RATE, SLICE_NUMBER},
  merkle_tree,
  my_hash::{my_hash, HashDigest},
};
use prime_field::FieldElement;
use std::{mem::size_of, time, usize, vec};

pub type TripleVec<'a> = (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);

#[derive(Default, Debug, Clone)]
pub struct CommitPhaseData {
  pub merkle: [Vec<HashDigest>; MAX_FRI_DEPTH],
  pub merkle_size: [usize; MAX_FRI_DEPTH],
  pub rs_codeword: [Vec<FieldElement>; MAX_FRI_DEPTH],
  pub poly_coef: [Vec<FieldElement>; MAX_FRI_DEPTH],
  pub rs_codeword_mapping: [Vec<usize>; MAX_FRI_DEPTH],
}

// namespace fri
impl CommitPhaseData {
  pub fn new() -> Self { Default::default() }

  pub fn delete_self(&mut self) { std::mem::take(self); }
}

#[derive(Debug, Clone)]
pub struct FieldElement64([Vec<FieldElement>; SLICE_NUMBER]);

impl Default for FieldElement64 {
  fn default() -> Self {
    const EMPTY_VEC: Vec<FieldElement> = Vec::new();
    FieldElement64([EMPTY_VEC; SLICE_NUMBER])
  }
}

#[derive(Debug, Clone)]
pub struct Mapping64([Vec<usize>; SLICE_NUMBER]);

impl Default for Mapping64 {
  fn default() -> Self {
    const EMPTY_VEC: Vec<usize> = Vec::new();
    Mapping64([EMPTY_VEC; SLICE_NUMBER])
  }
}
// Todo: Create a good init function for this
#[derive(Default, Debug, Clone)]
pub struct FRIContext {
  pub log_current_witness_size_per_slice: usize,
  pub witness_bit_length_per_slice: i64,
  pub current_step_no: usize,
  pub cpd: CommitPhaseData,
  pub fri_timer: f64,
  pub witness_merkle: [Vec<HashDigest>; 2],
  pub witness_rs_codeword_before_arrange: Vec<Vec<Vec<FieldElement>>>,
  pub witness_rs_codeword_interleaved: [Vec<FieldElement>; 2],
  pub witness_rs_mapping: Vec<Vec<Vec<usize>>>,
  pub l_group: Vec<FieldElement>,
  pub visited: [Vec<bool>; MAX_BIT_LENGTH],
  pub visited_init: [Vec<bool>; 2],
  pub visited_witness: [Vec<bool>; 2],
  pub virtual_oracle_witness: Vec<FieldElement>,
  pub virtual_oracle_witness_mapping: Vec<usize>,

  pub r_extended: Vec<FieldElement>,
  pub leaf_hash: [Vec<HashDigest>; 2],
}

impl FRIContext {
  pub fn new() -> Self {
    Self {
      witness_rs_codeword_before_arrange: vec![vec![Vec::new(); SLICE_NUMBER]; 2],
      //witness_rs_mapping: vec![vec![vec![0; SLICE_NUMBER]; 2]],
      ..Default::default()
    }
  }
}
/// Given private input, calculate the first oracle commitment
pub fn request_init_commit(
  FRIContext {
    log_current_witness_size_per_slice,
    witness_bit_length_per_slice,
    current_step_no,
    cpd,
    fri_timer,
    witness_merkle,
    witness_rs_codeword_before_arrange,
    witness_rs_codeword_interleaved,
    witness_rs_mapping,
    l_group,
    visited,
    visited_init,
    visited_witness,
    virtual_oracle_witness,
    virtual_oracle_witness_mapping,
    r_extended,
    leaf_hash,
  }: &mut FRIContext,
  PolyCommitContext {
    slice_size,
    slice_count,
    l_eval,
    h_eval_arr,
    ..
  }: &PolyCommitContext,
  bit_len: usize,
  oracle_indicator: usize,
) -> HashDigest {
  assert_eq!(
    slice_size * slice_count,
    (1 << (bit_len + RS_CODE_RATE - LOG_SLICE_NUMBER)) * (1 << LOG_SLICE_NUMBER)
  );

  *fri_timer = 0.;

  std::mem::take(&mut witness_merkle[oracle_indicator]);
  std::mem::take(&mut witness_rs_codeword_interleaved[oracle_indicator]);

  *current_step_no = 0;

  assert_eq!(1 << LOG_SLICE_NUMBER, SLICE_NUMBER);
  *log_current_witness_size_per_slice = bit_len + RS_CODE_RATE - LOG_SLICE_NUMBER;
  *witness_bit_length_per_slice = bit_len as i64 - LOG_SLICE_NUMBER as i64;

  let now = time::Instant::now();

  // let sliced_input_length_per_block = 1 << *witness_bit_length_per_slice; No
  // usages
  assert!(*witness_bit_length_per_slice >= 0);
  let mut root_of_unity =
    FieldElement::get_root_of_unity(*log_current_witness_size_per_slice).unwrap();
  if oracle_indicator == 0 {
    //l_group.reserve(1 << *log_current_witness_size_per_slice);
    l_group.push(FieldElement::from_real(1));
    //l_group[0] = FieldElement::from_real(1);

    for i in 1..(1 << *log_current_witness_size_per_slice) {
      //l_group[i] = l_group[i - 1] * root_of_unity;
      l_group.push(l_group[i - 1] * root_of_unity);
    }
    assert_eq!(
      l_group[(1 << *log_current_witness_size_per_slice) - 1] * root_of_unity,
      FieldElement::from_real(1)
    );
  }

  //witness_rs_codeword_interleaved[oracle_indicator].reserve(1 << (bit_len +
  // RS_CODE_RATE));
  witness_rs_codeword_interleaved[oracle_indicator] =
    vec![FieldElement::default(); 1 << (bit_len + RS_CODE_RATE)];

  let log_leaf_size = LOG_SLICE_NUMBER + 1;
  //println!("l_eval {:?}", l_eval[0]);
  //println!("h_eval_arr {:?}", h_eval_arr[0]); TODO find the cause why this is not initialized anywhere
  for i in 0..SLICE_NUMBER {
    assert_eq!(
      (*log_current_witness_size_per_slice - RS_CODE_RATE) as i64,
      *witness_bit_length_per_slice
    );
    root_of_unity =
      FieldElement::get_root_of_unity((*witness_bit_length_per_slice).try_into().unwrap()).unwrap();

    if oracle_indicator == 0 {
      witness_rs_codeword_before_arrange[0][i] = l_eval[i * slice_size..].to_vec();
    } else {
      witness_rs_codeword_before_arrange[1][i] = h_eval_arr[i * slice_size..].to_vec();
    }

    root_of_unity = FieldElement::get_root_of_unity(*log_current_witness_size_per_slice).unwrap();

    //witness_rs_mapping[oracle_indicator][i].reserve(1 <<
    // *log_current_witness_size_per_slice);
    if witness_rs_mapping.len() == 0 {
      for _ in 0..=oracle_indicator + 1 {
        witness_rs_mapping.push(vec![]);
      }
    }
    witness_rs_mapping[oracle_indicator].push(vec![0; 1 << *log_current_witness_size_per_slice]);

    // let a = FieldElement::zero(); No usages
    for j in 0..(1 << (*log_current_witness_size_per_slice - 1)) {
      assert!((j << log_leaf_size | (i << 1) | 1) < (1 << (bit_len + RS_CODE_RATE)));
      assert!((j << log_leaf_size | (i << 1) | 1) < slice_size * slice_count);

      witness_rs_mapping[oracle_indicator][i][j] = j << log_leaf_size | (i << 1) | 0;
      witness_rs_mapping[oracle_indicator][i][j + (1 << *log_current_witness_size_per_slice) / 2] =
        j << log_leaf_size | (i << 1) | 0;

      witness_rs_codeword_interleaved[oracle_indicator][(j) << log_leaf_size | (i << 1) | 0] =
        witness_rs_codeword_before_arrange[oracle_indicator][i][j];
      witness_rs_codeword_interleaved[oracle_indicator][(j) << log_leaf_size | (i << 1) | 1] =
        witness_rs_codeword_before_arrange[oracle_indicator][i]
          [j + (1 << *log_current_witness_size_per_slice) / 2];
    }
  }

  //leaf_hash[oracle_indicator].reserve(1 << (*log_current_witness_size_per_slice
  // - 1));
  leaf_hash[oracle_indicator] =
    vec![HashDigest::default(); 1 << (*log_current_witness_size_per_slice - 1)];

  for i in 0..(1 << (*log_current_witness_size_per_slice - 1)) {
    let mut tmp_hash = HashDigest::new();
    let mut data = [HashDigest::new(), HashDigest::new()];
    // Gian: Propose this way to do  copynonoverlapping
    let mut j = 0;
    let end = 1 << log_leaf_size;
    while j < end {
      unsafe {
        let x = witness_rs_codeword_interleaved[oracle_indicator][(i << log_leaf_size | j)];
        let y = witness_rs_codeword_interleaved[oracle_indicator][(i << log_leaf_size | j) + 1];
        let element = [x, y];

        let src = std::ptr::addr_of!(element) as *const HashDigest;
        let dst = std::ptr::addr_of_mut!(data[0]);
        std::ptr::copy_nonoverlapping(src, dst, 2);
      }

      data[1] = tmp_hash;
      tmp_hash = my_hash(data);

      j += 2;
    }

    leaf_hash[oracle_indicator][i] = tmp_hash;
  }
  merkle_tree::create_tree(
    leaf_hash[oracle_indicator].clone(),
    1 << (*log_current_witness_size_per_slice - 1),
    &mut witness_merkle[oracle_indicator],
    //Some(size_of::<HashDigest>()),
    Some(true),
  );

  visited_init[oracle_indicator] = vec![false; 1 << *log_current_witness_size_per_slice];
  visited_witness[oracle_indicator] = vec![false; 1 << (bit_len + RS_CODE_RATE)];

  *fri_timer = now.elapsed().as_secs_f64();

  witness_merkle[oracle_indicator][1]
}

pub fn request_init_value_with_merkle(
  mut pow_0: usize,
  mut pow_1: usize,
  mut new_size: usize,
  oracle_indicator: usize,
  fri_ctx: &mut FRIContext,
) -> TripleVec {
  if pow_0 > pow_1 {
    std::mem::swap(&mut pow_0, &mut pow_1);
  }

  assert_eq!(
    pow_0 + (1 << fri_ctx.log_current_witness_size_per_slice) / 2,
    pow_1
  );

  let mut value: Vec<(FieldElement, FieldElement)> = vec![];
  let log_leaf_size = LOG_SLICE_NUMBER + 1;
  new_size = 0;

  for i in 0..SLICE_NUMBER {
    value.push((
      fri_ctx.witness_rs_codeword_interleaved[oracle_indicator]
        [pow_0 << log_leaf_size | i << 1 | 0],
      fri_ctx.witness_rs_codeword_interleaved[oracle_indicator]
        [pow_0 << log_leaf_size | i << 1 | 1],
    ));

    /**
    it was `pow_0 << log_leaf_size | i << 1 | 1` but this makes the number be added by 1,
    As C++ returns the number calculated in the left part of this expression `70 << 7 | 0 << 1 | 1 == 3` the assert pass
    but in Rust the equals is actually evaluated.
     */
    assert_eq!(
      pow_0 << log_leaf_size | i << 1,
      fri_ctx.witness_rs_mapping[oracle_indicator][i][pow_1]
    );

    if !fri_ctx.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 0] {
      fri_ctx.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 0] = true;
      new_size += size_of::<FieldElement>();
    }

    if !fri_ctx.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 1] {
      fri_ctx.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 1] = true;
      new_size += size_of::<FieldElement>();
    }
  }

  let depth = fri_ctx.log_current_witness_size_per_slice - 1;
  let mut com_hhash = vec![HashDigest::default(); depth + 1];

  let mut pos = pow_0 + (1 << (fri_ctx.log_current_witness_size_per_slice - 1));
  let mut test_hash = fri_ctx.witness_merkle[oracle_indicator][pos];
  com_hhash[depth] = fri_ctx.witness_merkle[oracle_indicator][pos];
  let mut data = [HashDigest::default(); 2];

  for i in 0..depth {
    if !fri_ctx.visited_init[oracle_indicator][pos ^ 1] {
      new_size += size_of::<HashDigest>();
    }

    fri_ctx.visited_init[oracle_indicator][pos] = true;
    fri_ctx.visited_init[oracle_indicator][pos ^ 1] = true;

    if (pos & 1) == 1 {
      data[0] = fri_ctx.witness_merkle[oracle_indicator][pos ^ 1];
      data[1] = test_hash;
    } else {
      data[0] = test_hash;
      data[1] = fri_ctx.witness_merkle[oracle_indicator][pos ^ 1];
    }
    test_hash = my_hash(data);

    com_hhash[i] = fri_ctx.witness_merkle[oracle_indicator][pos ^ 1];
    pos /= 2;
    assert_eq!(test_hash, fri_ctx.witness_merkle[oracle_indicator][pos]);
  }

  //println!("324 -> {:?} {:?}", value[0], com_hhash[0]);
  assert_eq!(pos, 1);
  (value, com_hhash)
}

pub fn request_step_commit(
  lvl: usize,
  pow: usize,
  mut new_size: usize,
  fri_ctx: &mut FRIContext,
) -> TripleVec {
  new_size = 0;

  let mut pow_0: usize;
  let mut value_vec: Vec<(FieldElement, FieldElement)> = vec![];
  let mut visited_element = false;

  for i in 0..SLICE_NUMBER {
    pow_0 = fri_ctx.cpd.rs_codeword_mapping[lvl][pow << LOG_SLICE_NUMBER | i];
    pow_0 /= 2;

    if !fri_ctx.visited[lvl][pow_0 * 2] {
      fri_ctx.visited[lvl][pow_0 * 2] = true;
    } else {
      visited_element = true;
    }

    value_vec.push((
      fri_ctx.cpd.rs_codeword[lvl][pow_0 * 2],
      fri_ctx.cpd.rs_codeword[lvl][pow_0 * 2 + 1],
    ));
  }

  if !visited_element {
    new_size += size_of::<HashDigest>();
  }

  let mut com_hhash: Vec<HashDigest> = vec![];
  pow_0 = (fri_ctx.cpd.rs_codeword_mapping[lvl][pow << LOG_SLICE_NUMBER] >> (LOG_SLICE_NUMBER + 1))
    + fri_ctx.cpd.merkle_size[lvl];

  let val_hhash = fri_ctx.cpd.merkle[lvl][pow_0];

  while pow_0 != 1 {
    if !fri_ctx.visited[lvl][pow_0 ^ 1] {
      new_size += size_of::<HashDigest>();
      fri_ctx.visited[lvl][pow_0 ^ 1] = true;
      fri_ctx.visited[lvl][pow_0] = true;
    }
    com_hhash.push(fri_ctx.cpd.merkle[lvl][pow_0 ^ 1]);
    pow_0 /= 2;
  }

  com_hhash.push(val_hhash);

  (value_vec, com_hhash)
}
