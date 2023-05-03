mod test;
mod vpd;

use rand::Rng;
use std::{ffi::OsStr, fs::File, io::Read, os::unix::prelude::OsStrExt, process::Command, time};

use prime_field::FieldElement;

use crate::vpd::{
  fri::{
    request_init_commit, request_init_value_with_merkle, request_step_commit, FRIContext, TripleVec,
  },
  verifier::verify_merkle,
};
use infrastructure::{
  constants::*,
  my_hash::HashDigest,
  rs_polynomial::{fast_fourier_transform, inverse_fast_fourier_transform, ScratchPad},
  utility::my_log,
};

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
  pub fri_ctx: Option<FRIContext>,
  pub scratch_pad: ScratchPad,
}

impl PolyCommitProver {
  pub fn commit_private_array(
    &mut self,
    private_array: &[FieldElement],
    log_array_length: usize,
  ) -> HashDigest {
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

    //let l_eval = &mut self.ctx.l_eval;
    self.ctx.l_eval = vec![FieldElement::zero(); l_eval_len];
    //l_eval.reserve(l_eval_len);

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
          self.ctx.l_eval[i * slice_size + j] = zero;
        }
      } else {
        inverse_fast_fourier_transform(
          &mut self.scratch_pad,
          &private_array[i * slice_real_ele_cnt..],
          slice_real_ele_cnt,
          slice_real_ele_cnt,
          FieldElement::get_root_of_unity(my_log(slice_real_ele_cnt).unwrap()).unwrap(),
          &mut tmp[..],
        );

        fast_fourier_transform(
          &tmp[..],
          slice_real_ele_cnt,
          slice_size,
          FieldElement::get_root_of_unity(my_log(slice_size).unwrap()).unwrap(),
          &mut self.ctx.l_eval[i * slice_size..],
          &mut self.scratch_pad.twiddle_factor,
          &mut self.scratch_pad.dst,
          &mut self.scratch_pad.twiddle_factor_size,
        )
      }
    }

    let elapsed_time = now.elapsed();
    println!("FFT Prepare time: {} ms", elapsed_time.as_millis());

    if self.fri_ctx.is_none() {
      self.fri_ctx = Some(FRIContext::new());
    }

    let ret =
      vpd::prover::vpd_prover_init(self.fri_ctx.as_mut().unwrap(), &self.ctx, log_array_length);

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
    let mut default_fri_ctx = FRIContext::new();
    let mut fri_ctx = self.fri_ctx.as_mut().unwrap_or(&mut default_fri_ctx);
    fri_ctx.virtual_oracle_witness =
      vec![FieldElement::default(); self.ctx.slice_size * self.ctx.slice_count];
    fri_ctx.virtual_oracle_witness_mapping = vec![0; self.ctx.slice_size * self.ctx.slice_count];

    self.ctx.q_eval_len = self.ctx.l_eval_len;
    self.ctx.q_eval = vec![FieldElement::default(); self.ctx.q_eval_len];

    let mut tmp = vec![FieldElement::default(); self.ctx.slice_size];
    let mut ftt_time = 0.0;
    let mut re_mapping_time = 0.0;

    let mut ftt_t0 = time::Instant::now();

    for i in 0..self.ctx.slice_count {
      inverse_fast_fourier_transform(
        &mut self.scratch_pad,
        &public_array[i * self.ctx.slice_real_ele_cnt..],
        self.ctx.slice_real_ele_cnt,
        self.ctx.slice_real_ele_cnt,
        FieldElement::get_root_of_unity(my_log(self.ctx.slice_real_ele_cnt).unwrap()).unwrap(),
        &mut tmp,
      );

      fast_fourier_transform(
        &tmp,
        self.ctx.slice_real_ele_cnt,
        self.ctx.slice_size,
        FieldElement::get_root_of_unity(my_log(self.ctx.slice_size).unwrap()).unwrap(),
        &mut self.ctx.q_eval[i * self.ctx.slice_size..],
        &mut self.scratch_pad.twiddle_factor,
        &mut self.scratch_pad.dst,
        &mut self.scratch_pad.twiddle_factor_size,
      )
    }

    ftt_time += ftt_t0.elapsed().as_secs_f64();

    let mut sum = FieldElement::zero();
    assert_eq!(
      self.ctx.slice_count * self.ctx.slice_real_ele_cnt,
      1 << r_0_len
    );

    for i in 0..self.ctx.slice_count * self.ctx.slice_real_ele_cnt {
      assert!((i << RS_CODE_RATE) < self.ctx.q_eval_len);
      sum = sum + self.ctx.q_eval[i << RS_CODE_RATE] * self.ctx.l_eval[i << RS_CODE_RATE];
    }

    assert_eq!(sum, target_sum);

    // do fft for q_eval
    // experiment
    // poly mul first

    self.ctx.lq_eval = vec![FieldElement::default(); 2 * self.ctx.slice_real_ele_cnt];
    self.ctx.h_coef = vec![FieldElement::default(); self.ctx.slice_real_ele_cnt];
    self.ctx.lq_coef = vec![FieldElement::default(); 2 * self.ctx.slice_real_ele_cnt];
    let max = std::cmp::max(self.ctx.slice_size, self.ctx.slice_real_ele_cnt);
    self.ctx.h_eval = vec![FieldElement::default(); max];
    self.ctx.h_eval_arr = vec![FieldElement::default(); self.ctx.slice_count * self.ctx.slice_size];

    let log_leaf_size = LOG_SLICE_NUMBER + 1;

    for i in 0..self.ctx.slice_count {
      assert!(2 * self.ctx.slice_real_ele_cnt <= self.ctx.slice_size);
      let mut all_zero = true;
      let zero = FieldElement::zero();

      for j in 0..2 * self.ctx.slice_real_ele_cnt {
        self.ctx.lq_eval[j] = self.ctx.l_eval
          [i * self.ctx.slice_size + j * (self.ctx.slice_size / (2 * self.ctx.slice_real_ele_cnt))]
          * self.ctx.q_eval[i * self.ctx.slice_size
            + j * (self.ctx.slice_size / (2 * self.ctx.slice_real_ele_cnt))];

        if self.ctx.lq_eval[j] != zero {
          all_zero = false;
        }
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
          FieldElement::get_root_of_unity(my_log(2 * self.ctx.slice_real_ele_cnt).unwrap())
            .unwrap(),
          &mut self.ctx.lq_coef,
        );

        for j in 0..self.ctx.slice_real_ele_cnt {
          self.ctx.h_coef[j] = self.ctx.lq_coef[j + self.ctx.slice_real_ele_cnt];
        }

        fast_fourier_transform(
          &self.ctx.h_coef,
          self.ctx.slice_real_ele_cnt,
          self.ctx.slice_size,
          FieldElement::get_root_of_unity(my_log(self.ctx.slice_size).unwrap()).unwrap(),
          &mut self.ctx.h_eval,
          &mut self.scratch_pad.twiddle_factor,
          &mut self.scratch_pad.dst,
          &mut self.scratch_pad.twiddle_factor_size,
        );

        ftt_time += ftt_t0.elapsed().as_secs_f64();
      }

      let twiddle_gap =
        self.scratch_pad.twiddle_factor_size / self.ctx.slice_size * self.ctx.slice_real_ele_cnt;
      let inv_twiddle_gap = self.scratch_pad.twiddle_factor_size / self.ctx.slice_size;

      let remap_t0 = time::Instant::now();
      let const_sum = FieldElement::zero() - (self.ctx.lq_coef[0] + self.ctx.h_coef[0]);

      for j in 0..self.ctx.slice_size {
        let g = self.ctx.l_eval[i * self.ctx.slice_size + j]
          * self.ctx.q_eval[i * self.ctx.slice_size + j]
          - (self.scratch_pad.twiddle_factor
            [twiddle_gap * j % self.scratch_pad.twiddle_factor_size]
            - FieldElement::real_one())
            * self.ctx.h_eval[j];

        if j < self.ctx.slice_size / 2 {
          assert!((j << log_leaf_size | (i << 1) | 0) < self.ctx.slice_count * self.ctx.slice_size);
          assert_eq!((j << log_leaf_size) & (i << 1), 0);

          fri_ctx.virtual_oracle_witness[j << log_leaf_size | (i << 1) | 0] = (g + const_sum)
            * self.scratch_pad.inv_twiddle_factor
              [inv_twiddle_gap * j % self.scratch_pad.twiddle_factor_size]
            * FieldElement::from_real(self.ctx.slice_real_ele_cnt as u64);

          fri_ctx.virtual_oracle_witness_mapping[j << LOG_SLICE_NUMBER | i] =
            j << log_leaf_size | (i << 1) | 0;
        } else {
          let jj = j - self.ctx.slice_size / 2;
          assert!(
            (jj << log_leaf_size | (i << 1) | 0) < self.ctx.slice_count * self.ctx.slice_size
          );
          assert_eq!((jj << log_leaf_size) & (i << 1), 0);

          fri_ctx.virtual_oracle_witness[jj << log_leaf_size | (i << 1) | 1] = (g + const_sum)
            * self.scratch_pad.inv_twiddle_factor
              [inv_twiddle_gap * j % self.scratch_pad.twiddle_factor_size]
            * FieldElement::from_real(self.ctx.slice_real_ele_cnt as u64);

          fri_ctx.virtual_oracle_witness_mapping[jj << LOG_SLICE_NUMBER | i] =
            jj << log_leaf_size | (i << 1) | 0;
        }
      }

      re_mapping_time = remap_t0.elapsed().as_secs_f64();
      assert!(i < SLICE_NUMBER + 1);
      all_sum[i] = (self.ctx.lq_coef[0] + self.ctx.h_coef[0])
        * FieldElement::from_real(self.ctx.slice_real_ele_cnt as u64);

      for j in 0..self.ctx.slice_size {
        self.ctx.h_eval_arr[i * self.ctx.slice_size + j] = self.ctx.h_eval[j];
      }
    }

    let mut time_span = t0.elapsed().as_secs_f64();
    self.total_time_pc_p += time_span;
    println!("PostGKR FFT time: {}", ftt_time);
    println!("PostGKR remap time: {}", re_mapping_time);
    println!("PostGKR prepare time: 0 {}", time_span);

    t0 = time::Instant::now();
    let ret = request_init_commit(&mut fri_ctx, &self.ctx, r_0_len, 1);

    time_span = t0.elapsed().as_secs_f64();
    self.total_time_pc_p += time_span;
    println!("PostGKR prepare time 1 {}", time_span);

    ret
  }

  pub fn commit_phase(&mut self, log_length: usize) -> LdtCommitment {
    self
      .fri_ctx
      .as_mut()
      .unwrap()
      .commit_phase(log_length, self.ctx.slice_count)
  }
}

