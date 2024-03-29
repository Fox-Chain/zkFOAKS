use std::time::Instant;
use std::{io::Write, mem};

use global::constants::{FE_REAL_ONE, FE_ZERO, LOG_SLICE_NUMBER, SLICE_NUMBER};
use infrastructure::my_hash::HashDigest;
use infrastructure::rs_polynomial::{inverse_fast_fourier_transform, ScratchPad};
use poly_commitment::PolyCommitVerifier;
use prime_field::FieldElement;

use crate::prover::SumcheckInitArgs;
use crate::{circuit_fast_track::LayeredCircuit, polynomial::QuadraticPoly, prover::ZkProver};

#[derive(Default, Debug)]
pub struct VerifierContext {
  pub q_eval_real: Vec<FieldElement>,
  pub q_eval_verifier: Vec<FieldElement>,
  pub q_ratio: Vec<FieldElement>,
}

#[derive(Default, Debug)]
pub struct ZkVerifier {
  pub proof_size: usize,
  pub v_time: f64,
  pub poly_verifier: PolyCommitVerifier,
  /** @name Randomness&Const
   * Storing randomness or constant for simplifying computation */
  beta_g_r0_first_half: Vec<FieldElement>,
  beta_g_r0_second_half: Vec<FieldElement>,
  beta_g_r1_first_half: Vec<FieldElement>,
  beta_g_r1_second_half: Vec<FieldElement>,
  beta_u_first_half: Vec<FieldElement>,
  beta_u_second_half: Vec<FieldElement>,
  beta_v_first_half: Vec<FieldElement>,
  beta_v_second_half: Vec<FieldElement>,

  beta_g_r0_block_first_half: Vec<FieldElement>,
  beta_g_r0_block_second_half: Vec<FieldElement>,
  beta_g_r1_block_first_half: Vec<FieldElement>,
  beta_g_r1_block_second_half: Vec<FieldElement>,
  beta_u_block_first_half: Vec<FieldElement>,
  beta_u_block_second_half: Vec<FieldElement>,
  beta_v_block_first_half: Vec<FieldElement>,
  beta_v_block_second_half: Vec<FieldElement>,

  pub a_c: LayeredCircuit,
  vpd_randomness: Vec<FieldElement>,
  one_minus_vpd_randomness: Vec<FieldElement>,
  pub ctx: VerifierContext,
}

pub struct PredicateArgs<'a> {
  depth: usize,
  r_0: &'a Vec<FieldElement>,
  r_1: &'a Vec<FieldElement>,
  r_u: &'a Vec<FieldElement>,
  r_v: &'a Vec<FieldElement>,
}

pub struct BetaInitArgs<'a> {
  depth: usize,
  alpha: FieldElement,
  beta: FieldElement,
  r_0: &'a Vec<FieldElement>,
  r_1: &'a Vec<FieldElement>,
  r_u: &'a Vec<FieldElement>,
  r_v: &'a Vec<FieldElement>,
  one_minus_r_0: &'a Vec<FieldElement>,
  one_minus_r_1: &'a Vec<FieldElement>,
  one_minus_r_u: &'a Vec<FieldElement>,
  one_minus_r_v: &'a Vec<FieldElement>,
}

impl ZkVerifier {
  pub fn new() -> Self { Default::default() }

  pub fn init_array(&mut self, max_bit_length: usize) {
    let first_half_len = max_bit_length / 2;
    let first_half_size = 1 << (max_bit_length / 2);
    let second_half_len = max_bit_length - first_half_len;
    let second_half_size = 1 << second_half_len;

    self.beta_g_r0_first_half = vec![FE_ZERO; first_half_size];
    self.beta_g_r0_second_half = vec![FE_ZERO; second_half_size];
    self.beta_g_r1_first_half = vec![FE_ZERO; first_half_size];
    self.beta_g_r1_second_half = vec![FE_ZERO; second_half_size];
    self.beta_v_first_half = vec![FE_ZERO; first_half_size];
    self.beta_v_second_half = vec![FE_ZERO; second_half_size];
    self.beta_u_first_half = vec![FE_ZERO; first_half_size];
    self.beta_u_second_half = vec![FE_ZERO; second_half_size];

    self.beta_g_r0_block_first_half = vec![FE_ZERO; first_half_size];
    self.beta_g_r0_block_second_half = vec![FE_ZERO; second_half_size];
    self.beta_g_r1_block_first_half = vec![FE_ZERO; first_half_size];
    self.beta_g_r1_block_second_half = vec![FE_ZERO; second_half_size];
    self.beta_v_block_first_half = vec![FE_ZERO; first_half_size];
    self.beta_v_block_second_half = vec![FE_ZERO; second_half_size];
    self.beta_u_block_first_half = vec![FE_ZERO; first_half_size];
    self.beta_u_block_second_half = vec![FE_ZERO; second_half_size];
  }

