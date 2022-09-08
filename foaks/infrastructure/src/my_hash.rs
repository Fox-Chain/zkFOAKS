use sha3::{Digest, Sha3_256};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct HashDigest {
    pub h0: i128,
    pub h1: i128,
}

// NOTE: no need to implement
// #[inline]
// pub fn equals(a: &HashDigest, b: &HashDigest) -> bool {

// }

// TODO: implement
#[inline]
pub fn my_hash(src: [HashDigest; 2], dst: &mut HashDigest) {

}