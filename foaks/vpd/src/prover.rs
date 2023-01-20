use infrastructure::my_hash::HashDigest;
use poly_commitment::poly_commitment::PolyCommitContext;
use prime_field::FieldElement;

use crate::fri::{request_init_commit, FRIContext};

/// This will returns the merkle root
pub fn vpd_prover_init(
    l_eval: &mut Vec<FieldElement>,
    // l_coef: usize,
    log_array_length: usize,
    slice_size: usize,
    slice_count: usize,
) -> HashDigest {
    request_init_commit(
        &mut FRIContext::default(),
        slice_size,
        slice_count,
        l_eval,
        log_array_length,
        0,
    )
}
