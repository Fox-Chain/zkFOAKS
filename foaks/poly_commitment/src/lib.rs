mod vpd;

use std::thread::sleep;
use std::time;

use prime_field::FieldElement;

use infrastructure::constants::*;
use infrastructure::my_hash::HashDigest;
use infrastructure::rs_polynomial::{self, fast_fourier_transform, inverse_fast_fourier_transform, ScratchPad};
use infrastructure::utility;
use crate::vpd::fri::{FRIContext, request_init_commit};

#[derive(Default)]
pub struct LdtCommitment {
    pub commitment_hash: Vec<HashDigest>,
    pub randomness: Vec<FieldElement>,
    pub final_rs_code: Vec<FieldElement>,
    pub mx_depth: usize,
    // repeat_no: usize,
}

#[derive(Default, Debug, Clone)]
pub struct PolyCommitContext {
    pub twiddle_factor: Vec<FieldElement>,
    pub inv_twiddle_factor: Vec<FieldElement>,
    pub twiddle_factor_size: usize,
    pub inner_prod_evals: Vec<FieldElement>,

    pub l_coef: Vec<FieldElement>,
    pub l_coef_len: usize,

    pub l_eval: Vec<FieldElement>,
    pub l_eval_len: usize,
    pub q_coef: Vec<FieldElement>,
    pub q_coef_len: usize,

    pub q_eval: Vec<FieldElement>,
    pub q_eval_len: usize,

    pub lq_coef: Vec<FieldElement>,
    pub lq_eval: Vec<FieldElement>,
    pub h_coef: Vec<FieldElement>,
    pub h_eval: Vec<FieldElement>,

    pub h_eval_arr: Vec<FieldElement>,

    pub slice_size: usize,
    pub slice_count: usize,
    pub slice_real_ele_cnt: usize,
    pub pre_prepare_executed: bool,
}

#[derive(Default, Debug, Clone)]
pub struct PolyCommitProver {
    pub total_time_pc_p: f64,
    ctx: PolyCommitContext,
    fri_ctx: Option<FRIContext>,
    scratch_pad: ScratchPad
}

