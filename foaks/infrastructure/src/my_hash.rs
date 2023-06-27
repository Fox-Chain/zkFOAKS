use blake3::Hasher;
use ring::digest::{Context, SHA256};

/// TODO: https://doc.rust-lang.org/beta/core/arch/x86_64/struct.__m128i.html
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct HashDigest {
  pub h0: u128,
  pub h1: u128,
}

impl HashDigest {
  pub fn new() -> Self { HashDigest { h0: 0, h1: 0 } }

  /// C++ represents u128 as two hex u64. <br> Example: <br>
  /// `h0: 0x760a6c50ad47cbae 0xf8101008f3bfebd5` <br>
  /// `h1: 0x8cf39445888620bb 0xd391c02aa69d486f`
  pub fn new_from_c(h0_0: u64, h0_1: u64, h1_0: u64, h1_1: u64) -> Self {
    HashDigest {
      h0: ((h0_0 as u128) << 64) | (h0_1 as u128),
      h1: ((h1_0 as u128) << 64) | (h1_1 as u128)
    }
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
  let mut digest_u128 = [0u8; 16];

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
  let mut digest_u128 = [0u8; 16];

  digest_u128.copy_from_slice(&digest_bytes[..16]);
  hash.h0 = u128::from_be_bytes(digest_u128);
  digest_u128.copy_from_slice(&digest_bytes[16..]);
  hash.h1 = u128::from_be_bytes(digest_u128);

  hash
}
