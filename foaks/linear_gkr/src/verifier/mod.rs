use std::{fs, fs::read_to_string, process, time::Instant};
use std::{
  fs::File,
  io::{Error, Write},
  mem,
};

use infrastructure::my_hash::HashDigest;
use infrastructure::{
  constants::{LOG_SLICE_NUMBER, SLICE_NUMBER},
  rs_polynomial::{inverse_fast_fourier_transform, ScratchPad},
};
use poly_commitment::PolyCommitVerifier;
use prime_field::FieldElement;

use crate::{
  circuit_fast_track::{Gate, Layer, LayeredCircuit},
  polynomial::QuadraticPoly,
  prover::ZkProver,
};

#[derive(Default, Debug)]
pub struct VerifierContext {
  pub q_eval_real: Vec<FieldElement>,
  pub q_eval_verifier: Vec<FieldElement>,
  pub q_ratio: Vec<FieldElement>,
}

#[derive(Default, Debug)]
pub struct ZkVerifier {
  //pub prover: zk_prover, // ZY suggestion
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
  // The circuit
  //zk_prover *p; //!< The prover
  vpd_randomness: Vec<FieldElement>,
  one_minus_vpd_randomness: Vec<FieldElement>,
  pub ctx: VerifierContext,
}

impl ZkVerifier {
  pub fn new() -> Self { Default::default() }