  pub fn verify(
    &mut self,
    bit_length: usize,
    inputs: Vec<FieldElement>,
    query_count: usize,
    combined_codeword: Vec<FieldElement>,
    q: Vec<usize>,
  ) -> (bool, f64) {
    // ZkProver initialization
    let mut zk_prover = ZkProver::new();
    zk_prover.init_array(bit_length, self.a_c.clone());
    zk_prover.get_witness(inputs);

    // Times
    let (mut verification_time, mut predicates_calc_time, mut verification_rdl_time) =
      (0.0, 0.0, 0.0);

    //Below function is not implemented neither in virgo repo nor orion repo
    //prime_field::init_random();

    //Below function is not implemented neither in virgo repo nor orion repo
    //self.prover.unwrap().proof_init();

    let result = zk_prover.evaluate();
    let mut alpha = FE_REAL_ONE;
    let mut beta = FE_ZERO;

    //	random_oracle oracle; // Orion just declare the variable but dont use it
    let capacity = self.a_c.circuit[self.a_c.total_depth - 1].bit_length;

    let mut r_0 = generate_randomness(capacity);
    let mut r_1 = generate_randomness(capacity);

    //todo: Refactor parallel
    let mut one_minus_r_0 = r_0.iter().map(|x| FE_REAL_ONE - *x).collect::<Vec<_>>();
    let mut one_minus_r_1 = r_1.iter().map(|x| FE_REAL_ONE - *x).collect::<Vec<_>>();

    let t_a = Instant::now();

    println!("Calc V_output(r)");
    assert_eq!(result.len(), 1 << capacity);
    let mut alpha_beta_sum = zk_prover.v_res(&one_minus_r_0, &r_0, result);

    let time_span = t_a.elapsed();
    println!("    Time:: {}", time_span.as_secs_f64());

    let mut direct_relay_value: FieldElement;

    for i in (1..=(self.a_c.total_depth - 1)).rev() {
      let previous_bit_length = self.a_c.circuit[i - 1].bit_length;

      zk_prover.sumcheck_init(SumcheckInitArgs {
        sumcheck_layer_id: i,
        length_g: self.a_c.circuit[i].bit_length,
        length_v: previous_bit_length,
        alpha,
        beta,
        r_0: r_0.clone(),
        r_1: r_1.clone(),
        one_minus_r_0: one_minus_r_0.clone(),
        one_minus_r_1: one_minus_r_1.clone(),
      });

      zk_prover.sumcheck_phase1_init();

      let mut previous_random = FE_ZERO;

      //next level random
      let r_u = generate_randomness(previous_bit_length);
      let mut r_v = generate_randomness(previous_bit_length);

      direct_relay_value =
        alpha * self.direct_relay(i, &r_0, &r_u) + beta * self.direct_relay(i, &r_1, &r_u);

      if i == 1 {
        r_v.fill(FE_ZERO);
      }

      //V should test the maskR for two points, V does random linear combination of these points first
      //let random_combine = generate_randomness(1)[0];       // never used

      //Every time all one test to V, V needs to do a linear combination for security.
      //let linear_combine = generate_randomness(1)[0]; // mem leak // never used

      let one_minus_r_u: Vec<FieldElement> = r_u.iter().map(|x| FE_REAL_ONE - *x).collect();
      let one_minus_r_v: Vec<FieldElement> = r_v.iter().map(|x| FE_REAL_ONE - *x).collect();

      for (j, elem) in r_u.iter().enumerate() {
        let poly = zk_prover.sumcheck_phase1_update(previous_random, j);
        self.proof_size += mem::size_of::<QuadraticPoly>();
        previous_random = *elem;
        let eval_zero = poly.eval(&FE_ZERO);
        let eval_one = poly.eval(&FE_REAL_ONE);

        assert_eq!(
          eval_zero + eval_one,
          alpha_beta_sum,
          "Verification fail, phase1, circuit {}, current bit {}",
          i,
          j
        );

        alpha_beta_sum = poly.eval(elem);
      }
      zk_prover.v_u = zk_prover.v_mult_add[0].eval(previous_random);
      zk_prover.sumcheck_phase2_init(&r_u, &one_minus_r_u);
      let mut previous_random = FE_ZERO;
      for (j, elem) in r_v.iter_mut().enumerate() {
        if i == 1 {
          *elem = FE_ZERO;
        }
        let poly = zk_prover.sumcheck_phase2_update(previous_random, j);
        self.proof_size += mem::size_of::<QuadraticPoly>();
        //poly.c = poly.c; ???

        previous_random = *elem;

        assert_eq!(
          poly.eval(&FE_ZERO) + poly.eval(&FE_REAL_ONE) + direct_relay_value * zk_prover.v_u,
          alpha_beta_sum,
          "Verification fail, phase2, circuit {}, current bit {}",
          i,
          j
        );

        alpha_beta_sum = poly.eval(elem) + direct_relay_value * zk_prover.v_u;
      }
      //Add one more round for maskR
      //quadratic_poly poly p->sumcheck_finalroundR(previous_random, C.current[i -
      // 1].bit_length);

      let final_claims = zk_prover.sumcheck_finalize(previous_random);

      let v_u = final_claims.0;
      let v_v = final_claims.1;

      let predicates_calc = Instant::now();
      self.beta_init(BetaInitArgs {
        depth: i,
        alpha,
        beta,
        r_0: &r_0,
        r_1: &r_1,
        r_u: &r_u,
        r_v: &r_v,
        one_minus_r_0: &one_minus_r_0,
        one_minus_r_1: &one_minus_r_1,
        one_minus_r_u: &one_minus_r_u,
        one_minus_r_v: &one_minus_r_v,
      });

      let predicates_value = self.predicates(PredicateArgs {
        depth: i,
        r_0: &r_0,
        r_1: &r_1,
        r_u: &r_u,
        r_v: &r_v,
      });

      let predicates_calc_span = predicates_calc.elapsed();
      if !self.a_c.circuit[i].is_parallel {
        verification_rdl_time += predicates_calc_span.as_secs_f64();
      }
      verification_time += predicates_calc_span.as_secs_f64();
      predicates_calc_time += predicates_calc_span.as_secs_f64();

      let mult_value = predicates_value[1];
      let add_value = predicates_value[0];
      let not_value = predicates_value[6];
      let minus_value = predicates_value[7];
      let xor_value = predicates_value[8];
      let naab_value = predicates_value[9];
      let sum_value = predicates_value[5];
      let relay_value = predicates_value[10];
      let exp_sum_value = predicates_value[12];
      let bit_test_value = predicates_value[13];
      let custom_comb_value = predicates_value[14];

      let mut r = r_u.clone();
      r.reserve(r_v.len());
      r.extend(r_v.clone());

      if alpha_beta_sum
        != (add_value * (v_u + v_v)
          + mult_value * v_u * v_v
          + not_value * (FE_REAL_ONE - v_u)
          + minus_value * (v_u - v_v)
          + xor_value * (v_u + v_v - FieldElement::from_real(2) * v_u * v_v)
          + naab_value * (v_v - v_u * v_v)
          + sum_value * v_u
          + custom_comb_value * v_u
          + relay_value * v_u
          + exp_sum_value * v_u
          + bit_test_value * (FE_REAL_ONE - v_v) * v_u)
          + direct_relay_value * v_u
      {
        eprintln!("Verification fail, semi final, circuit level {}", i,);
        return (false, 0.0);
      }

      alpha = FieldElement::new_random();
      beta = FieldElement::new_random();

      if i != 1 {
        alpha_beta_sum = alpha * v_u + beta * v_v;
      } else {
        alpha_beta_sum = v_u;
      }
      r_0 = r_u;
      r_1 = r_v;
      one_minus_r_0 = one_minus_r_u;
      one_minus_r_1 = one_minus_r_v;
    }

    println!("GKR Prove Time: {}", zk_prover.total_time);
    let mut all_sum = vec![FE_ZERO; SLICE_NUMBER];
    println!("GKR witness size: {}", self.a_c.circuit[0].gates.len());
    let merkle_root_l = zk_prover
      .poly_prover
      .commit_private_array(&zk_prover.circuit_value[0], self.a_c.circuit[0].bit_length);
    self.ctx.q_eval_real = vec![FE_ZERO; self.a_c.circuit[0].gates.len()];
    self.dfs_for_public_eval(
      0usize,
      FE_REAL_ONE,
      &r_0,
      &one_minus_r_0,
      self.a_c.circuit[0].bit_length,
      0,
    );

    let merkle_root_h = zk_prover.poly_prover.commit_public_array(
      &self.ctx.q_eval_real,
      self.a_c.circuit[0].bit_length,
      alpha_beta_sum,
      &mut all_sum,
    );

    self.proof_size += 2 * mem::size_of::<HashDigest>();
    self.vpd_randomness = r_0.clone();
    self.one_minus_vpd_randomness = one_minus_r_0.clone();
    self.poly_verifier.pc_prover = zk_prover.poly_prover.clone();

    let public_array = self.public_array_prepare(
      &r_0,
      one_minus_r_0.clone(),
      self.a_c.circuit[0].bit_length,
      &mut zk_prover.poly_prover.scratch_pad,
    );

    let (v_time, proof_size, p_time, input_0_verify) = self.poly_verifier.verify_poly_commitment(
      &all_sum,
      self.a_c.circuit[0].bit_length,
      &public_array,
      merkle_root_l,
      merkle_root_h,
    );

    verification_time += v_time;
    self.proof_size += proof_size;
    zk_prover.total_time += p_time;

    zk_prover.poly_prover.total_time_pc_p += self.poly_verifier.pc_prover.total_time_pc_p;

    if !input_0_verify {
      eprintln!("Verification fail, input vpd");
      return (false, 0.0);
    }

    println!("Verification pass");
    println!("Prove time {}", zk_prover.total_time);
    println!("Verification rdl time {}", verification_rdl_time);
    println!(
      "Verification time {}",
      verification_time - verification_rdl_time
    );
    self.v_time = verification_time - verification_rdl_time;
    println!("Proof size (bytes) {}", self.proof_size);

    let output_path = "log.txt";

    ZkVerifier::write_file(
      &String::from(output_path),
      zk_prover.total_time,
      verification_time,
      predicates_calc_time,
      verification_rdl_time,
      self.proof_size,
    )
    .expect("Error while writing file");

    // Code added from tensor_product()
    let sample_t0 = Instant::now();
    for i in 0..query_count {
      assert_eq!(
        zk_prover.circuit_value[zk_prover.a_c.total_depth - 1][i],
        combined_codeword[q[i]],
      );
    }
    let time_span = sample_t0.elapsed().as_secs_f64();

    (true, time_span)
  }