impl PolyCommitProver {
    pub fn commit_private_array(&mut self, private_array: &[FieldElement], log_array_length: usize, ) -> HashDigest {
        self.total_time_pc_p = 0.;
        let t0 = time::Instant::now();

        self.ctx.pre_prepare_executed = true;

        let slice_count = 1 << LOG_SLICE_NUMBER;
        self.ctx.slice_count = slice_count;

        let slice_size = 1 << (log_array_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
        self.ctx.slice_size = slice_size;

        let slice_real_ele_cnt = slice_size >> RS_CODE_RATE;
        self.ctx.slice_real_ele_cnt = slice_real_ele_cnt;

        let l_eval_len = slice_count * slice_size;
        self.ctx.l_eval_len = l_eval_len;

        let l_eval = &mut self.ctx.l_eval;
        l_eval.reserve(l_eval_len);

        let mut tmp = Vec::<FieldElement>::with_capacity(slice_real_ele_cnt);

        //let order = slice_size * slice_count;

        let now = time::Instant::now();

        // replaces init_scratch_pad
        self.scratch_pad = ScratchPad::from_order(slice_size * slice_count);

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
                    l_eval[i * slice_size + j] = zero;
                }
            } else {
                inverse_fast_fourier_transform(
                    &mut self.scratch_pad,
                    &private_array[i * slice_real_ele_cnt..],
                    slice_real_ele_cnt,
                    slice_real_ele_cnt,
                    FieldElement::get_root_of_unity(utility::my_log(slice_real_ele_cnt).unwrap()).unwrap(),
                    &mut tmp[..],
                );

                fast_fourier_transform(
                    &tmp[..],
                    slice_real_ele_cnt,
                    slice_size,
                    FieldElement::get_root_of_unity(utility::my_log(slice_size).unwrap()).unwrap(),
                    &mut l_eval[i * slice_size..],
                    &mut self.scratch_pad.twiddle_factor,
                    &mut self.scratch_pad.dst,
                    &mut self.scratch_pad.twiddle_factor_size,
                )
            }
        }

        let elapsed_time = now.elapsed();
        println!("FFT Prepare time: {} ms", elapsed_time.as_millis());

        if self.fri_ctx.is_none() {
            self.fri_ctx = Some(FRIContext::default());
        }

        let ret = vpd::prover::vpd_prover_init(&mut self.fri_ctx.unwrap(), log_array_length);

        let time_span = t0.elapsed().as_secs_f64();
        self.total_time_pc_p += time_span;
        println!("VPD prepare time {}", time_span);

        ret
    }

    pub fn commit_public_array(
        &mut self,
        public_array: &[FieldElement],
        r_0_len: usize,
        target_sum: FieldElement,
        all_sum: &mut [FieldElement],
    ) -> HashDigest {
        let mut t0 = time::Instant::now();
        assert!(self.ctx.pre_prepare_executed);
        let mut fri_ctx = self.fri_ctx.unwrap_or(FRIContext::default());
        fri_ctx.virtual_oracle_witness = vec![FieldElement::default(), self.ctx.slice_size * self.ctx.slice_count];
        fri_ctx.virtual_oracle_witness_mapping = vec![0, self.ctx.slice_size * self.ctx.slice_count];

        self.ctx.q_eval_len = self.ctx.l_eval_len;
        self.ctx.q_eval = vec![FieldElement::default(), self.ctx.q_eval_len];

        let mut tmp = vec![FieldElement::default(), self.ctx.slice_size];
        let mut ftt_time = 0.0;
        let mut re_mapping_time = 0.0;

        let mut ftt_t0 = time::Instant::now();

        for i in 0..self.ctx.slice_count {
            inverse_fast_fourier_transform(
                &mut self.scratch_pad,
                &public_array[i * self.ctx.slice_real_ele_cnt..],
                self.ctx.slice_real_ele_cnt,
                self.ctx.slice_real_ele_cnt,
                FieldElement::get_root_of_unity(utility::my_log(self.ctx.slice_real_ele_cnt).unwrap()).unwrap(),
                &mut tmp,
            );

            fast_fourier_transform(
                &temp,
                self.ctx.slice_real_ele_cnt,
                self.ctx.slice_size,
                FieldElement::get_root_of_unity(utility::my_log(self.ctx.slice_size).unwrap()).unwrap(),
                &mut self.ctx.q_eval[i * self.ctx.slice_size..],
                &mut self.scratch_pad.twiddle_factor,
                &mut self.scratch_pad.dst,
                &mut self.scratch_pad.twiddle_factor_size,
            )
        }

        ftt_time += ftt_t0.elapsed().as_secs_f64();

        let mut sum = FieldElement::zero();
        assert_eq!(self.ctx.slice_count * self.ctx.slice_real_ele_cnt, 1 << r_0_len);

        for i in 0..self.ctx.slice_count * self.ctx.slice_real_ele_cnt {
            assert!((i << RS_CODE_RATE) < self.ctx.q_eval_len);
            sum += sum + self.ctx.q_eval[i << RS_CODE_RATE] * self.ctx.l_eval[i << RS_CODE_RATE];
        }

        assert_eq!(sum, target_sum);

        // do fft for q_eval
        // experiment
        // poly mul first

        self.ctx.lq_eval = vec![FieldElement::default(), 2 * self.ctx.slice_real_ele_cnt];
        self.ctx.h_coef = vec![FieldElement::default(), self.ctx.slice_real_ele_cnt];
        self.ctx.lq_coef = vec![FieldElement::default(), 2 * self.ctx.slice_real_ele_cnt];
        let max = std::cmp::max(self.ctx.slice_size, self.ctx.slice_real_ele_cnt);
        self.ctx.h_eval = vec![FieldElement::default(), max];
        self.ctx.h_eval_arr = vec![FieldElement::default(), self.ctx.slice_count * self.ctx.slice_size];

        let log_leaf_size = LOG_SLICE_NUMBER + 1;

        for i in 0..self.ctx.slice_count {
            assert!(2 * self.ctx.slice_real_ele_cnt <= self.ctx.slice_size);
            let mut all_zero = true;
            let zero = FieldElement::zero();

            for j in 0..2 * self.ctx.slice_real_ele_cnt {
                self.ctx.lq_eval[j] =
                    self.ctx.l_eval[i * self.ctx.slice_size + j * (self.ctx.slice_size / (2 * self.ctx.slice_real_ele_cnt))] *
                        self.ctx.q_eval[i * self.ctx.slice_size + j * (self.ctx.slice_size / (2 * self.ctx.slice_real_ele_cnt))];

                if self.ctx.lq_eval[j] != zero { all_zero = false; }
            }

            if all_zero {
                for j in 0..2 * self.ctx.slice_real_ele_cnt {
                    self.ctx.lq_coef[j] = zero;
                }

                for j in 0..self.ctx.slice_real_ele_cnt {
                    self.ctx.h_coef[j] = zero;
                }

                for j in 0..self.ctx.slice_size {
                    self.ctx.h_eval[j] = zero;
                }
            } else {
                ftt_t0 = time::Instant::now();
                inverse_fast_fourier_transform(
                    &mut self.scratch_pad,
                    &self.ctx.lq_eval,
                    2 * self.ctx.slice_real_ele_cnt,
                    2 * self.ctx.slice_real_ele_cnt,
                    FieldElement::get_root_of_unity(utility::my_log(2 * self.ctx.slice_real_ele_cnt).unwrap()).unwrap(),
                    &mut self.ctx.lq_coef,
                );

                for j in 0..self.ctx.slice_real_ele_cnt {
                    self.ctx.h_coef[j] = self.ctx.lq_coef[j + self.ctx.slice_real_ele_cnt];
                }

                fast_fourier_transform(
                    &self.ctx.h_coef,
                    self.ctx.slice_real_ele_cnt,
                    self.ctx.slice_size,
                    FieldElement::get_root_of_unity(utility::my_log(self.ctx.slice_size).unwrap()).unwrap(),
                    &mut self.ctx.h_eval,
                    &mut self.scratch_pad.twiddle_factor,
                    &mut self.scratch_pad.dst,
                    &mut self.scratch_pad.twiddle_factor_size,
                );

                ftt_time += ftt_t0.elapsed().as_secs_f64();
            }

            let twiddle_gap = self.scratch_pad.twiddle_factor_size / self.ctx.slice_size * self.ctx.slice_real_ele_cnt;
            let inv_twiddle_gap = self.scratch_pad.twiddle_factor_size / self.ctx.slice_size;

            let remap_t0 = time::Instant::now();
            let const_sum = FieldElement::zero() - (self.ctx.lq_coef[0] + self.ctx.h_coef[0]);

            for j in 0..self.ctx.slice_size {
                let g = self.ctx.l_eval[i * self.ctx.slice_size + j] *
                    self.ctx.q_eval[i * self.ctx.slice_size + j] -
                    (self.scratch_pad.twiddle_factor[twiddle_gap * j % self.scratch_pad.twiddle_factor_size] - FieldElement::real_one()) *
                    self.ctx.h_eval[j];

                if j < self.ctx.slice_size / 2 {
                    assert!((j << log_leaf_size | (i << 1) | 0) < self.ctx.slice_count * self.ctx.slice_size);
                    assert_eq!((j << log_leaf_size) & (i << 1), 0);

                    fri_ctx.virtual_oracle_witness[j << log_leaf_size | (i << 1) | 0] =
                        (g + const_sum) *
                            self.scratch_pad.inv_twiddle_factor[inv_twiddle_gap * j % self.scratch_pad.twiddle_factor_size] *
                            FieldElement::from_real(self.ctx.slice_real_ele_cnt as u64);

                    fri_ctx.virtual_oracle_witness_mapping[j << LOG_SLICE_NUMBER | i] =
                        j << log_leaf_size | (i << 1) | 0;
                } else {
                    let jj = j - self.ctx.slice_size / 2;
                    assert!((jj << log_leaf_size | (i << 1) | 0) < self.ctx.slice_count * self.ctx.slice_size);
                    assert_eq!((jj << log_leaf_size) & (i << 1), 0);

                    fri_ctx.virtual_oracle_witness[jj << log_leaf_size | (i << 1) | 1] =
                        (g + const_sum) *
                            self.scratch_pad.inv_twiddle_factor[inv_twiddle_gap * j % self.scratch_pad.twiddle_factor_size] *
                            FieldElement::from_real(self.ctx.slice_real_ele_cnt as u64);

                    fri_ctx.virtual_oracle_witness_mapping[jj << LOG_SLICE_NUMBER | i] =
                        jj << log_leaf_size | (i << 1) | 0;
                }
            }

            re_mapping_time = remap_t0.elapsed().as_secs_f64();
            assert!(i < SLICE_NUMBER + 1);
            all_sum[i] = (self.ctx.lq_coef[0] + self.ctx.h_coef[0]) * FieldElement::from_real(self.ctx.slice_real_ele_cnt as u64);

            for j in 0..self.ctx.slice_size {
                self.ctx.h_eval_arr[i * self.ctx.slice_size + j] = self.ctx.h_eval[j];
            }
        }

        let mut time_span = t0.elapsed().as_secs_f64();
        self.total_time_pc_p += time_span;
        println!("PostGKR FFT time {}", ftt_time);
        println!("PostGKR remap time {}", re_mapping_time);
        println!("PostGKR prepare time {}", time_span);

        t0 = time::Instant::now();
        let ret = request_init_commit(&mut fri_ctx, &self.ctx,r_0_len, 1);

        time_span = t0.elapsed().as_secs_f64();
        self.total_time_pc_p += time_span;
        println!("PostGKR prepare time 1 {}", time_span);

        ret
    }

    pub fn commit_phase(self, log_length: usize) -> LdtCommitment {
        self.fri_ctx.unwrap_or_else(|_| FRIContext::default()).commit_phase(log_length)
    }
}

#[derive(Default, Debug)]
pub struct PolyCommitVerifier {
    pub pc_prover: PolyCommitProver,
}

impl PolyCommitVerifier {
    pub fn verify_poly_commitment(
        self,
        all_sum: &[FieldElement],
        log_length: usize,
        public_array: &[FieldElement],
        v_time: f64,
        proof_size: usize,
        p_time: f64,
        merkle_tree_l: HashDigest,
        merkle_tree_h: HashDigest
    ) {
        let slice_count = 1 << LOG_SLICE_NUMBER;
        let slice_size = 1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

        let t0 = time::Instant::now();
        let t1 = time::Instant::now();

        // let inv_2 =
        unimplemented!()
    }
}

pub fn commit_phrase_step(r: FieldElement) -> HashDigest {
  HashDigest::new()
}
