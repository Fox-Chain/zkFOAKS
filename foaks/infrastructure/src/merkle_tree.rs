use std::{
  mem::{size_of, size_of_val},
  vec::Vec,
};

use prime_field::FieldElement;

use crate::my_hash::{HashDigest, my_hash};

// Todo: Debug coppy no overlapping
pub unsafe fn hash_single_field_element(x: FieldElement) -> HashDigest {
  let mut data = [HashDigest::default(); 2];
  data[0].h0 = HashDigest::memcpy_from_field_element(x).h0; // merkle_tree.cpp 9
  assert_eq!(size_of_val(&x), size_of_val(&data[0].h0));
  my_hash(data)
}

//ToDo: Debbug copy_nonoverlapping
pub unsafe fn hash_double_field_element_merkle_damgard(
  x: FieldElement,
  y: FieldElement,
  prev_hash: HashDigest,
) -> HashDigest {
  let mut data = [HashDigest::default(); 2];
  data[0] = prev_hash;
  let element = [x, y];
  data[1] = HashDigest::memcpy_from_field_elements(element); // merkle_tree.cpp 22
  assert_eq!(size_of::<HashDigest>(), 2 * size_of::<FieldElement>());
  my_hash(data)
}

pub fn create_tree(
  src_data: Vec<HashDigest>,
  element_num: usize,
  dst: &mut Vec<HashDigest>,
  alloc_required: bool,
) {
  // ToDo: Check this, do not need element_size_
  //let element_num = element_num.unwrap_or(256 / 8);
  //let alloc_required = alloc_required_.unwrap_or(false);

  let mut size_after_padding = 1;
  while size_after_padding < element_num {
    size_after_padding *= 2;
  }
  if alloc_required {
    *dst = vec![HashDigest::default(); size_after_padding * 2];
    //println!("dst size: {}", dst.len());
  }
  let mut start_idx = size_after_padding;
  let mut current_lvl_size = size_after_padding;
  // TODO: parallel
  for i in (0..current_lvl_size).rev() {
    if i < element_num {
      dst[i + start_idx] = src_data[i];
    } else {
      // my_hash(data, &mut dst[i + start_idx]);
      dst[i + start_idx] = my_hash([HashDigest::default(); 2]);
    }
  }
  current_lvl_size /= 2;
  start_idx -= current_lvl_size;
  while current_lvl_size >= 1 {
    // TODO: parallel
    for i in 0..current_lvl_size {
      let mut data = [HashDigest::default(); 2];
      data[0] = dst[start_idx + current_lvl_size + i * 2];
      data[1] = dst[start_idx + current_lvl_size + i * 2 + 1];
      // my_hash(data, &mut dst[start_idx + i]);
      dst[start_idx + i] = my_hash(data);
    }
    current_lvl_size /= 2;
    start_idx -= current_lvl_size;
  }
}
// Gian: Propose this way to implement verify_claim()
pub fn verify_claim(
  root_hash: HashDigest,
  tree: Vec<HashDigest>,
  mut leaf_hash: HashDigest,
  pos_element_arr: usize,
  n: usize,
  visited: &mut Vec<bool>,
  proof_size: &mut usize,
) -> bool {
  // check N is power of 2
  assert_eq!(((n as i64) & -(n as i64)), n as i64);
  let mut pos_element = pos_element_arr + n;
  let mut data = [HashDigest::default(); 2];
  while pos_element != 1 {
    data[pos_element & 1] = leaf_hash;
    data[(pos_element & 1) ^ 1] = tree[pos_element ^ 1];
    if !visited[pos_element ^ 1] {
      visited[pos_element ^ 1] = true;
      *proof_size = *proof_size + size_of_val(&leaf_hash);
    }
    leaf_hash = my_hash(data);
    pos_element /= 2;
  }
  root_hash == leaf_hash
}