  pub fn write_file(
    output_path: &str,
    total_time: f64,
    verification_time: f64,
    predicates_calc_time: f64,
    verification_rdl_time: f64,
    proof_size: usize,
  ) -> Result<(), std::io::Error> {
    let full_path = std::path::Path::new(output_path);
    std::fs::create_dir_all(
      full_path
        .parent()
        .expect("Failed to retrieve parent directory"),
    )?;
    let mut result_file = std::fs::File::create(full_path)?;
    writeln!(
      result_file,
      "{} {} {} {} {}",
      total_time, verification_time, predicates_calc_time, verification_rdl_time, proof_size
    )?;
    Ok(())
  }

  pub fn public_array_prepare(
    &mut self,
    r: &Vec<FieldElement>,
    one_minus_r: Vec<FieldElement>,
    log_length: usize,
    scratch_pad: &mut ScratchPad,
  ) -> Vec<FieldElement> {
    self.ctx.q_eval_verifier = vec![FE_ZERO; 1 << (log_length - LOG_SLICE_NUMBER)];
    self.ctx.q_ratio = vec![FE_ZERO; 1 << LOG_SLICE_NUMBER];

    Self::dfs_ratio(
      self,
      0,
      FE_REAL_ONE,
      &r[log_length - LOG_SLICE_NUMBER..].to_vec(),
      one_minus_r[log_length - LOG_SLICE_NUMBER..].to_vec(),
      0,
    );
    Self::dfs_coef(
      self,
      0,
      FE_REAL_ONE,
      r,
      one_minus_r,
      0,
      log_length - LOG_SLICE_NUMBER,
    );
    let mut q_coef_verifier = vec![FE_ZERO; 1 << (log_length - LOG_SLICE_NUMBER)];

    inverse_fast_fourier_transform(
      scratch_pad,
      &self.ctx.q_eval_verifier,
      1 << (log_length - LOG_SLICE_NUMBER),
      1 << (log_length - LOG_SLICE_NUMBER),
      FieldElement::get_root_of_unity(log_length - LOG_SLICE_NUMBER)
        .expect("Failed to retrieve root of unity"),
      &mut q_coef_verifier,
    );

    let mut q_coef_arr = vec![FE_ZERO; 1 << log_length];
    let coef_slice_size = 1 << (log_length - LOG_SLICE_NUMBER);
    for i in 0..(1 << LOG_SLICE_NUMBER) {
      for j in 0..coef_slice_size {
        q_coef_arr[i * coef_slice_size + j] = q_coef_verifier[j] * self.ctx.q_ratio[i];
        assert_eq!(
          self.ctx.q_eval_real[i * coef_slice_size + j],
          self.ctx.q_ratio[i] * self.ctx.q_eval_verifier[j]
        );
      }
    }
    q_coef_arr
  }

