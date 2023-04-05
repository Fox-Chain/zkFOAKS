use infrastructure::my_hash::HashDigest;
use linear_gkr::prover::ZkProver;
use linear_gkr::verifier::ZkVerifier;
use prime_field::FieldElement;
use linear_code::{parameter::{COLUMN_SIZE}, linear_code_encode::encode};

//commit
pub struct LinearPC {
  encoded_codeword: Vec<Vec<FieldElement>>,
  coef: Vec<Vec<FieldElement>>,
  codeword_size: Vec<u64>,
  mt: Vec<HashDigest>,
  p: ZkProver,
  v: ZkVerifier,
}

impl LinearPC {
	
	pub fn commit(&mut self, src: Vec<FieldElement>, n: u64) -> Vec<HashDigest> {
	let stash= vec![HashDigest::new(); (n / COLUMN_SIZE  * 2).try_into().unwrap()] ;
  self.codeword_size = vec![0; COLUMN_SIZE.try_into().unwrap()];
    assert_eq!(n % COLUMN_SIZE ,0);
    self.encoded_codeword = vec![vec![FieldElement::zero()]; COLUMN_SIZE.try_into().unwrap()];
    self.coef = vec![vec![FieldElement::zero()]; COLUMN_SIZE.try_into().unwrap()];
    for i in 0..COLUMN_SIZE as usize
    {
        self.encoded_codeword[i] = vec![FieldElement::zero();(n / COLUMN_SIZE  * 2).try_into().unwrap()];
        self.coef[i] = vec![FieldElement::zero(); COLUMN_SIZE.try_into().unwrap()];
				// Todo: Debug, could use std::ptr::copy_nonoverlapping() instead
			  self.coef[i] = (src[i * (n / COLUMN_SIZE).try_into().unwrap()..]).to_vec();

        //memset(encoded_codeword[i], 0, sizeof(prime_field::field_element) * n / COLUMN_SIZE * 2);
        self.codeword_size[i] = encode(src[i * (n / COLUMN_SIZE).try_into().unwrap()..], &mut self.encoded_codeword[i], n / COLUMN_SIZE, Some(0));
    }

    for(int i = 0; i < n / COLUMN_SIZE * 2; ++i)
    {
        memset(&stash[i], 0, sizeof(__hhash_digest));
        for(int j = 0; j < COLUMN_SIZE / 2; ++j)
        {
            stash[i] = merkle_tree::hash_double_field_element_merkle_damgard(encoded_codeword[2 * j][i], encoded_codeword[2 * j + 1][i], stash[i]);
        }
    }

    merkle_tree::merkle_tree_prover::create_tree(stash, n / COLUMN_SIZE * 2, mt, sizeof(__hhash_digest), true);
    return mt;
}
}


//open

//verify
fn open_and_verify(x: FieldElement, n: i64, com_mt: *const hhash_digest) -> (FieldElement, bool);
fn open_and_verify(
  r: &mut [FieldElement],
  size_r: usize,
  n: i64,
  com_mt: *const hhash_digest,
) -> (FieldElement, bool);
