use blake3::Hasher;
use ring::digest::{Context, SHA256};

/// TODO: https://doc.rust-lang.org/beta/core/arch/x86_64/struct.__m128i.html
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct HashDigest {
  pub h0: u128,
  pub h1: u128,
}

impl HashDigest {
  pub fn new() -> Self {
    HashDigest { h0: 0, h1: 0 }
  }
}

#[inline]
pub fn my_hash(src: [HashDigest; 2]) -> HashDigest {
  // the original sha256_update_shani type signature is an optimised function for
  // SHA-NI instruction sets machine, this is the fallback one
  // https://www.ic.unicamp.br/~ra142685/sok-apkc.pdf
  // let mut hasher = Sha3_256::new();
  let mut ctx = Context::new(&SHA256);

  src.iter().for_each(|h| {
    ctx.update(&h.h0.to_be_bytes());
    ctx.update(&h.h1.to_be_bytes());
  });

  let mut hash = HashDigest::new();
  let digest = ctx.finish();
  let digest_bytes = digest.as_ref();
  let mut digest_u128= [0u8; 16];

  digest_u128.copy_from_slice(&digest_bytes[..16]);
  hash.h0 = u128::from_be_bytes(digest_u128);
  digest_u128.copy_from_slice(&digest_bytes[16..]);
  hash.h1 = u128::from_be_bytes(digest_u128);

  hash
}

pub fn my_hash_blake(src: [HashDigest; 2]) -> HashDigest {
  let mut hasher = Hasher::new();

  src.iter().for_each(|h| {
    hasher.update(&h.h0.to_be_bytes());
    hasher.update(&h.h1.to_be_bytes());
  });

  let mut hash = HashDigest::new();
  let digest = hasher.finalize();
  let digest_bytes = digest.as_bytes();
  let mut digest_u128= [0u8; 16];

  digest_u128.copy_from_slice(&digest_bytes[..16]);
  hash.h0 = u128::from_be_bytes(digest_u128);
  digest_u128.copy_from_slice(&digest_bytes[16..]);
  hash.h1 = u128::from_be_bytes(digest_u128);

  hash
}