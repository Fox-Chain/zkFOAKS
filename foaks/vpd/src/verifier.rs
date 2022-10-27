use infrastructure::my_hash::{self, HashDigest};

pub fn verify_merkle(
    hash_digest: HashDigest,
    merkle_path: Vec<HashDigest>,
    len: usize,
    pow: i32,
) -> bool {
    // We need to make sure the len is always smaller than the size of merklePath.
    assert!(merkle_path.len() > len);

    let current_hash: HashDigest = *merkle_path.last().unwrap();

    let mut data: [HashDigest; 2] = [HashDigest::new(), HashDigest::new()];
    let value_hash: [HashDigest; 2] = [HashDigest::new(), HashDigest::new()];

    let mut new_hash = HashDigest::new();

    for i in 0..len {
        if (pow & i as i32).is_positive() {
            data[0] = merkle_path[i];
            data[1] = current_hash;
        } else {
            data[0] = current_hash;
            data[1] = merkle_path[i];
        }

        let pow = pow >> 1;
        new_hash = my_hash::my_hash(data);

        // TODO: field element needs to be done
    }

    hash_digest == new_hash // && merkle_path.last() == Some(value_hash)
}
