use infrastructure::my_hash::HashDigest;
use crate::PolyCommitContext;

use crate::vpd::fri::{request_init_commit, FRIContext};

/// This will returns the merkle root
pub fn vpd_prover_init(fri_ctx: &mut FRIContext, log_input_length: usize) -> HashDigest {
  request_init_commit(
    fri_ctx,
    &PolyCommitContext::default(),
    log_input_length,
    0,
  )
}