  pub fn dfs_coef(
    &mut self,
    dep: usize,
    val: FieldElement,
    r: &Vec<FieldElement>,
    one_minus_r: Vec<FieldElement>,
    pos: usize,
    r_len: usize,
  ) {
    if dep == r_len {
      self.ctx.q_eval_verifier[pos] = val;
    } else {
      Self::dfs_coef(
        self,
        dep + 1,
        val * one_minus_r[r_len - 1 - dep],
        r,
        one_minus_r.clone(),
        pos << 1,
        r_len,
      );
      Self::dfs_coef(
        self,
        dep + 1,
        val * r[r_len - 1 - dep],
        r,
        one_minus_r,
        pos << 1 | 1,
        r_len,
      );
    }
  }

  pub fn dfs_ratio(
    &mut self,
    dep: usize,
    val: FieldElement,
    r: &Vec<FieldElement>,
    one_minus_r: Vec<FieldElement>,
    pos: usize,
  ) {
    if dep == LOG_SLICE_NUMBER {
      self.ctx.q_ratio[pos] = val;
    } else {
      Self::dfs_ratio(
        self,
        dep + 1,
        val * one_minus_r[LOG_SLICE_NUMBER - 1 - dep],
        r,
        one_minus_r.clone(),
        pos << 1,
      );
      Self::dfs_ratio(
        self,
        dep + 1,
        val * r[LOG_SLICE_NUMBER - 1 - dep],
        r,
        one_minus_r,
        pos << 1 | 1,
      );
    }
  }

  pub fn dfs_for_public_eval(
    &mut self,
    dep: usize,
    val: FieldElement,
    r_0: &Vec<FieldElement>,
    one_minus_r_0: &Vec<FieldElement>,
    r_0_len: usize,
    pos: usize,
  ) {
    if dep == r_0_len {
      self.ctx.q_eval_real[pos] = val;
    } else {
      Self::dfs_for_public_eval(
        self,
        dep + 1,
        val * (*one_minus_r_0)[r_0_len - 1 - dep],
        r_0,
        one_minus_r_0,
        r_0_len,
        pos << 1,
      );
      Self::dfs_for_public_eval(
        self,
        dep + 1,
        val * r_0[r_0_len - 1 - dep],
        r_0,
        one_minus_r_0,
        r_0_len,
        pos << 1 | 1,
      );
    }
  }

  pub fn direct_relay(
    &mut self,
    depth: usize,
    r_g: &[FieldElement],
    r_u: &[FieldElement],
  ) -> FieldElement {
    if depth != 1 {
      FE_ZERO
    } else {
      let ret = FE_REAL_ONE;
      let result = ret
        * r_g
          .iter()
          .zip(r_u.iter())
          .map(|(&g, &u)| FE_REAL_ONE - g - u + FieldElement::from_real(2) * g * u)
          .fold(FE_REAL_ONE, |acc, val| acc * val);
      result
    }
  }

