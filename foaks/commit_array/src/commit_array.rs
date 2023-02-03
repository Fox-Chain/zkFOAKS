use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::time::{self, Duration, Instant};

use poly_commitment::poly_commitment::{
    LdtCommitment, PolyCommitContext, PolyCommitProver, PolyCommitVerifier,
};
use prime_field::FieldElement;
use vpd::fri::FRIContext;
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

pub fn commit_phase(log_length: usize) -> LdtCommitment {
    // let log_current_witness_size_per_slice_cp = self.log_current_witness_size_per_slice;
    // assumming we already have the initial commit
    let mut codeword_size = 1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
    // repeat until the codeword is constant
    let mut ret: Vec<HashDigest> = Vec::with_capacity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
    let mut randomness: Vec<FieldElement> =
        Vec::with_capacity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

    let mut ptr = 0;

    let frycontext = &mut FRIContext::default();

    while codeword_size > 1 << RS_CODE_RATE {
        assert!(ptr < log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
        randomness[ptr] = FieldElement::new_random();
        ret[ptr] = frycontext.commit_phase_step(randomness[ptr]);
        codeword_size /= 2;
        ptr += 1;
    }

    LdtCommitment {
        commitment_hash: ret,
        final_rs_code: frycontext.commit_phase_final(),
        randomness,
        mx_depth: ptr,
    }
}

pub fn verify_poly_commitment(
    //mut poly_commit_verifier: PolyCommitVerifier,
    all_sum: Vec<FieldElement>,
    log_length: usize,
    public_array: Vec<FieldElement>,
    v_time: &mut f64,
    proof_size: &mut usize,
    p_time: &mut f64,
    merkle_tree_l: HashDigest,
    merkle_tree_h: HashDigest,
) -> Result<(), Error> {
    let dif = log_length - LOG_SLICE_NUMBER;
    let mut command = String::from("./fft_gkr ");
    command = command + &dif.to_string() + " log_fftgkr.txt";
    //Todo!     system(command);
    let result_file = File::open("log_fftgkr.txt")?;
    let result_reader = BufReader::new(result_file);
    let mut lines_iter = result_reader.lines().map(|l| l.unwrap());
    let next_line = lines_iter.next().unwrap();
    let mut next_line_splited = next_line.split_whitespace();
    let v_time_fft: f64 = next_line_splited.next().unwrap().parse().unwrap();
    let p_time_fft: f64 = next_line_splited.next().unwrap().parse().unwrap();
    let proof_size_fft: usize = next_line_splited.next().unwrap().parse().unwrap();

    *v_time += v_time_fft;
    *p_time += p_time_fft;
    *proof_size += proof_size_fft;

    let com = commit_phase(log_length);
    let coef_slice_size = (1 << (log_length - LOG_SLICE_NUMBER));

    for rep in 0..33 {
        let slice_count = 1 << LOG_SLICE_NUMBER;
        let slice_size = (1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER));
        let mut t0: Instant;
        let time_span: Duration;
        let inv_2 = FieldElement::inverse(FieldElement::from_real(2));
        let pre_alpha_1: (FieldElement, (HashDigest, usize));
        let alpha_l: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let alpha_h: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let alpha: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let beta_l: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let beta_h: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let beta: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let s0: FieldElement;
        let s1: FieldElement;
        let pre_y: FieldElement;
        let root_of_unity: FieldElement;
        let y: FieldElement;
        let equ_beta: bool;
        assert!(log_length - LOG_SLICE_NUMBER > 0);
        let pow: usize;

        for i in 0..(log_length - LOG_SLICE_NUMBER) {
            t0 = Instant::now();
            if i == 0 {
            } else {
            }
        }
    }
    Ok(())
}
