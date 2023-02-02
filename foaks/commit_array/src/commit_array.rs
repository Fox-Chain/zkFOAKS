use std::time;

use poly_commitment::poly_commitment::{PolyCommitContext, PolyCommitProver};
use prime_field::FieldElement;
use vpd::{fri, prover};

use infrastructure::constants::*;
use infrastructure::my_hash::HashDigest;
use infrastructure::rs_polynomial::{self, fast_fourier_transform, inverse_fast_fourier_transform};
use infrastructure::utility;

// This work for now but the poly_commit_prover should be a pointer instead of a clone, need to figure out how to change poly_commit_prover in memory not the clone
pub fn commit_private_array(
    mut poly_commit_prover: PolyCommitProver,
    private_array: &[FieldElement],
    log_array_length: usize,
) -> (HashDigest, PolyCommitProver) {
    poly_commit_prover.total_time_pc_p = 0.;
    let now = time::Instant::now();

    let t0 = now.elapsed();

    poly_commit_prover.ctx.pre_prepare_executed = true;

    let slice_count = 1 << LOG_SLICE_NUMBER;
    poly_commit_prover.ctx.slice_count = slice_count;

    let slice_size = 1 << (log_array_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
    poly_commit_prover.ctx.slice_size = slice_size;

    let slice_real_ele_cnt = slice_size >> RS_CODE_RATE;
    poly_commit_prover.ctx.slice_real_ele_cnt = slice_real_ele_cnt;

    let l_eval_len = slice_count * slice_size;
    poly_commit_prover.ctx.l_eval_len = l_eval_len;

    let l_eval = &mut poly_commit_prover.ctx.l_eval;
    l_eval.reserve(l_eval_len);

    let mut tmp = Vec::<FieldElement>::with_capacity(slice_real_ele_cnt);

    let order = slice_size * slice_count;

    let mut scratch_pad = rs_polynomial::ScratchPad::from_order(order);

    for i in 0..slice_count {
        let mut all_zero = true;
        let zero = FieldElement::zero();

        for j in 0..slice_real_ele_cnt {
            if private_array[i * slice_real_ele_cnt + j] == zero {
                continue;
            }
            all_zero = false;
            break;
        }

        if all_zero {
            for j in 0..slice_size {
                l_eval.push(zero);
            }
        } else {
            inverse_fast_fourier_transform(
                &mut scratch_pad,
                &private_array[i * slice_real_ele_cnt..],
                slice_real_ele_cnt,
                slice_real_ele_cnt,
                FieldElement::get_root_of_unity(utility::my_log(slice_real_ele_cnt).unwrap())
                    .unwrap(),
                &mut tmp[..],
            );

            fast_fourier_transform(
                &mut scratch_pad.dst,
                &mut scratch_pad.twiddle_factor,
                &mut scratch_pad.twiddle_factor_size,
                &tmp[..],
                slice_real_ele_cnt,
                slice_size,
                FieldElement::get_root_of_unity(utility::my_log(slice_size).unwrap()).unwrap(),
                &mut l_eval[i * slice_size..],
            )
        }
    }

    let elapsed_time = now.elapsed();
    println!("FFT Prepare time: {} ms", elapsed_time.as_millis());

    let ret = prover::vpd_prover_init(poly_commit_prover.ctx.clone(), log_array_length);

    let t1 = now.elapsed();
    let time_span = t1 - t0;
    poly_commit_prover.total_time_pc_p += time_span.as_secs_f64();
    println!("VPD prepare time {:?}", time_span);
    return (ret, poly_commit_prover.clone());
}

pub fn commit_public_array(
    mut poly_commit_prover: PolyCommitProver,
    public_array: Vec<FieldElement>,
    r_0_len: usize,
    target_sum: FieldElement,
    all_sum: Vec<FieldElement>,
) -> HashDigest {
    let now = time::Instant::now();
    let t0 = now.elapsed();

    assert!(poly_commit_prover.ctx.pre_prepare_executed);

    // TODO: fri::virtual_oracle_witness
    // fri::virtual_oracle_witness = new prime_field::field_element[slice_size * slice_count];
    // fri::virtual_oracle_witness_mapping = new int[slice_size * slice_count];
    // q_eval_len = l_eval_len;
    // q_eval = new prime_field::field_element[q_eval_len];

    // skip some code
    let t0 = now.elapsed();
    let ret = fri::request_init_commit(
        &mut fri::FRIContext::default(),
        poly_commit_prover.ctx,
        r_0_len,
        1,
    );
    // let t1 = now.elapsed();
    // time_span = std::chrono::duration_cast<std::chrono::duration<double>>(t1 - t0);
    // total_time += time_span.count();

    // printf("PostGKR prepare time 1 %lf\n", time_span.count());
    return ret;
}