#[derive(Default, Debug)]
pub struct PolyCommitVerifier {
  pub pc_prover: PolyCommitProver,
}

impl PolyCommitVerifier {
  pub fn verify_poly_commitment(
    &mut self,
    all_sum: &[FieldElement],
    log_length: usize,
    public_array: &[FieldElement],
    mut v_time: f64,
    mut proof_size: usize,
    mut p_time: f64,
    merkle_tree_l: HashDigest,
    merkle_tree_h: HashDigest,
    fri_ctx: &mut FRIContext,
  ) -> bool {
    let command = format!("./fft_gkr {} log_fftgkr.txt", log_length - LOG_SLICE_NUMBER);

    let _output = Command::new("sh")
      .arg("-c")
      .arg(OsStr::from_bytes(command.as_bytes()))
      .output()
      .expect("Failed to execute command");

    let v_time_fft;
    let proof_size_fft;
    let p_time_fft;

    let mut file = match File::open("log_fftgkr.txt") {
      Err(err) => panic!("Couldn't open {}: {}", "log_fftgkr.txt", err),
      Ok(file) => file,
    };

    let mut contents = String::new();
    file
      .read_to_string(&mut contents)
      .expect("something went wrong reading the file");

    let mut iter = contents.split_whitespace();
    v_time_fft = iter.next().unwrap().parse::<f64>().unwrap();
    proof_size_fft = iter.next().unwrap().parse::<usize>().unwrap();
    p_time_fft = iter.next().unwrap().parse::<f64>().unwrap();

    v_time += v_time_fft as f64;
    proof_size += proof_size_fft;
    p_time += p_time_fft as f64;

    let com = self.pc_prover.commit_phase(log_length);
    let coef_slice_size = 1 << (log_length - LOG_SLICE_NUMBER);

    for _ in 0..33 {
      let slice_count = 1 << LOG_SLICE_NUMBER;
      let slice_size = 1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

      let mut t0: time::Instant;
      let mut time_span: f64;

      let inv_2 = FieldElement::from_real(2).inverse();

      let mut alpha_l: TripleVec;
      let mut alpha_h: TripleVec;
      let mut alpha: TripleVec = (vec![], vec![]);
      let mut beta: TripleVec = (vec![], vec![]);

      let mut s0 = FieldElement::default();
      let mut s1 = FieldElement::default();
      let mut pre_y = FieldElement::default();
      let mut root_of_unity = FieldElement::default();
      let mut y = FieldElement::default();

      let mut equ_beta: bool;
      assert!(log_length - LOG_SLICE_NUMBER > 0);
      let mut pow: u128 = 0;

      for i in 0..log_length - LOG_SLICE_NUMBER {
        t0 = time::Instant::now();

        if i == 0 {
          pow = rand::random::<u128>() % (1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i));
          while pow < (1 << (log_length - LOG_SLICE_NUMBER - i)) || pow % 2 == 1 {
            pow =
              rand::random::<u128>() % (1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i));
          }
          root_of_unity =
            FieldElement::get_root_of_unity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i)
              .unwrap();
          y = FieldElement::fast_pow(root_of_unity, pow);
        } else {
          root_of_unity = root_of_unity * root_of_unity;
          pow = pow % (1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i));
          pre_y = y;
          y = y * y;
        }

        assert_eq!(pow % 2, 0);

        let s0_pow = pow / 2;
        let s1_pow = (pow + (1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER - i))) / 2;

        s0 = root_of_unity.fast_pow(s0_pow);
        s1 = root_of_unity.fast_pow(s1_pow);

        let indicator;

        if i != 0 {
          assert!(s1 == pre_y || s0 == pre_y);
          if s1 == pre_y {
            indicator = 1;
          } else {
            indicator = 0;
          }
        }

        assert_eq!(s0 * s0, y);
        assert_eq!(s1 * s1, y);

        let new_size = 0;
        let gen_val = |alpha: &TripleVec, inv_mu: FieldElement, i: usize, j: usize| {
          (alpha.0[j].0 + alpha.0[j].1) * inv_2
            + (alpha.0[j].0 - alpha.0[j].1) * inv_2 * com.randomness[i] * inv_mu
        };

        if i == 0 {
          time_span = t0.elapsed().as_secs_f64();
          v_time += time_span;
          alpha_l = request_init_value_with_merkle(
            s0_pow.try_into().unwrap(),
            s1_pow.try_into().unwrap(),
            new_size,
            0,
            fri_ctx,
          );
          alpha_h = request_init_value_with_merkle(
            s0_pow.try_into().unwrap(),
            s1_pow.try_into().unwrap(),
            new_size,
            1,
            fri_ctx,
          );

          proof_size += new_size;

          t0 = time::Instant::now();

          let min_pow = |s0: u128, s1: u128| {
            if s0 < s1 {
              s0
            } else {
              s1
            }
          };

          if !verify_merkle(
            merkle_tree_l,
            &alpha_l.1,
            alpha_l.1.len(),
            min_pow(s0_pow, s1_pow),
            &alpha_l.0,
          ) {
            return false;
          }

          if !verify_merkle(
            merkle_tree_h,
            &alpha_h.1,
            alpha_h.1.len(),
            min_pow(s0_pow, s1_pow),
            &alpha_h.0,
          ) {
            return false;
          }

          v_time += t0.elapsed().as_secs_f64();
          beta = request_step_commit(0, (pow / 2).try_into().unwrap(), new_size, fri_ctx);

          proof_size += new_size;

          t0 = time::Instant::now();

          let inv_mu = root_of_unity.fast_pow((pow / 2) as u128).inverse();
          alpha.0.clear();
          alpha.1.clear();

          let mut rou = [FieldElement::default(); 2];
          let mut x = [FieldElement::default(); 2];
          let mut inv_x = [FieldElement::default(); 2];

          x[0] = FieldElement::get_root_of_unity(my_log(slice_size).unwrap()).unwrap();
          x[1] = FieldElement::get_root_of_unity(my_log(slice_size).unwrap()).unwrap();
          x[0] = x[0].fast_pow(s0_pow as u128);
          x[1] = x[1].fast_pow(s1_pow as u128);

          rou[0] = x[0].fast_pow((slice_size >> RS_CODE_RATE) as u128);
          rou[1] = x[1].fast_pow((slice_size >> RS_CODE_RATE) as u128);

          inv_x[0] = x[0].clone().inverse();
          inv_x[1] = x[1].clone().inverse();

          alpha.0.resize(
            slice_count,
            (FieldElement::default(), FieldElement::default()),
          );

          let mut tst0;
          let mut tst1;
          let mut x0;
          let mut x1;

          for j in 0..slice_count {
            tst0 = FieldElement::from_real(0);
            tst1 = FieldElement::from_real(0);
            x0 = FieldElement::from_real(1);
            x1 = FieldElement::from_real(1);

            for k in 0..(1 << (log_length - LOG_SLICE_NUMBER)) {
              tst0 = tst0 + x0 * public_array[k + j * coef_slice_size];
              x0 = x0 * x[0];
              tst1 = tst1 + x1 * public_array[k + j * coef_slice_size];
              x1 = x1 * x[1];
            }

            let one = FieldElement::from_real(1);
            {
              alpha.0[j].0 = alpha_l.0[j].0 * tst0 - (rou[0] - one) * alpha_h.0[j].0;
              alpha.0[j].0 = (alpha.0[j].0
                * FieldElement::from_real((slice_size >> RS_CODE_RATE) as u64)
                - all_sum[j])
                * inv_x[0];
              alpha.0[j].1 = alpha_l.0[j].1 * tst1 - (rou[1] - one) * alpha_h.0[j].1;
              alpha.0[j].1 = (alpha.0[j].1
                * FieldElement::from_real((slice_size >> RS_CODE_RATE) as u64)
                - all_sum[j])
                * inv_x[1];
            }

            if s0_pow > s1_pow {
              let mut a = alpha.0[j];
              std::mem::swap(&mut a.0, &mut a.1);
            }

            let p_val = gen_val(&alpha, inv_mu, i, j);

            if p_val != beta.0[j].0 && p_val != beta.0[j].1 {
              eprintln!("Fri check consistency first round fail {}", j);
              return false;
            }

            if p_val == beta.0[j].0 {
              equ_beta = false
            } else {
              equ_beta = true
            };
          }

          time_span = t0.elapsed().as_secs_f64();
        } else {
          time_span = t0.elapsed().as_secs_f64();
          v_time += time_span;

          alpha = beta.clone();
          beta = request_step_commit(i, (pow / 2).try_into().unwrap(), new_size, fri_ctx);

          proof_size += new_size;

          t0 = time::Instant::now();

          if !verify_merkle(
            com.commitment_hash[i],
            &beta.1,
            beta.1.len(),
            (pow / 2) as u128,
            &beta.0,
          ) {
            return false;
          }

          let inv_mu = root_of_unity.fast_pow((pow / 2) as u128).inverse();
          time_span = t0.elapsed().as_secs_f64();
          v_time += time_span;

          for j in 0..slice_count {
            let p_val_0 = gen_val(&alpha, inv_mu, i, j);
            let p_val_1 = (alpha.0[j].0 + alpha.0[j].1) * inv_2
              + (alpha.0[j].1 - alpha.0[j].0) * inv_2 * com.randomness[i] * inv_mu;

            if p_val_0 != beta.0[j].0
              && p_val_0 != beta.0[j].1
              && p_val_1 != beta.0[j].0
              && p_val_1 != beta.0[j].1
            {
              eprintln!("Fri check consistency first round fail {}", j);
              return false;
            }
          }
        }
      }

      for i in 0..slice_count {
        let template =
          fri_ctx.cpd.rs_codeword[com.mx_depth - 1][0 << (LOG_SLICE_NUMBER + 1) | i << 1 | 0];
        for j in 0..(1 << (RS_CODE_RATE - 1)) {
          if fri_ctx.cpd.rs_codeword[com.mx_depth - 1][j << (LOG_SLICE_NUMBER + 1) | i << 1 | 0]
            != template
          {
            eprintln!("Fri rs code check fail");
            return false;
          }
        }
      }
    }

    true
  }
}
