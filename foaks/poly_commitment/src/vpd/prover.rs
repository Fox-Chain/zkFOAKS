use crate::PolyCommitContext;
use infrastructure::my_hash::HashDigest;

use crate::vpd::fri::{request_init_commit, FRIContext};

/// This will returns the merkle root
pub fn vpd_prover_init(
  fri_ctx: &mut FRIContext,
  poly_ctx: &PolyCommitContext,
  log_input_length: usize,
) -> HashDigest {
  request_init_commit(fri_ctx, poly_ctx, log_input_length, 0)
}