  pub fn beta_init(
    &mut self,
    BetaInitArgs {
      depth,
      alpha,
      beta,
      r_0,
      r_1,
      r_u,
      r_v,
      one_minus_r_0,
      one_minus_r_1,
      one_minus_r_u,
      one_minus_r_v,
    }: BetaInitArgs,
  ) {
    let debug_mode = false;
    if !self.a_c.circuit[depth].is_parallel || debug_mode {
      self.beta_g_r0_first_half[0] = alpha;
      self.beta_g_r1_first_half[0] = beta;
      self.beta_g_r0_second_half[0] = FE_REAL_ONE;
      self.beta_g_r1_second_half[0] = FE_REAL_ONE;

      let first_half_len = self.a_c.circuit[depth].bit_length / 2;
      let second_half_len = self.a_c.circuit[depth].bit_length - first_half_len;

      for i in 0..first_half_len {
        let r0 = r_0[i];
        let r1 = r_1[i];
        let or0 = one_minus_r_0[i];
        let or1 = one_minus_r_1[i];

        for j in 0..(1 << i) {
          self.beta_g_r0_first_half[j | (1 << i)] = self.beta_g_r0_first_half[j] * r0;
          self.beta_g_r1_first_half[j | (1 << i)] = self.beta_g_r1_first_half[j] * r1;
        }

        for j in 0..(1 << i) {
          self.beta_g_r0_first_half[j] = self.beta_g_r0_first_half[j] * or0;
          self.beta_g_r1_first_half[j] = self.beta_g_r1_first_half[j] * or1;
        }
      }

      for i in 0..second_half_len {
        let r0 = r_0[i + first_half_len];
        let r1 = r_1[i + first_half_len];
        let or0 = one_minus_r_0[i + first_half_len];
        let or1 = one_minus_r_1[i + first_half_len];

        for j in 0..(1 << i) {
          self.beta_g_r0_second_half[j | (1 << i)] = self.beta_g_r0_second_half[j] * r0;
          self.beta_g_r1_second_half[j | (1 << i)] = self.beta_g_r1_second_half[j] * r1;
        }

        for j in 0..(1 << i) {
          self.beta_g_r0_second_half[j] = self.beta_g_r0_second_half[j] * or0;
          self.beta_g_r1_second_half[j] = self.beta_g_r1_second_half[j] * or1;
        }
      }

      self.beta_u_first_half[0] = FE_REAL_ONE;
      self.beta_v_first_half[0] = FE_REAL_ONE;
      self.beta_u_second_half[0] = FE_REAL_ONE;
      self.beta_v_second_half[0] = FE_REAL_ONE;
      let first_half_len = self.a_c.circuit[depth - 1].bit_length / 2;
      let second_half_len = self.a_c.circuit[depth - 1].bit_length - first_half_len;

      for i in 0..first_half_len {
        let ru = r_u[i];
        let rv = r_v[i];
        let oru = one_minus_r_u[i];
        let orv = one_minus_r_v[i];

        for j in 0..(1 << i) {
          self.beta_u_first_half[j | (1 << i)] = self.beta_u_first_half[j] * ru;
          self.beta_v_first_half[j | (1 << i)] = self.beta_v_first_half[j] * rv;
        }

        for j in 0..(1 << i) {
          self.beta_u_first_half[j] = self.beta_u_first_half[j] * oru;
          self.beta_v_first_half[j] = self.beta_v_first_half[j] * orv;
        }
      }

      for i in 0..second_half_len {
        let ru = r_u[i + first_half_len];
        let rv = r_v[i + first_half_len];
        let oru = one_minus_r_u[i + first_half_len];
        let orv = one_minus_r_v[i + first_half_len];

        for j in 0..(1 << i) {
          self.beta_u_second_half[j | (1 << i)] = self.beta_u_second_half[j] * ru;
          self.beta_v_second_half[j | (1 << i)] = self.beta_v_second_half[j] * rv;
        }

        for j in 0..(1 << i) {
          self.beta_u_second_half[j] = self.beta_u_second_half[j] * oru;
          self.beta_v_second_half[j] = self.beta_v_second_half[j] * orv;
        }
      }
    }

    if self.a_c.circuit[depth].is_parallel {
      self.beta_g_r0_block_first_half[0] = alpha;
      self.beta_g_r1_block_first_half[0] = beta;
      self.beta_g_r0_block_second_half[0] = FE_REAL_ONE;
      self.beta_g_r1_block_second_half[0] = FE_REAL_ONE;

      let first_half_len = self.a_c.circuit[depth - 1].log_block_size / 2;
      let second_half_len = self.a_c.circuit[depth - 1].log_block_size - first_half_len;

      for i in 0..first_half_len {
        let r0 = r_0[i + first_half_len];
        let r1 = r_1[i + first_half_len];
        let or0 = one_minus_r_0[i + first_half_len];
        let or1 = one_minus_r_1[i + first_half_len];

        for j in 0..(1 << i) {
          self.beta_g_r0_block_first_half[j | (1 << i)] = self.beta_g_r0_block_first_half[j] * r0;
          self.beta_g_r1_block_first_half[j | (1 << i)] = self.beta_g_r1_block_first_half[j] * r1;
        }

        for j in 0..(1 << i) {
          self.beta_g_r0_block_first_half[j] = self.beta_g_r0_block_first_half[j] * or0;
          self.beta_g_r1_block_first_half[j] = self.beta_g_r1_block_first_half[j] * or1;
        }
      }

      for i in 0..second_half_len {
        let r0 = r_0[i + first_half_len];
        let r1 = r_1[i + first_half_len];
        let or0 = one_minus_r_0[i + first_half_len];
        let or1 = one_minus_r_1[i + first_half_len];

        for j in 0..(1 << i) {
          self.beta_g_r0_block_second_half[j | (1 << i)] = self.beta_g_r0_block_second_half[j] * r0;
          self.beta_g_r1_block_second_half[j | (1 << i)] = self.beta_g_r1_block_second_half[j] * r1;
        }

        for j in 0..(1 << 1) {
          self.beta_g_r0_block_second_half[j] = self.beta_g_r0_block_second_half[j] * or0;
          self.beta_g_r1_block_second_half[j] = self.beta_g_r1_block_second_half[j] * or1;
        }
      }

      self.beta_u_block_first_half[0] = FE_REAL_ONE;
      self.beta_v_block_first_half[0] = FE_REAL_ONE;
      self.beta_u_block_second_half[0] = FE_REAL_ONE;
      self.beta_v_block_second_half[0] = FE_REAL_ONE;
      let first_half_len = self.a_c.circuit[depth - 1].bit_length / 2;
      let second_half_len = self.a_c.circuit[depth - 1].bit_length - first_half_len;

      for i in 0..first_half_len {
        let ru = r_u[i];
        let rv = r_v[i];
        let oru = one_minus_r_u[i];
        let orv = one_minus_r_v[i];

        for j in 0..(1 << i) {
          self.beta_u_block_first_half[j | (1 << i)] = self.beta_u_block_first_half[j] * ru;
          self.beta_v_block_first_half[j | (1 << i)] = self.beta_v_block_first_half[j] * rv;
        }

        for j in 0..(1 << i) {
          self.beta_u_block_first_half[j] = self.beta_u_block_first_half[j] * oru;
          self.beta_v_block_first_half[j] = self.beta_v_block_first_half[j] * orv;
        }
      }

      for i in 0..second_half_len {
        let ru = r_u[i + first_half_len];
        let rv = r_v[i + first_half_len];
        let oru = one_minus_r_u[i + first_half_len];
        let orv = one_minus_r_v[i + first_half_len];

        for j in 0..(1 << i) {
          self.beta_u_block_second_half[j | (1 << i)] = self.beta_u_block_second_half[j] * ru;
          self.beta_v_block_second_half[j | (1 << i)] = self.beta_v_block_second_half[j] * rv;
        }

        for j in 0..(1 << i) {
          self.beta_u_block_second_half[j] = self.beta_u_block_second_half[j] * oru;
          self.beta_v_block_second_half[j] = self.beta_v_block_second_half[j] * orv;
        }
      }
    }
  }

