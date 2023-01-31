use infrastructure::my_hash::HashDigest;
use poly_commitment::poly_commitment::PolyCommitContext;

use crate::fri::{request_init_commit, FRIContext};

/// This will returns the merkle root
pub fn vpd_prover_init(
    PolyCommitContext: PolyCommitContext,
    log_input_length: usize,
) -> HashDigest {
    request_init_commit(
        &mut FRIContext::default(),
        PolyCommitContext,
        log_input_length,
        0,
    )
}