  pub fn read_circuit(&mut self, circuit_path: &str, meta_path: &str) -> Option<usize> {
    let d: usize;
    let circuit_content = read_to_string(&circuit_path).unwrap();
    let mut circuit_lines = circuit_content.lines();
    let describe_gate = |circuit: &Vec<Layer>, input_gate: usize, ix: usize, ty, g, u, v| {
      if !(input_gate < (1 << circuit[ix - 1].bit_length)) {
        println!(
          "{} {} {} {} {} ",
          ty,
          g,
          u,
          v,
          (1 << circuit[ix - 1].bit_length)
        );
      }
    };

    if let Some(value) = circuit_lines.next() {
      d = value.parse().unwrap_or_else(|err| {
        eprintln!("Problem parsing total number of layers from file: {}", err);
        process::exit(1)
      });
    } else {
      eprintln!("Empty File!!!");
      process::exit(1);
    }

    self.a_c.circuit = vec![Layer::default(); d + 1];
    self.a_c.total_depth = d + 1;

    let mut max_bit_length: Option<usize> = None;
    let mut n_pad: usize;

    for i in 1..d + 1 {
      let pad_requirement: usize;
      let mut next_line_splited = circuit_lines.next().unwrap().split_whitespace();
      let mut number_gates: usize = next_line_splited.next().unwrap().parse().unwrap();
      if d > 3 {
        pad_requirement = 17;
      } else {
        pad_requirement = 15;
      }
      if i == 1 && number_gates < (1 << pad_requirement) {
        n_pad = 1 << pad_requirement;
      } else {
        n_pad = number_gates;
      }

      if i != 1 {
        if number_gates == 1 {
          self.a_c.circuit[i].gates = vec![Gate::new(); 2];
        } else {
          self.a_c.circuit[i].gates = vec![Gate::new(); n_pad];
        }
      } else {
        if number_gates == 1 {
          self.a_c.circuit[0].gates = vec![Gate::new(); 2];
          self.a_c.circuit[1].gates = vec![Gate::new(); 2];
        } else {
          self.a_c.circuit[0].gates = vec![Gate::new(); n_pad];
          self.a_c.circuit[1].gates = vec![Gate::new(); n_pad];
        }
      }

      let mut max_gate: Option<usize>;
      let mut previous_g: Option<usize> = None;

      for j in 0..number_gates {
        let ty: usize = next_line_splited.next().unwrap().parse().unwrap();
        let g: usize = next_line_splited.next().unwrap().parse().unwrap();
        let u: usize = next_line_splited.next().unwrap().parse().unwrap();
        let mut v: usize = next_line_splited.next().unwrap().parse().unwrap();

        if ty != 3 {
          if ty == 5 {
            assert!(u < (1 << self.a_c.circuit[i - 1].bit_length));
            assert!(v > u && v <= (1 << self.a_c.circuit[i - 1].bit_length));
          } else {
            describe_gate(&self.a_c.circuit, u, i, ty, g, u, v);
            assert!(u < (1 << self.a_c.circuit[i - 1].bit_length));
            describe_gate(&self.a_c.circuit, v, i, ty, g, u, v);
            assert!(v < (1 << self.a_c.circuit[i - 1].bit_length));
          }
        }
        if ty == 6 {
          if v != 0 {
            eprintln!("WARNING, v!=0 for NOT gate")
          }
          v = 0;
        }
        if ty == 10 {
          if v != 0 {
            eprintln!("WARNING, v!=0 for relay gate {}", i)
          }
          v = 0;
        }
        if ty == 13 {
          assert_eq!(u, v);
        }
        if let Some(prev_g) = previous_g {
          if g != prev_g + 1 {
            eprintln!(
              "Error, gates must be in sorted order, and full [0, 2^n - 1]. {} {} {} {}",
              i, j, g, prev_g
            );
            process::exit(1)
          }
        } else if g != 0 {
          {
            eprintln!(
              "Error, gates must be in sorted order, and full [0, 2^n - 1]. {} {} {} -1",
              i, j, g
            );
            process::exit(1)
          }
        }
        previous_g = Some(g);
        if i != 1 {
          self.a_c.circuit[i].gates[g] = Gate::from_params(ty, u, v);
        } else {
          assert!(ty == 2 || ty == 3);
          self.a_c.circuit[1].gates[g] = Gate::from_params(4, g, 0);
          self.a_c.circuit[0].gates[g] = Gate::from_params(ty, u, v);
        }
      }
      if i == 1 {
        for g in number_gates..n_pad {
          self.a_c.circuit[1].gates[g] = Gate::from_params(4, g, 0);
          self.a_c.circuit[0].gates[g] = Gate::from_params(3, 0, 0);
        }
        number_gates = n_pad;
        previous_g = Some(n_pad - 1);
      }

      max_gate = previous_g;
      let mut cnt = 0;
      while max_gate > Some(0) {
        cnt += 1;
        if let Some(val) = max_gate {
          max_gate = Some(val >> 1);
        }
      }
      max_gate = Some(1);
      while cnt > 0 {
        if let Some(val) = max_gate {
          max_gate = Some(val << 1);
        }
        cnt -= 1;
      }
      let mut mx_gate = max_gate;
      while mx_gate > Some(0) {
        cnt += 1;
        if let Some(val) = mx_gate {
          mx_gate = Some(val >> 1);
        }
      }
      if number_gates == 1 {
        //add a dummy gate to avoid ill-defined layer.
        if i != 1 {
          self.a_c.circuit[i].gates[max_gate.unwrap()] = Gate::from_params(2, 0, 0);
          self.a_c.circuit[i].bit_length = cnt;
        } else {
          self.a_c.circuit[0].gates[max_gate.unwrap()] = Gate::from_params(2, 0, 0);
          self.a_c.circuit[0].bit_length = cnt;
          self.a_c.circuit[1].gates[max_gate.unwrap()] = Gate::from_params(4, 1, 0);
          self.a_c.circuit[1].bit_length = cnt;
        }
      } else {
        self.a_c.circuit[i].bit_length = cnt - 1;
        if i == 1 {
          self.a_c.circuit[0].bit_length = cnt - 1;
        }
      }

      println!("layer {}, bit_length {}", i, self.a_c.circuit[i].bit_length);

      if Some(self.a_c.circuit[i].bit_length) > max_bit_length {
        max_bit_length = Some(self.a_c.circuit[i].bit_length);
      }
    }
    self.a_c.circuit[0].is_parallel = false;

    let meta_content = read_to_string(&meta_path).unwrap();
    let mut meta_lines = meta_content.lines();

    for i in 1..=d {
      let mut meta_line_splited = meta_lines.next().unwrap().split_whitespace();

      let is_para: usize = meta_line_splited.next().unwrap().parse().unwrap();
      self.a_c.circuit[i].block_size = meta_line_splited.next().unwrap().parse().unwrap();
      self.a_c.circuit[i].repeat_num = meta_line_splited.next().unwrap().parse().unwrap();
      self.a_c.circuit[i].log_block_size = meta_line_splited.next().unwrap().parse().unwrap();
      self.a_c.circuit[i].log_repeat_num = meta_line_splited.next().unwrap().parse().unwrap();

      if is_para != 0 {
        assert_eq!(
          1 << self.a_c.circuit[i].log_repeat_num,
          self.a_c.circuit[i].repeat_num
        );
        self.a_c.circuit[i].is_parallel = true;
      } else {
        self.a_c.circuit[i].is_parallel = false;
      }
    }

    Self::init_array(self, max_bit_length.unwrap());
    Some(max_bit_length.unwrap())
  }

