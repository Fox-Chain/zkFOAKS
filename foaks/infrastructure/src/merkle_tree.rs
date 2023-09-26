use std::{
  mem::{size_of, size_of_val},
  vec::Vec,
};

use crate::my_hash::{my_hash, HashDigest};
use prime_field::FieldElement;

pub fn hash_single_field_element(x: FieldElement) -> HashDigest {
  let mut data = [HashDigest::default(); 2];
  data[0].h0 = HashDigest::memcpy_from_field_element(x).h0; // merkle_tree.cpp 9
  assert_eq!(size_of_val(&x), size_of_val(&data[0].h0));
  my_hash(data)
}

pub fn hash_double_field_element_merkle_damgard(
  x: FieldElement,
  y: FieldElement,
  prev_hash: HashDigest,
) -> HashDigest {
  let mut data = [prev_hash, HashDigest::default()];
  let element = [x, y];
  data[1] = HashDigest::memcpy_from_field_elements(element); // merkle_tree.cpp 22
  assert_eq!(size_of::<HashDigest>(), 2 * size_of::<FieldElement>());
  my_hash(data)
}

pub fn create_tree(dst: &mut Vec<HashDigest>, src_data: &[HashDigest], alloc_required: bool) {
  let element_num = src_data.len();
  let size_after_padding = 1 << (element_num as f64).log2().ceil() as usize;

  if alloc_required {
    dst.clear();
    dst.reserve(size_after_padding * 2);
    dst.extend_from_slice(&vec![HashDigest::default(); size_after_padding * 2]);
  }
  let mut start_idx = size_after_padding;
  let mut current_lvl_size = size_after_padding;

  // Refactored secction
  assert_eq!(current_lvl_size, element_num);
  // @dev If assert_eq! fails,then check original code(C/C++)

  // @dev original code(C/C++) uses reversed bucle
  dst[start_idx..start_idx + current_lvl_size].copy_from_slice(src_data);

  current_lvl_size /= 2;
  start_idx -= current_lvl_size;
  while current_lvl_size >= 1 {
    let chunk_start = start_idx + current_lvl_size;

    for i in 0..current_lvl_size {
      let data = [dst[chunk_start + i * 2], dst[chunk_start + i * 2 + 1]];
      dst[start_idx + i] = my_hash(data);
    }

    current_lvl_size >>= 1;
    start_idx -= current_lvl_size;
  }
}

// New way to implement verify_claim()
pub fn verify_claim(
  root_hash: HashDigest,
  tree: &[HashDigest],
  mut leaf_hash: HashDigest,
  pos_element_arr: usize,
  n: usize,
  visited: &mut [bool],
  proof_size: &mut usize,
) -> bool {
  assert_eq!(((n as i64) & -(n as i64)), n as i64);
  let mut pos_element = pos_element_arr + n;
  let mut data = [HashDigest::default(); 2];
  while pos_element != 1 {
    data[pos_element & 1] = leaf_hash;
    data[(pos_element & 1) ^ 1] = tree[pos_element ^ 1];
    if !visited[pos_element ^ 1] {
      visited[pos_element ^ 1] = true;
      *proof_size += size_of_val(&leaf_hash)
    }
    leaf_hash = my_hash(data);
    pos_element /= 2;
  }
  root_hash == leaf_hash
}
