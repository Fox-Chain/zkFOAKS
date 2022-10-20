use sha3::{Digest, Sha3_256};

use crypto::sha2::sha256_digest_block;

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
    // the original sha256_update_shani type signature is
    // static void update_shani(uint32_t *state, const uint8_t *msg, uint32_t num_blocks)
    // return sha256_digest_block(src, dst);
}