  pub fn init_array(&mut self, max_bit_length: usize) {
    let first_half_len = max_bit_length / 2;
    let second_half_len = max_bit_length - first_half_len;

    self.beta_g_r0_first_half = vec![FieldElement::zero(); 1 << first_half_len];
    self.beta_g_r0_second_half = vec![FieldElement::zero(); 1 << second_half_len];
    self.beta_g_r1_first_half = vec![FieldElement::zero(); 1 << first_half_len];
    self.beta_g_r1_second_half = vec![FieldElement::zero(); 1 << second_half_len];
    self.beta_v_first_half = vec![FieldElement::zero(); 1 << first_half_len];
    self.beta_v_second_half = vec![FieldElement::zero(); 1 << second_half_len];
    self.beta_u_first_half = vec![FieldElement::zero(); 1 << first_half_len];
    self.beta_u_second_half = vec![FieldElement::zero(); 1 << second_half_len];

    self.beta_g_r0_block_first_half = vec![FieldElement::zero(); 1 << first_half_len];
    self.beta_g_r0_block_second_half = vec![FieldElement::zero(); 1 << second_half_len];
    self.beta_g_r1_block_first_half = vec![FieldElement::zero(); 1 << first_half_len];
    self.beta_g_r1_block_second_half = vec![FieldElement::zero(); 1 << second_half_len];
    self.beta_v_block_first_half = vec![FieldElement::zero(); 1 << first_half_len];
    self.beta_v_block_second_half = vec![FieldElement::zero(); 1 << second_half_len];
    self.beta_u_block_first_half = vec![FieldElement::zero(); 1 << first_half_len];
    self.beta_u_block_second_half = vec![FieldElement::zero(); 1 << second_half_len];
  }