  pub fn predicates(
    &mut self,
    PredicateArgs {
      depth,
      r_0,
      r_1,
      r_u,
      r_v,
    }: PredicateArgs,
  ) -> Vec<FieldElement> {
    let gate_type_count = 15;

    let mut ret_para = vec![FE_ZERO; gate_type_count];
    let mut ret = vec![FE_ZERO; gate_type_count];

    if depth == 1 {
      return ret;
    }

    let debug_mode = false;
    if self.a_c.circuit[depth].is_parallel {
      let first_half_g = self.a_c.circuit[depth].log_block_size / 2;
      let first_half_uv = self.a_c.circuit[depth - 1].log_block_size / 2;

      let mut one_block_alpha = vec![FE_ZERO; gate_type_count];
      let mut one_block_beta = vec![FE_ZERO; gate_type_count];

      assert_eq!(
        (1 << self.a_c.circuit[depth].log_block_size),
        self.a_c.circuit[depth].block_size
      );

      for i in 0..self.a_c.circuit[depth].log_block_size {
        let mut g = i;
        let mut u = self.a_c.circuit[depth].gates[i].u;
        let mut v = self.a_c.circuit[depth].gates[i].v;
        g &= (1 << self.a_c.circuit[depth].log_block_size) - 1;
        u &= (1 << self.a_c.circuit[depth - 1].log_block_size) - 1;
        v &= (1 << self.a_c.circuit[depth - 1].log_block_size) - 1;

        match self.a_c.circuit[depth].gates[i].ty {
          0 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            let uv_value = (self.beta_u_block_first_half[u_first_half]
              * self.beta_u_block_second_half[u_second_half])
              * (self.beta_v_block_first_half[v_first_half]
                * self.beta_v_block_second_half[v_second_half]);
            one_block_alpha[0] = one_block_alpha[0]
              + (self.beta_g_r0_block_first_half[g_first_half]
                * self.beta_g_r0_block_second_half[g_second_half])
                * uv_value;
            one_block_beta[0] = one_block_beta[0]
              + (self.beta_g_r1_block_first_half[g_first_half]
                * self.beta_g_r1_block_second_half[g_second_half])
                * uv_value;
          }
          1 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            let uv_value = (self.beta_u_block_first_half[u_first_half]
              * self.beta_u_block_second_half[u_second_half])
              * (self.beta_v_block_first_half[v_first_half]
                * self.beta_v_block_second_half[v_second_half]);
            one_block_alpha[1] = one_block_alpha[1]
              + (self.beta_g_r0_block_first_half[g_first_half]
                * self.beta_g_r0_block_second_half[g_second_half])
                * uv_value;
            one_block_beta[1] = one_block_beta[1]
              + (self.beta_g_r1_block_first_half[g_first_half]
                * self.beta_g_r1_block_second_half[g_second_half])
                * uv_value;
          }
          2 => {}
          3 => {}
          4 => {}
          5 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;

            let beta_g_val_alpha = self.beta_g_r0_block_first_half[g_first_half]
              * self.beta_g_r0_block_second_half[g_second_half];
            let beta_g_val_beta = self.beta_g_r1_block_first_half[g_first_half]
              * self.beta_g_r1_block_second_half[g_second_half];
            let beta_v_0 = self.beta_v_block_first_half[0] * self.beta_v_block_second_half[0];
            for j in u..v {
              let u_first_half = j & ((1 << first_half_uv) - 1);
              let u_second_half = j >> first_half_uv;
              one_block_alpha[5] = one_block_alpha[5]
                + beta_g_val_alpha
                  * beta_v_0
                  * (self.beta_u_block_first_half[u_first_half]
                    * self.beta_u_block_second_half[u_second_half]);
              one_block_beta[5] = one_block_beta[5]
                + beta_g_val_beta
                  * beta_v_0
                  * (self.beta_u_block_first_half[u_first_half]
                    * self.beta_u_block_second_half[u_second_half]);
            }
          }
          12 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;

            let beta_g_val_alpha = self.beta_g_r0_block_first_half[g_first_half]
              * self.beta_g_r0_block_second_half[g_second_half];
            let beta_g_val_beta = self.beta_g_r1_block_first_half[g_first_half]
              * self.beta_g_r1_block_second_half[g_second_half];
            let mut beta_v_0 = self.beta_v_block_first_half[0] * self.beta_v_block_second_half[0];
            for j in u..=v {
              let u_first_half = j & ((1 << first_half_uv) - 1);
              let u_second_half = j >> first_half_uv;
              one_block_alpha[12] = one_block_alpha[12]
                + beta_g_val_alpha
                  * beta_v_0
                  * (self.beta_u_block_first_half[u_first_half]
                    * self.beta_u_block_second_half[u_second_half]);
              one_block_beta[12] = one_block_beta[12]
                + beta_g_val_beta
                  * beta_v_0
                  * (self.beta_u_block_first_half[u_first_half]
                    * self.beta_u_block_second_half[u_second_half]);

              beta_v_0 = beta_v_0 + beta_v_0;
            }
          }
          6 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            let uv_value = (self.beta_u_block_first_half[u_first_half]
              * self.beta_u_block_second_half[u_second_half])
              * (self.beta_v_block_first_half[v_first_half]
                * self.beta_v_block_second_half[v_second_half]);
            one_block_alpha[6] = one_block_alpha[6]
              + (self.beta_g_r0_block_first_half[g_first_half]
                * self.beta_g_r0_block_second_half[g_second_half])
                * uv_value;
            one_block_beta[6] = one_block_beta[6]
              + (self.beta_g_r1_block_first_half[g_first_half]
                * self.beta_g_r1_block_second_half[g_second_half])
                * uv_value;
          }
          7 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            let uv_value = (self.beta_u_block_first_half[u_first_half]
              * self.beta_u_block_second_half[u_second_half])
              * (self.beta_v_block_first_half[v_first_half]
                * self.beta_v_block_second_half[v_second_half]);
            one_block_alpha[7] = one_block_alpha[7]
              + (self.beta_g_r0_block_first_half[g_first_half]
                * self.beta_g_r0_block_second_half[g_second_half])
                * uv_value;
            one_block_beta[7] = one_block_beta[7]
              + (self.beta_g_r1_block_first_half[g_first_half]
                * self.beta_g_r1_block_second_half[g_second_half])
                * uv_value;
          }
          8 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            let uv_value = (self.beta_u_block_first_half[u_first_half]
              * self.beta_u_block_second_half[u_second_half])
              * (self.beta_v_block_first_half[v_first_half]
                * self.beta_v_block_second_half[v_second_half]);
            one_block_alpha[8] = one_block_alpha[8]
              + (self.beta_g_r0_block_first_half[g_first_half]
                * self.beta_g_r0_block_second_half[g_second_half])
                * uv_value;
            one_block_beta[8] = one_block_beta[8]
              + (self.beta_g_r1_block_first_half[g_first_half]
                * self.beta_g_r1_block_second_half[g_second_half])
                * uv_value;
          }
          9 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            let uv_value = (self.beta_u_block_first_half[u_first_half]
              * self.beta_u_block_second_half[u_second_half])
              * (self.beta_v_block_first_half[v_first_half]
                * self.beta_v_block_second_half[v_second_half]);
            one_block_alpha[9] = one_block_alpha[9]
              + (self.beta_g_r0_block_first_half[g_first_half]
                * self.beta_g_r0_block_second_half[g_second_half])
                * uv_value;
            one_block_beta[9] = one_block_beta[9]
              + (self.beta_g_r1_block_first_half[g_first_half]
                * self.beta_g_r1_block_second_half[g_second_half])
                * uv_value;
          }
          10 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            let uv_value = (self.beta_u_block_first_half[u_first_half]
              * self.beta_u_block_second_half[u_second_half])
              * (self.beta_v_block_first_half[v_first_half]
                * self.beta_v_block_second_half[v_second_half]);
            one_block_alpha[10] = one_block_alpha[10]
              + (self.beta_g_r0_block_first_half[g_first_half]
                * self.beta_g_r0_block_second_half[g_second_half])
                * uv_value;
            one_block_beta[10] = one_block_beta[10]
              + (self.beta_g_r1_block_first_half[g_first_half]
                * self.beta_g_r1_block_second_half[g_second_half])
                * uv_value;
          }
          13 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            let uv_value = (self.beta_u_block_first_half[u_first_half]
              * self.beta_u_block_second_half[u_second_half])
              * (self.beta_v_block_first_half[v_first_half]
                * self.beta_v_block_second_half[v_second_half]);
            one_block_alpha[13] = one_block_alpha[13]
              + (self.beta_g_r0_block_first_half[g_first_half]
                * self.beta_g_r0_block_second_half[g_second_half])
                * uv_value;
            one_block_beta[13] = one_block_beta[13]
              + (self.beta_g_r1_block_first_half[g_first_half]
                * self.beta_g_r1_block_second_half[g_second_half])
                * uv_value;
          }
          _ => {}
        }
      }
      for i in 0..self.a_c.circuit[depth].repeat_num {
        let mut prefix_alpha = FE_REAL_ONE;
        let mut prefix_beta = FE_REAL_ONE;
        let mut prefix_alpha_v0 = FE_REAL_ONE;
        let mut prefix_beta_v0 = FE_REAL_ONE;

        for j in 0..self.a_c.circuit[depth].log_repeat_num {
          if (i >> j) > 0 {
            let uv_value = r_u[j + self.a_c.circuit[depth - 1].log_block_size]
              * r_v[j + self.a_c.circuit[depth - 1].log_block_size];
            prefix_alpha =
              prefix_alpha * r_0[j + self.a_c.circuit[depth].log_block_size] * uv_value;
            prefix_beta = prefix_beta * r_1[j + self.a_c.circuit[depth].log_block_size] * uv_value;

            let uv_value_v0 = r_u[j + self.a_c.circuit[depth - 1].log_block_size]
              * (FE_REAL_ONE - r_v[j + self.a_c.circuit[depth - 1].log_block_size]);

            prefix_alpha_v0 =
              prefix_alpha_v0 * r_0[j + self.a_c.circuit[depth].log_block_size] * uv_value_v0;
            prefix_beta_v0 =
              prefix_beta_v0 * r_1[j + self.a_c.circuit[depth].log_block_size] * uv_value_v0;
          } else {
            let uv_value = (FE_REAL_ONE - r_u[j + self.a_c.circuit[depth - 1].log_block_size])
              * (FE_REAL_ONE - r_v[j + self.a_c.circuit[depth - 1].log_block_size]);
            prefix_alpha = prefix_alpha
              * (FE_REAL_ONE - r_0[j + self.a_c.circuit[depth].log_block_size])
              * uv_value;
            prefix_beta = prefix_beta
              * (FE_REAL_ONE - r_1[j + self.a_c.circuit[depth].log_block_size])
              * uv_value;
          }
        }

        (0..gate_type_count).for_each(|j| {
          let update_alpha = if j == 6 || j == 10 || j == 5 || j == 12 {
            prefix_alpha_v0
          } else {
            prefix_alpha
          };

          let update_beta = if j == 6 || j == 10 || j == 5 || j == 12 {
            prefix_beta_v0
          } else {
            prefix_beta
          };

          ret_para[j] =
            ret_para[j] + update_alpha * one_block_alpha[j] + update_beta * one_block_beta[j];
        });
      }
      if !debug_mode {
        ret = ret_para.clone();
      }
    }
    if !self.a_c.circuit[depth].is_parallel || debug_mode {
      let first_half_g = self.a_c.circuit[depth].bit_length / 2;
      let first_half_uv = self.a_c.circuit[depth - 1].bit_length / 2;

      let mut tmp_u_val = vec![FE_ZERO; self.a_c.circuit[depth - 1].gates.len()];
      let zero_v = self.beta_v_first_half[0] * self.beta_v_second_half[0];
      let mut relay_set = false;
      for i in 0..(self.a_c.circuit[depth].gates.len()) {
        let g = i;
        let u = self.a_c.circuit[depth].gates[i].u;
        let v = self.a_c.circuit[depth].gates[i].v;

        let g_first_half = g & ((1 << first_half_g) - 1);
        let g_second_half = g >> first_half_g;
        let u_first_half = u & ((1 << first_half_uv) - 1);
        let u_second_half = u >> first_half_uv;
        let v_first_half = v & ((1 << first_half_uv) - 1);
        let v_second_half = v >> first_half_uv;

        let ty = self.a_c.circuit[depth].gates[i].ty;

        match ty {
          0 | 1 | 6 | 7 | 8 | 9 | 13 => {
            ret[ty] = ret[ty]
              + (self.beta_g_r0_first_half[g_first_half]
                * self.beta_g_r0_second_half[g_second_half]
                + self.beta_g_r1_first_half[g_first_half]
                  * self.beta_g_r1_second_half[g_second_half])
                * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half])
                * (self.beta_v_first_half[v_first_half] * self.beta_v_second_half[v_second_half]);
          }
          2 => {}
          3 => {}
          4 => {}
          5 => {
            let beta_g_val = self.beta_g_r0_first_half[g_first_half]
              * self.beta_g_r0_second_half[g_second_half]
              + self.beta_g_r1_first_half[g_first_half] * self.beta_g_r1_second_half[g_second_half];
            let beta_v_0 = self.beta_v_first_half[0] * self.beta_v_second_half[0];
            for j in u..v {
              let u_first_half = j & ((1 << first_half_uv) - 1);
              let u_second_half = j >> first_half_uv;
              ret[5] = ret[5]
                + beta_g_val
                  * beta_v_0
                  * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half]);
            }
          }
          12 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;

            let beta_g_val = self.beta_g_r0_first_half[g_first_half]
              * self.beta_g_r0_second_half[g_second_half]
              + self.beta_g_r1_first_half[g_first_half] * self.beta_g_r1_second_half[g_second_half];
            let mut beta_v_0 = self.beta_v_first_half[0] * self.beta_v_second_half[0];
            for j in u..=v {
              let u_first_half = j & ((1 << first_half_uv) - 1);
              let u_second_half = j >> first_half_uv;
              ret[12] = ret[12]
                + beta_g_val
                  * beta_v_0
                  * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half]);
              beta_v_0 = beta_v_0 + beta_v_0;
            }
          }
          14 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;

            let beta_g_val = self.beta_g_r0_first_half[g_first_half]
              * self.beta_g_r0_second_half[g_second_half]
              + self.beta_g_r1_first_half[g_first_half] * self.beta_g_r1_second_half[g_second_half];
            let beta_v_0 = self.beta_v_first_half[0] * self.beta_v_second_half[0];
            for j in 0..self.a_c.circuit[depth].gates[i].parameter_length {
              let src = self.a_c.circuit[depth].gates[i].src[j];
              let u_first_half = src & ((1 << first_half_uv) - 1);
              let u_second_half = src >> first_half_uv;
              let weight = self.a_c.circuit[depth].gates[i].weight[j];
              ret[14] = ret[14]
                + beta_g_val
                  * beta_v_0
                  * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half])
                  * weight;
            }
          }
          10 => {
            if !relay_set {
              tmp_u_val = vec![FE_ZERO; self.a_c.circuit[depth - 1].gates.len()];

              for (i, tmp_item) in tmp_u_val
                .iter_mut()
                .enumerate()
                .take(self.a_c.circuit[depth - 1].gates.len())
              {
                let u_first_half = i & ((1 << first_half_uv) - 1);
                let u_second_half = i >> first_half_uv;
                *tmp_item =
                  self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half];
              }

              relay_set = true;
            }
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            ret[10] = ret[10]
              + (self.beta_g_r0_first_half[g_first_half]
                * self.beta_g_r0_second_half[g_second_half]
                + self.beta_g_r1_first_half[g_first_half]
                  * self.beta_g_r1_second_half[g_second_half])
                * tmp_u_val[u];
          }
          _ => {}
        }
      }
      ret[10] = ret[10] * zero_v;
    }
    ret
      .iter()
      .zip(ret_para.iter())
      .for_each(|(ret_val, ret_para_val)| {
        if self.a_c.circuit[depth].is_parallel {
          assert_eq!(ret_val, ret_para_val);
        }
      });

    ret
  }

  pub fn v_in() {}

  //Never used
  pub fn read_r1cs() {}

  //Never used, original code is all commented in Orion, empty in Virgo
  pub fn self_inner_product_test() {} //Never used, implemented only in Virgo, empty in Orion
}

pub fn generate_randomness(size: usize) -> Vec<FieldElement> {
  (0..size).map(|_| FieldElement::new_random()).collect()
}
