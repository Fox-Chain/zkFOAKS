use rand::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::mem::swap;
use std::time::{self, Duration, Instant};
use vpd::verifier::verify_merkle;

use poly_commitment::poly_commitment::{
    LdtCommitment, PolyCommitContext, PolyCommitProver, PolyCommitVerifier,
};
use prime_field::FieldElement;
use vpd::fri::FRIContext;
use vpd::{fri, prover};

use infrastructure::constants::*;
use infrastructure::my_hash::HashDigest;
use infrastructure::rs_polynomial::{self, fast_fourier_transform, inverse_fast_fourier_transform};
use infrastructure::utility::{self, min, my_log};

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

//Todo: debug     let frycontext = &mut FRIContext::default();
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
) -> Result<bool, Error> {
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
        let mut time_span: Duration;
        let inv_2 = FieldElement::inverse(FieldElement::from_real(2));
        let pre_alpha_1: (FieldElement, (HashDigest, usize));
        let mut alpha_l: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let mut alpha_h: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let mut alpha: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>) =
            (Vec::new(), Vec::new());
        let beta_l: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let beta_h: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>);
        let mut beta: (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>) =
            (Vec::new(), Vec::new());
        let mut s0: FieldElement;
        let mut s1: FieldElement;
        let mut pre_y = FieldElement::zero();
        let mut root_of_unity = FieldElement::zero();
        let mut y = FieldElement::zero();
        let mut equ_beta: bool;
        assert!(log_length - LOG_SLICE_NUMBER > 0);
        let mut pow: u128 = 0;

        for i in 0..(log_length - LOG_SLICE_NUMBER) {
            t0 = Instant::now();
            if i == 0 {
                pow = rand::random::<u128>()
                    % (1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i));
                while pow < (1 << (log_length - LOG_SLICE_NUMBER - i)) || pow % 2 == 1 {
                    pow = rand::random::<u128>()
                        % (1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i));
                }
                root_of_unity = FieldElement::get_root_of_unity(
                    log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i,
                )
                .unwrap();
                y = FieldElement::fast_pow(root_of_unity, pow);
            } else {
                root_of_unity = root_of_unity * root_of_unity;
                pow = pow % (1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i));
                pre_y = y;
                y = y * y;
            }
            assert!(pow % 2 == 0);
            let s0_pow = pow / 2;
            let s1_pow = (pow + (1u128 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i))) / 2;
            s0 = FieldElement::fast_pow(root_of_unity, s0_pow);
            s1 = FieldElement::fast_pow(root_of_unity, s1_pow);
            let mut indicator: usize = 0;
            if i != 0 {
                assert!(s1 == pre_y || s0 == pre_y);
                if s1 == pre_y {
                    indicator = 1;
                } else {
                    indicator = 0;
                }
            }
            assert!(s0 * s0 == y);
            assert!(s1 * s1 == y);
            let mut new_size = 0;
            let frycontext = &mut FRIContext::default();

            if i == 0 {
                time_span = t0.elapsed();
                *v_time += time_span.as_secs_f64();

                alpha_l = FRIContext::request_init_value_with_merkle(
                    frycontext,
                    s0_pow.try_into().unwrap(),
                    s1_pow.try_into().unwrap(),
                    &mut new_size,
                    0,
                );
                alpha_h = FRIContext::request_init_value_with_merkle(
                    frycontext,
                    s0_pow.try_into().unwrap(),
                    s1_pow.try_into().unwrap(),
                    &mut new_size,
                    1,
                );

                *proof_size += new_size; //both h and p

                t0 = Instant::now();
                if !verify_merkle(
                    merkle_tree_l,
                    alpha_l.1.clone(),
                    alpha_l.1.len(),
                    min(s0_pow, s1_pow).try_into().unwrap(),
                    alpha_l.0.clone(),
                ) {
                    return Ok(false);
                }
                if !verify_merkle(
                    merkle_tree_h,
                    alpha_h.1.clone(),
                    alpha_h.1.len(),
                    min(s0_pow, s1_pow).try_into().unwrap(),
                    alpha_h.0.clone(),
                ) {
                    return Ok(false);
                }
                time_span = t0.elapsed();
                *v_time += time_span.as_secs_f64();
                beta = FRIContext::request_step_commit(
                    frycontext,
                    0,
                    (pow / 2).try_into().unwrap(),
                    &mut new_size,
                );

                *proof_size += new_size;

                t0 = Instant::now();
                if (!verify_merkle(
                    com.commitment_hash[0],
                    beta.1.clone(),
                    beta.1.len(),
                    (pow / 2).try_into().unwrap(),
                    beta.0.clone(),
                )) {
                    return Ok(false);
                }

                let inv_mu = FieldElement::inverse(FieldElement::fast_pow(root_of_unity, pow / 2));
                alpha.0.clear();
                alpha.1.clear();
                let mut rou = vec![FieldElement::zero(); 2];
                let mut x = vec![FieldElement::zero(); 2];
                let mut inv_x = vec![FieldElement::zero(); 2];
                x[0] = FieldElement::get_root_of_unity(my_log(slice_size).unwrap()).unwrap();
                x[1] = FieldElement::get_root_of_unity(my_log(slice_size).unwrap()).unwrap();
                x[0] = FieldElement::fast_pow(x[0], s0_pow);
                x[1] = FieldElement::fast_pow(x[1], s1_pow);
                rou[0] =
                    FieldElement::fast_pow(x[0], (slice_size >> RS_CODE_RATE).try_into().unwrap());
                rou[1] =
                    FieldElement::fast_pow(x[1], (slice_size >> RS_CODE_RATE).try_into().unwrap());
                inv_x[0] = FieldElement::inverse(x[0]);
                inv_x[1] = FieldElement::inverse(x[1]);

                let inv_H =
                    FieldElement::inverse(FieldElement::from_real(slice_size >> RS_CODE_RATE));
                alpha
                    .0
                    .resize(slice_count, (FieldElement::zero(), FieldElement::zero()));

                let mut tst0 = FieldElement::zero();
                let mut tst1 = FieldElement::zero();
                let q_eval_0_val = FieldElement::zero();
                let q_eval_1_val = FieldElement::zero();
                let mut x0 = FieldElement::real_one();
                let mut x1 = FieldElement::real_one();

                for j in 0..slice_count {
                    tst0 = FieldElement::zero();
                    tst1 = FieldElement::zero();
                    x0 = FieldElement::real_one();
                    x1 = FieldElement::real_one();
                    for k in 0..(1 << (log_length - LOG_SLICE_NUMBER)) {
                        tst0 = tst0 + x0 * public_array[k + j * coef_slice_size];
                        x0 = x0 * x[0];
                        tst1 = tst1 + x1 * public_array[k + j * coef_slice_size];
                        x1 = x1 * x[1];
                    }
                    let mut q_eval_0 = FieldElement::zero();
                    let x0 = FieldElement::real_one();
                    let mut q_eval_1 = FieldElement::zero();
                    let x1 = FieldElement::real_one();
                    q_eval_0 = q_eval_0_val;
                    q_eval_1 = q_eval_1_val;
                    let one = FieldElement::real_one();
                    //merge l and h
                    {
                        alpha.0[j].0 =
                            alpha_l.clone().0[j].0 * tst0 - (rou[0] - one) * alpha_h.clone().0[j].0;
                        alpha.0[j].0 = (alpha.0[j].0
                            * FieldElement::from_real(slice_size >> RS_CODE_RATE)
                            - all_sum[j])
                            * inv_x[0];
                        alpha.0[j].1 =
                            alpha_l.clone().0[j].1 * tst1 - (rou[1] - one) * alpha_h.clone().0[j].1;
                        alpha.0[j].1 = (alpha.0[j].1
                            * FieldElement::from_real(slice_size >> RS_CODE_RATE)
                            - all_sum[j])
                            * inv_x[1];
                    }
                    if s0_pow > s1_pow {
                        //swap
                        let first = alpha.0[j].0.clone();
                        let second = alpha.0[j].1.clone();

                        alpha.0[j].0 = second;
                        alpha.0[j].1 = first;
                    }
                    let p_val = (alpha.0[j].0 + alpha.0[j].1) * inv_2
                        + (alpha.0[j].0 - alpha.0[j].1) * inv_2 * com.randomness[i] * inv_mu;

                    if (p_val != beta.clone().0[j].0 && p_val != beta.clone().0[j].1) {
                        //todo! improve print
                        println!("Fri check consistency 0 round fail {}", j);
                        //println!(stderr, "Fri check consistency 0 round fail %d\n", j);
                        return Ok(false);
                    }
                    if (p_val == beta.clone().0[j].0) {
                        equ_beta = false;
                    } else {
                        equ_beta = true;
                    }
                }
                time_span = t0.elapsed();
                //This will not added into v time since the fft gkr already give the result, we didn't have time to integrate the fft gkr into the main body, so we have the evaluation code here
                //v_time += time_span.count();
            } else {
                time_span = t0.elapsed();
                *v_time += time_span.as_secs_f64();
                // std::cerr << "Verification Time " << v_time << std::endl;
                if indicator == 1 {
                    alpha = beta;
                } else {
                    alpha = beta;
                }

                beta = FRIContext::request_step_commit(
                    frycontext,
                    i,
                    (pow / 2).try_into().unwrap(),
                    &mut new_size,
                );

                *proof_size += new_size;

                t0 = Instant::now();
                if (!verify_merkle(
                    com.commitment_hash[i],
                    beta.1.clone(),
                    beta.1.len(),
                    (pow / 2).try_into().unwrap(),
                    beta.0.clone(),
                )) {
                    return Ok(false);
                }

                let inv_mu = FieldElement::inverse(FieldElement::fast_pow(root_of_unity, pow / 2));
                time_span = t0.elapsed();
                *v_time += time_span.as_secs_f64();
                for j in 0..slice_count {
                    let p_val_0 = (alpha.0[j].0 + alpha.0[j].1) * inv_2
                        + (alpha.0[j].0 - alpha.0[j].1) * inv_2 * com.randomness[i] * inv_mu;
                    let p_val_1 = (alpha.0[j].0 + alpha.0[j].1) * inv_2
                        + (alpha.0[j].1 - alpha.0[j].0) * inv_2 * com.randomness[i] * inv_mu;
                    if (p_val_0 != beta.0[j].0
                        && p_val_0 != beta.0[j].1
                        && p_val_1 != beta.0[j].0
                        && p_val_1 != beta.0[j].1)
                    {
                        //todo: improve output
                        println!("Fri check consistency {} round fail", i);
                        //fprintf(stderr, "Fri check consistency %d round fail\n", i);
                        return Ok(false);
                    }
                }
            }
        }
        for i in 0..slice_count {
            //Todo: Debug cpd
            let cpd = fri::CommitPhaseData::new();
            let tmplate =
                cpd.rs_codeword[com.mx_depth - 1][0 << (LOG_SLICE_NUMBER + 1) | i << 1 | 0];
            for j in 0..(1 << (RS_CODE_RATE - 1)) {
                if (cpd.rs_codeword[com.mx_depth - 1][j << (LOG_SLICE_NUMBER + 1) | i << 1 | 0]
                    != tmplate)
                {
                    //todo: improve output
                    println!("Fri rs code check fail");
                    //fprintf(stderr, "Fri rs code check fail\n");
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}