  pub fn verify(
    &mut self,
    output_path: &String,
    bit_length: usize,
    inputs: Vec<FieldElement>,
    n: usize,
    query_count: usize,
    combined_codeword: Vec<FieldElement>,
    q: Vec<usize>,
  ) -> (bool, f64) {
    //println!("output path: {}", output_path);
    // Initialize the prover,
    // the original repo initialize the prover in the main fn()
    let mut zk_prover = ZkProver::new();
    zk_prover.init_array(bit_length, self.a_c.clone());
    zk_prover.get_witness(inputs, n);

    self.proof_size = 0;
    //there is a way to compress binlinear pairing element
    let mut verification_time: f64 = 0.0;
    let mut predicates_calc_time: f64 = 0.0;
    let mut verification_rdl_time: f64 = 0.0;

    //Below function is not implemented neither in virgo repo nor orion repo
    //prime_field::init_random();

    //Below function is not implemented neither in virgo repo nor orion repo
    //self.prover.unwrap().proof_init();

    let result = zk_prover.evaluate();
    let mut alpha = FieldElement::real_one();
    let mut beta = FieldElement::zero();

    //	random_oracle oracle; // Orion just declare the variable but dont use it
    // later
    let capacity = self.a_c.circuit[self.a_c.total_depth - 1].bit_length;

    let mut r_0 = generate_randomness(capacity);
    let mut r_1 = generate_randomness(capacity);

    let mut one_minus_r_0 = Vec::with_capacity(capacity);
    let mut one_minus_r_1 = Vec::with_capacity(capacity);

    for i in 0..capacity {
      one_minus_r_0.push(FieldElement::real_one() - r_0[i]);
      one_minus_r_1.push(FieldElement::real_one() - r_1[i]);
    }
    let t_a = Instant::now();

    println!("Calc V_output(r)");
    let mut a_0 = zk_prover.v_res(
      one_minus_r_0.clone(),
      r_0.clone(),
      result.clone(),
      capacity,
      1 << capacity,
    );

    let time_span = t_a.elapsed();
    println!("    Time:: {}", time_span.as_secs_f64());

    a_0 = alpha * a_0;
    let mut alpha_beta_sum = a_0;
    let mut direct_relay_value: FieldElement;

    for i in (1..=(self.a_c.total_depth - 1)).rev() {
      // never used
      //let rho = FieldElement::new_random();

      zk_prover.sumcheck_init(
        i,
        self.a_c.circuit[i].bit_length,
        self.a_c.circuit[i - 1].bit_length,
        self.a_c.circuit[i - 1].bit_length,
        alpha,
        beta,
        r_0.clone(),
        r_1.clone(),
        one_minus_r_0.clone(),
        one_minus_r_1.clone(),
      );

      zk_prover.sumcheck_phase1_init();

      let mut previous_random = FieldElement::from_real(0);
      let r_u;
      let mut r_v;

      //next level random
      r_u = generate_randomness(self.a_c.circuit[i - 1].bit_length);
      r_v = generate_randomness(self.a_c.circuit[i - 1].bit_length);

      direct_relay_value =
        alpha * self.direct_relay(i, &r_0, &r_u) + beta * self.direct_relay(i, &r_1, &r_u);

      if i == 1 {
        for j in 0..self.a_c.circuit[i - 1].bit_length {
          r_v[j] = FieldElement::zero();
        }
      }

      //V should test the maskR for two points, V does random linear combination of
      // these points first
      // never used
      //let random_combine = generate_randomness(1)[0];

      //Every time all one test to V, V needs to do a linear combination for
      // security.
      //let linear_combine = generate_randomness(1)[0]; // mem leak

      let mut one_minus_r_u = vec![FieldElement::zero(); self.a_c.circuit[i - 1].bit_length];
      let mut one_minus_r_v = vec![FieldElement::zero(); self.a_c.circuit[i - 1].bit_length];

      for j in 0..(self.a_c.circuit[i - 1].bit_length) {
        one_minus_r_u[j] = FieldElement::from_real(1) - r_u[j];
        one_minus_r_v[j] = FieldElement::from_real(1) - r_v[j];
        //one_minus_r_u.push(FieldElement::from_real(1) - r_u[j]);
        //one_minus_r_v.push(FieldElement::from_real(1) - r_v[j]);
      }

      for j in 0..(self.a_c.circuit[i - 1].bit_length) {
        let poly = zk_prover.sumcheck_phase1_update(previous_random, j);
        self.proof_size += mem::size_of::<QuadraticPoly>();
        previous_random = r_u[j];
        //todo: Debug eval() fn
        let eval_zero = poly.eval(&FieldElement::zero());
        let eval_one = poly.eval(&FieldElement::real_one());
        // println!(
        //   "j:{}, i:{}, alpha_beta_sum.real:{}, img{}",
        //   j, i, alpha_beta_sum.real, alpha_beta_sum.img
        // );

        if eval_zero + eval_one != alpha_beta_sum {
          //todo: Improve error handling
          eprintln!(
            "Verification fail, phase1, circuit {}, current bit {}",
            i, j
          );
          return (false, 0.0);
        } else {
          //println!(
          //  "Verification pass, phase1, circuit {}, current bit {}",
          //i, j
          //);
        }
        alpha_beta_sum = poly.eval(&r_u[j].clone());
      }

      zk_prover.sumcheck_phase2_init(previous_random, r_u.clone(), one_minus_r_u.clone());
      let mut previous_random = FieldElement::zero();
      for j in 0..self.a_c.circuit[i - 1].bit_length {
        if i == 1 {
          r_v[j] = FieldElement::zero();
        }
        let poly = zk_prover.sumcheck_phase2_update(previous_random, j);
        self.proof_size += mem::size_of::<QuadraticPoly>();
        //poly.c = poly.c; ???

        previous_random = r_v[j];

        if poly.eval(&FieldElement::zero())
          + poly.eval(&FieldElement::real_one())
          + direct_relay_value * zk_prover.v_u
          != alpha_beta_sum
        {
          //todo: Improve error handling
          eprintln!(
            "Verification fail, phase2, circuit {}, current bit {}",
            i, j
          );
          return (false, 0.0);
        } else {
          //println!(
          //  "Verification fail, phase1, circuit {}, current bit {}",
          //i, j
          //);
        }
        alpha_beta_sum = poly.eval(&r_v[j].clone()) + direct_relay_value * zk_prover.v_u;
      }
      //Add one more round for maskR
      //quadratic_poly poly p->sumcheck_finalroundR(previous_random, C.current[i -
      // 1].bit_length);

      let final_claims = zk_prover.sumcheck_finalize(previous_random);

      let v_u = final_claims.0;
      let v_v = final_claims.1;

      let predicates_calc = Instant::now();
      self.beta_init(
        i,
        alpha,
        beta,
        &r_0,
        &r_1,
        &r_u,
        &r_v,
        &one_minus_r_0,
        &one_minus_r_1,
        &one_minus_r_u,
        &one_minus_r_v,
      );

      let predicates_value = self.predicates(
        i,
        r_0.clone(),
        r_1.clone(),
        r_u.clone(),
        r_v.clone(),
        alpha,
        beta,
      );

      let predicates_calc_span = predicates_calc.elapsed();
      //println!("predicates_calc_span: {:?}", predicates_calc_span);
      if self.a_c.circuit[i].is_parallel == false {
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

      let mut r = Vec::new();
      for j in 0..self.a_c.circuit[i - 1].bit_length {
        r.push(r_u[j].clone());
      }
      for j in 0..self.a_c.circuit[i - 1].bit_length {
        r.push(r_v[j].clone());
      }

      if alpha_beta_sum
        != (add_value * (v_u + v_v)
          + mult_value * v_u * v_v
          + not_value * (FieldElement::real_one() - v_u)
          + minus_value * (v_u - v_v)
          + xor_value * (v_u + v_v - FieldElement::from_real(2) * v_u * v_v)
          + naab_value * (v_v - v_u * v_v)
          + sum_value * v_u
          + custom_comb_value * v_u
          + relay_value * v_u
          + exp_sum_value * v_u
          + bit_test_value * (FieldElement::real_one() - v_v) * v_u)
          + direct_relay_value * v_u
      {
        //Todo: improve error handling
        eprintln!("Verification fail, semi final, circuit level {}", i,);
        return (false, 0.0);
      }

      // let tmp_alpha = generate_randomness(1); There's no need to generate a vector, assign random directly
      // let tmp_beta= generate_randomness(1);

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
    let mut all_sum = vec![FieldElement::zero(); SLICE_NUMBER];
    println!("GKR witness size: {}", 1 << self.a_c.circuit[0].bit_length);
    let merkle_root_l = zk_prover
      .poly_prover
      .commit_private_array(&zk_prover.circuit_value[0], self.a_c.circuit[0].bit_length);
    self.ctx.q_eval_real = vec![FieldElement::zero(); 1 << self.a_c.circuit[0].bit_length];
    self.dfs_for_public_eval(
      0usize,
      FieldElement::real_one(),
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

    let input_0_verify = self.poly_verifier.verify_poly_commitment(
      &all_sum,
      self.a_c.circuit[0].bit_length,
      &public_array,
      &mut verification_time,
      &mut self.proof_size,
      &mut zk_prover.total_time,
      merkle_root_l,
      merkle_root_h,
    );

    zk_prover.poly_prover.total_time_pc_p += self.poly_verifier.pc_prover.total_time_pc_p;

    if !input_0_verify {
      eprintln!("Verification fail, input vpd");
      return (false, 0.0);
    } else {
      println!("Verification pass");
      println!("Prove time {}", zk_prover.total_time);
      println!("Verification rdl time {}", verification_rdl_time);
      println!(
        "Verification time {}",
        verification_time - verification_rdl_time
      );
      self.v_time = verification_time - verification_rdl_time;
      println!("Proof size (bytes) {}", self.proof_size);

      ZkVerifier::write_file(
        output_path,
        zk_prover.total_time,
        verification_time,
        predicates_calc_time,
        verification_rdl_time,
        self.proof_size,
      )
      .expect("Error while writing file");
    }
    // Code added from tensor_product()
    let sample_t0 = Instant::now();
    for i in 0..query_count {
      // i = 717 q[i] = 128 combined_codeword[128] = 0
      assert_eq!(
        zk_prover.circuit_value[zk_prover.a_c.total_depth - 1][i],
        combined_codeword[q[i]],
      );
    }
    let time_span = sample_t0.elapsed().as_secs_f64();

    (true, time_span)
  }

  pub fn write_file(
    output_path: &String,
    total_time: f64,
    verification_time: f64,
    predicates_calc_time: f64,
    verification_rdl_time: f64,
    proof_size: usize,
  ) -> Result<(), Error> {
    //fs::create_dir("/some/dir")?;

    //let mut result_file = File::create(output_path)?;
    let full_path = std::path::Path::new(output_path);
    let prefix = full_path.parent().unwrap();
    fs::create_dir_all(prefix).unwrap();
    let mut result_file = File::create(full_path)?;

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
    self.ctx.q_eval_verifier = vec![FieldElement::zero(); 1 << (log_length - LOG_SLICE_NUMBER)];
    self.ctx.q_ratio = vec![FieldElement::zero(); 1 << LOG_SLICE_NUMBER];

    // TODO check
    Self::dfs_ratio(
      self,
      0,
      FieldElement::real_one(),
      &r[log_length - LOG_SLICE_NUMBER..].to_vec(),
      one_minus_r[log_length - LOG_SLICE_NUMBER..].to_vec(),
      //vec![one_minus_r[log_length - LOG_SLICE_NUMBER]],
      //one_minus_r.clone() + log_length - LOG_SLICE_NUMBER,
      0,
    );
    Self::dfs_coef(
      self,
      0,
      FieldElement::real_one(),
      r,
      one_minus_r.clone(),
      0,
      log_length - LOG_SLICE_NUMBER,
    );
    let mut q_coef_verifier = vec![FieldElement::default(); 1 << (log_length - LOG_SLICE_NUMBER)];

    inverse_fast_fourier_transform(
      scratch_pad,
      &self.ctx.q_eval_verifier,
      1 << (log_length - LOG_SLICE_NUMBER),
      1 << (log_length - LOG_SLICE_NUMBER),
      FieldElement::get_root_of_unity(log_length - LOG_SLICE_NUMBER).unwrap(),
      &mut q_coef_verifier,
    );

    let mut q_coef_arr = vec![FieldElement::default(); 1 << log_length];
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

  //Todo: Debug aritmetic pointes
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
      FieldElement::from_real(0)
    } else {
      let ret = FieldElement::from_real(1);
      let result = ret
        * r_g
          .iter()
          .zip(r_u.iter())
          .map(|(&g, &u)| FieldElement::from_real(1) - g - u + FieldElement::from_real(2) * g * u)
          .fold(FieldElement::from_real(1), |acc, val| acc * val);
      result
    }
  }

  pub fn beta_init(
    &mut self,
    depth: usize,
    alpha: FieldElement,
    beta: FieldElement,
     r_0: &[FieldElement],
    r_1: &[FieldElement],
    r_u: &[FieldElement],
    r_v: &[FieldElement],
    one_minus_r_0: &[FieldElement],
    one_minus_r_1: &[FieldElement],
    one_minus_r_u: &[FieldElement],
    one_minus_r_v: &[FieldElement],
  ) {
    let debug_mode = false;
    if !self.a_c.circuit[depth].is_parallel || debug_mode {
      self.beta_g_r0_first_half[0] = alpha;
      self.beta_g_r1_first_half[0] = beta;
      self.beta_g_r0_second_half[0] = FieldElement::from_real(1);
      self.beta_g_r1_second_half[0] = FieldElement::from_real(1);

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

      self.beta_u_first_half[0] = FieldElement::real_one();
      self.beta_v_first_half[0] = FieldElement::real_one();
      self.beta_u_second_half[0] = FieldElement::real_one();
      self.beta_v_second_half[0] = FieldElement::real_one();
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
      self.beta_g_r0_block_second_half[0] = FieldElement::from_real(1);
      self.beta_g_r1_block_second_half[0] = FieldElement::from_real(1);

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

      self.beta_u_block_first_half[0] = FieldElement::real_one();
      self.beta_v_block_first_half[0] = FieldElement::real_one();
      self.beta_u_block_second_half[0] = FieldElement::real_one();
      self.beta_v_block_second_half[0] = FieldElement::real_one();
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
    depth: usize,
    r_0: Vec<FieldElement>,
    r_1: Vec<FieldElement>,
    r_u: Vec<FieldElement>,
    r_v: Vec<FieldElement>,
    _alpha: FieldElement,
    _beta: FieldElement,
  ) -> Vec<FieldElement> {
    let gate_type_count = 15;
    let mut ret_para = vec![FieldElement::zero(); gate_type_count];
    let mut ret = vec![FieldElement::zero(); gate_type_count];

    for i in 0..gate_type_count {
      ret[i] = FieldElement::zero();
      ret_para[i] = FieldElement::zero();
    }

    if depth == 1 {
      return ret;
    }

    let debug_mode = false;
    if self.a_c.circuit[depth].is_parallel {
      let first_half_g = self.a_c.circuit[depth].log_block_size / 2;
      let first_half_uv = self.a_c.circuit[depth - 1].log_block_size / 2;

      let mut one_block_alpha = vec![FieldElement::zero(); gate_type_count];
      let mut one_block_beta = vec![FieldElement::zero(); gate_type_count];

      for _i in 0..gate_type_count {
        one_block_alpha.push(FieldElement::zero());
        one_block_beta.push(FieldElement::zero());
      }

      assert_eq!(
        (1 << self.a_c.circuit[depth].log_block_size),
        self.a_c.circuit[depth].block_size
      );

      for i in 0..self.a_c.circuit[depth].log_block_size {
        let mut g = i;
        let mut u = self.a_c.circuit[depth].gates[i].u;
        let mut v = self.a_c.circuit[depth].gates[i].v;
        g = g & ((1 << self.a_c.circuit[depth].log_block_size) - 1);
        u = u & ((1 << self.a_c.circuit[depth - 1].log_block_size) - 1);
        v = v & ((1 << self.a_c.circuit[depth - 1].log_block_size) - 1);

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
      let one = FieldElement::real_one();
      for i in 0..self.a_c.circuit[depth].repeat_num {
        let mut prefix_alpha = one;
        let mut prefix_beta = one;
        let mut prefix_alpha_v0 = one;
        let mut prefix_beta_v0 = one;

        for j in 0..self.a_c.circuit[depth].log_repeat_num {
          if (i >> j) > 0 {
            let uv_value = r_u[j + self.a_c.circuit[depth - 1].log_block_size]
              * r_v[j + self.a_c.circuit[depth - 1].log_block_size];
            prefix_alpha =
              prefix_alpha * r_0[j + self.a_c.circuit[depth].log_block_size] * uv_value;
            prefix_beta = prefix_beta * r_1[j + self.a_c.circuit[depth].log_block_size] * uv_value;
          } else {
            let uv_value = (one - r_u[j + self.a_c.circuit[depth - 1].log_block_size])
              * (one - r_v[j + self.a_c.circuit[depth - 1].log_block_size]);
            prefix_alpha =
              prefix_alpha * (one - r_0[j + self.a_c.circuit[depth].log_block_size]) * uv_value;
            prefix_beta =
              prefix_beta * (one - r_1[j + self.a_c.circuit[depth].log_block_size]) * uv_value;
          }
        }
        for j in 0..self.a_c.circuit[depth].log_repeat_num {
          if (i >> j) > 0 {
            let uv_value = r_u[j + self.a_c.circuit[depth - 1].log_block_size]
              * (one - r_v[j + self.a_c.circuit[depth - 1].log_block_size]);
            prefix_alpha_v0 =
              prefix_alpha_v0 * r_0[j + self.a_c.circuit[depth].log_block_size] * uv_value;
            prefix_beta_v0 =
              prefix_beta_v0 * r_1[j + self.a_c.circuit[depth].log_block_size] * uv_value;
          } else {
            let uv_value = (one - r_u[j + self.a_c.circuit[depth - 1].log_block_size])
              * (one - r_v[j + self.a_c.circuit[depth - 1].log_block_size]);
            prefix_alpha_v0 =
              prefix_alpha_v0 * (one - r_0[j + self.a_c.circuit[depth].log_block_size]) * uv_value;
            prefix_beta_v0 =
              prefix_beta_v0 * (one - r_1[j + self.a_c.circuit[depth].log_block_size]) * uv_value;
          }
        }
        for j in 0..gate_type_count {
          if j == 6 || j == 10 || j == 5 || j == 12 {
            ret_para[j] = ret_para[j]
              + prefix_alpha_v0 * one_block_alpha[j]
              + prefix_beta_v0 * one_block_beta[j];
          } else {
            ret_para[j] =
              ret_para[j] + prefix_alpha * one_block_alpha[j] + prefix_beta * one_block_beta[j];
          }
        }
      }
      if !debug_mode {
        ret = ret_para.clone();
      }
    }
    if !self.a_c.circuit[depth].is_parallel || debug_mode {
      let first_half_g = self.a_c.circuit[depth].bit_length / 2;
      let first_half_uv = self.a_c.circuit[depth - 1].bit_length / 2;

      //Todo: Debug tmp_u_val
      let mut tmp_u_val = vec![FieldElement::zero(); 1 << self.a_c.circuit[depth - 1].bit_length];
      let zero_v = self.beta_v_first_half[0] * self.beta_v_second_half[0];
      let mut relay_set = false;
      for i in 0..(1 << self.a_c.circuit[depth].bit_length) {
        let g = i;
        let u = self.a_c.circuit[depth].gates[i].u;
        let v = self.a_c.circuit[depth].gates[i].v;

        let g_first_half = g & ((1 << first_half_g) - 1);
        let g_second_half = g >> first_half_g;
        let u_first_half = u & ((1 << first_half_uv) - 1);
        let u_second_half = u >> first_half_uv;
        let v_first_half = v & ((1 << first_half_uv) - 1);
        let v_second_half = v >> first_half_uv;

        match self.a_c.circuit[depth].gates[i].ty {
          0 => {
            ret[0] = ret[0]
              + (self.beta_g_r0_first_half[g_first_half]
                * self.beta_g_r0_second_half[g_second_half]
                + self.beta_g_r1_first_half[g_first_half]
                  * self.beta_g_r1_second_half[g_second_half])
                * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half])
                * (self.beta_v_first_half[v_first_half] * self.beta_v_second_half[v_second_half]);
          }
          1 => {
            ret[1] = ret[1]
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
          6 => {
            ret[6] = ret[6]
              + (self.beta_g_r0_first_half[g_first_half]
                * self.beta_g_r0_second_half[g_second_half]
                + self.beta_g_r1_first_half[g_first_half]
                  * self.beta_g_r1_second_half[g_second_half])
                * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half])
                * (self.beta_v_first_half[v_first_half] * self.beta_v_second_half[v_second_half]);
          }
          7 => {
            ret[7] = ret[7]
              + (self.beta_g_r0_first_half[g_first_half]
                * self.beta_g_r0_second_half[g_second_half]
                + self.beta_g_r1_first_half[g_first_half]
                  * self.beta_g_r1_second_half[g_second_half])
                * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half])
                * (self.beta_v_first_half[v_first_half] * self.beta_v_second_half[v_second_half]);
          }
          8 => {
            ret[8] = ret[8]
              + (self.beta_g_r0_first_half[g_first_half]
                * self.beta_g_r0_second_half[g_second_half]
                + self.beta_g_r1_first_half[g_first_half]
                  * self.beta_g_r1_second_half[g_second_half])
                * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half])
                * (self.beta_v_first_half[v_first_half] * self.beta_v_second_half[v_second_half]);
          }
          9 => {
            ret[9] = ret[9]
              + (self.beta_g_r0_first_half[g_first_half]
                * self.beta_g_r0_second_half[g_second_half]
                + self.beta_g_r1_first_half[g_first_half]
                  * self.beta_g_r1_second_half[g_second_half])
                * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half])
                * (self.beta_v_first_half[v_first_half] * self.beta_v_second_half[v_second_half]);
          }
          10 => {
            if relay_set == false {
              tmp_u_val = vec![FieldElement::zero(); 1 << self.a_c.circuit[depth - 1].bit_length];

              for i in 0..(1 << self.a_c.circuit[depth - 1].bit_length) {
                let u_first_half = i & ((1 << first_half_uv) - 1);
                let u_second_half = i >> first_half_uv;
                tmp_u_val[i] =
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
          13 => {
            let g_first_half = g & ((1 << first_half_g) - 1);
            let g_second_half = g >> first_half_g;
            let u_first_half = u & ((1 << first_half_uv) - 1);
            let u_second_half = u >> first_half_uv;
            let v_first_half = v & ((1 << first_half_uv) - 1);
            let v_second_half = v >> first_half_uv;
            ret[13] = ret[13]
              + (self.beta_g_r0_first_half[g_first_half]
                * self.beta_g_r0_second_half[g_second_half]
                + self.beta_g_r1_first_half[g_first_half]
                  * self.beta_g_r1_second_half[g_second_half])
                * (self.beta_u_first_half[u_first_half] * self.beta_u_second_half[u_second_half])
                * (self.beta_v_first_half[v_first_half] * self.beta_v_second_half[v_second_half]);
          }
          _ => {}
        }
      }
      ret[10] = ret[10] * zero_v;
    }
    for i in 0..gate_type_count {
      if self.a_c.circuit[depth].is_parallel {
        assert_eq!(ret[i], ret_para[i]);
      }
    }
    ret
  }

  pub fn v_in() {}

  //Never used
  pub fn read_r1cs() {}

  //Never used, original code is all commented in Orion, empty in Virgo
  pub fn self_inner_product_test() {} //Never used, implemented only in Virgo, empty in Orion
}

pub fn generate_randomness(size: usize) -> Vec<FieldElement> {
  let k = size;
  let mut ret = vec![FieldElement::zero(); k];

  for i in 0..k {
    ret[i] = FieldElement::new_random();
  }
  ret
}

pub fn read_vec_fe_file(path: &str) -> Vec<FieldElement> {
  let result_content = read_to_string(path).unwrap();
  let result_lines = result_content.lines();

  let res: Vec<FieldElement> = result_lines
    .into_iter()
    .map(|x| {
      let mut line = x.split_whitespace();
      let real: u64 = line.next().unwrap().parse().unwrap();
      let img: u64 = line.next().unwrap().parse().unwrap();
      FieldElement::new(real, img)
    })
    .collect();
  res
}
