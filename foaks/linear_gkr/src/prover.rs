use global::constants::{FE_REAL_ONE, FE_ZERO, SIZE};
use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;
//use rayon::prelude::*;
use std::{
  mem::swap,
  time::{self},
};

use crate::{
  circuit_fast_track::LayeredCircuit,
  polynomial::{LinearPoly, QuadraticPoly},
};

#[derive(Default, Debug, Clone)]
pub struct ProverContext {
  pub inv_2: FieldElement,
  pub v_mult_add_new: Vec<LinearPoly>,
  pub add_v_array_new: Vec<LinearPoly>,
  pub add_mult_sum_new: Vec<LinearPoly>,
  pub gate_meet: Vec<bool>,
  pub rets_prev: Vec<QuadraticPoly>,
  pub rets_cur: Vec<QuadraticPoly>,
}
#[derive(Default, Debug, Clone)]
pub struct ZkProver {
  pub a_c: LayeredCircuit,
  pub poly_prover: PolyCommitProver,
  /** @name Basic
   * Basic information and variables about the arithmetic circuit */
  //< two random gates v_u and v_v queried by V in each layer    v_u: FieldElement,
  pub v_v: FieldElement,
  pub v_u: FieldElement,
  pub total_uv: usize,
  pub circuit_value: Vec<Vec<FieldElement>>,
  sumcheck_layer_id: usize,
  length_g: usize,
  length_u: usize,
  length_v: usize,

  /** @name Randomness
   * Some randomness or values during the proof phase. */
  alpha: FieldElement,
  beta: FieldElement,

  r_0: Vec<FieldElement>,
  r_1: Vec<FieldElement>,
  one_minus_r_0: Vec<FieldElement>,
  one_minus_r_1: Vec<FieldElement>,

  pub add_v_array: Vec<LinearPoly>,
  pub v_mult_add: Vec<LinearPoly>,
  pub beta_g_r0_fhalf: Vec<FieldElement>,
  beta_g_r0_shalf: Vec<FieldElement>,
  beta_g_r1_fhalf: Vec<FieldElement>,
  beta_g_r1_shalf: Vec<FieldElement>,
  beta_u_fhalf: Vec<FieldElement>,
  beta_u_shalf: Vec<FieldElement>,
  /*beta_u: Vec<FieldElement>,
  beta_v: Vec<FieldElement>,
  beta_g: Vec<FieldElement>,*/ //Variables never used
  pub add_mult_sum: Vec<LinearPoly>,

  pub total_time: f64,

  pub ctx: ProverContext,
}

pub struct SumcheckInitArgs {
  pub sumcheck_layer_id: usize,
  pub length_g: usize,
  pub length_v: usize,
  pub alpha: FieldElement,
  pub beta: FieldElement,
  pub r_0: Vec<FieldElement>,
  pub r_1: Vec<FieldElement>,
  pub one_minus_r_0: Vec<FieldElement>,
  pub one_minus_r_1: Vec<FieldElement>,
}

impl ZkProver {
  pub fn new() -> Self {
    Self {
      circuit_value: vec![vec![]; SIZE],
      ..Default::default()
    }
  }

  pub fn init_array(&mut self, max_bit_length: usize, aritmetic_circuit: LayeredCircuit) {
    let half_length = (max_bit_length >> 1) + 1;

    let poly_size = 1 << max_bit_length;
    let fe_size = 1 << half_length;
    self.ctx.gate_meet = vec![false; 15];
    self.ctx.v_mult_add_new = vec![LinearPoly::zero(); poly_size];
    self.ctx.add_v_array_new = vec![LinearPoly::zero(); poly_size];
    self.ctx.add_mult_sum_new = vec![LinearPoly::zero(); poly_size];
    self.ctx.rets_prev = vec![QuadraticPoly::zero(); poly_size];
    self.ctx.rets_cur = vec![QuadraticPoly::zero(); poly_size];

    self.beta_g_r0_fhalf = vec![FE_ZERO; fe_size];
    self.beta_g_r0_shalf = vec![FE_ZERO; fe_size];
    self.beta_g_r1_fhalf = vec![FE_ZERO; fe_size];
    self.beta_g_r1_shalf = vec![FE_ZERO; fe_size];
    self.beta_u_fhalf = vec![FE_ZERO; fe_size];
    self.beta_u_shalf = vec![FE_ZERO; fe_size];
    self.add_mult_sum = vec![LinearPoly::zero(); poly_size];
    self.v_mult_add = vec![LinearPoly::zero(); poly_size];
    self.add_v_array = vec![LinearPoly::zero(); poly_size];

    // In The original repo init, total_time, and get_circuit() are in the main fn()
    self.total_time = 0.0;
    self.get_circuit(aritmetic_circuit);
  }

  pub fn get_circuit(&mut self, from_verifier: LayeredCircuit) {
    self.a_c = from_verifier;
    self.ctx.inv_2 = FieldElement::from_real(2);
  }

  pub fn v_res(
    &mut self,
    one_minus_r_0: &[FieldElement],
    r_0: &[FieldElement],
    mut output: Vec<FieldElement>,
  ) -> FieldElement {
    let mut output_size = output.len();
    let t0 = time::Instant::now();
    for (i, elem) in r_0.iter().enumerate() {
      for j in 0..(output_size >> 1) {
        output[j] = output[j << 1] * one_minus_r_0[i] + output[j << 1 | 1] * *elem;
      }
      output_size >>= 1;
    }
    let time_span = t0.elapsed();
    self.total_time += time_span.as_secs_f64();

    output[0]
  }

  pub fn evaluate(&mut self) -> Vec<FieldElement> {
    let t0 = time::Instant::now();

    for gate in &self.a_c.circuit[0].gates {
      assert!(gate.ty == 3 || gate.ty == 2);
    }

    assert!(self.a_c.total_depth < 1000000);

    for i in 1..self.a_c.total_depth {
      self.circuit_value[i] = vec![FE_ZERO; self.a_c.circuit[i].gates.len()];

      for (g, gate) in self.a_c.circuit[i].gates.iter().enumerate() {
        let (ty, u, v) = (gate.ty, gate.u, gate.v);
        let (value_u, value_v) = (self.circuit_value[i - 1][u], self.circuit_value[i - 1][v]);

        self.circuit_value[i][g] = match ty {
          0 => value_u + value_v,
          1 => {
            let (u_len, v_len) = (
              self.a_c.circuit[i - 1].gates.len(),
              self.a_c.circuit[i - 1].gates.len(),
            );
            assert!(u < u_len && v < v_len);
            value_u * value_v
          }
          2 => FE_ZERO,
          3 => FieldElement::from_real(u as u64),
          4 | 10 => value_u,
          5 => (u..v)
            .map(|k| self.circuit_value[i - 1][k])
            .fold(FE_ZERO, |acc, val| acc + val),
          6 => FE_REAL_ONE - value_u,
          7 => value_u - value_v,
          8 => value_u + value_v - FieldElement::from_real(2) * value_u * value_v,
          9 => {
            let (u_len, v_len) = (
              self.a_c.circuit[i - 1].gates.len(),
              self.a_c.circuit[i - 1].gates.len(),
            );
            assert!(u < u_len && v < v_len);
            value_v - value_u * value_v
          }
          12 => (u..=v)
            .map(|k| self.circuit_value[i - 1][k] * FieldElement::from_real(1u64 << (k - u)))
            .fold(FE_ZERO, |acc, val| acc + val),
          13 => {
            assert_eq!(u, v);
            assert!(u < self.a_c.circuit[i - 1].gates.len());
            value_u * (FE_REAL_ONE - value_v)
          }
          14 => (0..gate.parameter_length)
            .map(|k| self.circuit_value[i - 1][gate.src[k]] * gate.weight[k])
            .fold(FE_ZERO, |acc, val| acc + val),
          _ => panic!("gate type not supported"),
        };
      }
    }

    let time_span = t0.elapsed();

    println!(
      "total evaluation time: {:.6} seconds",
      time_span.as_secs_f64()
    );
    self.circuit_value[self.a_c.total_depth - 1].clone()
  }

  pub fn get_witness(&mut self, inputs: Vec<FieldElement>) {
    //assert_eq!(inputs.len(), self.a_c.circuit[0].gates.len()); // since assert is true, we can refactor this function
    self.circuit_value[0] = inputs;
  }

  pub fn sumcheck_init(&mut self, zkprover: SumcheckInitArgs) {
    self.r_0 = zkprover.r_0;
    self.r_1 = zkprover.r_1;
    self.alpha = zkprover.alpha;
    self.beta = zkprover.beta;
    self.sumcheck_layer_id = zkprover.sumcheck_layer_id;
    self.length_g = zkprover.length_g;
    self.length_u = zkprover.length_v; // Since they are equal, we can refactor this function
    self.length_v = zkprover.length_v;
    self.one_minus_r_0 = zkprover.one_minus_r_0;
    self.one_minus_r_1 = zkprover.one_minus_r_1;
  }

  pub fn sumcheck_phase1_init(&mut self) {
    let t0 = time::Instant::now();
    self.total_uv = self.a_c.circuit[self.sumcheck_layer_id - 1].gates.len();

    for i in 0..self.total_uv {
      self.v_mult_add[i] =
        LinearPoly::new_single_input(self.circuit_value[self.sumcheck_layer_id - 1][i]);
      self.add_v_array[i].a = FE_ZERO;
      self.add_v_array[i].b = FE_ZERO;
      self.add_mult_sum[i].a = FE_ZERO;
      self.add_mult_sum[i].b = FE_ZERO;
    }

    self.beta_g_r0_fhalf[0] = self.alpha;
    self.beta_g_r1_fhalf[0] = self.beta;
    self.beta_g_r0_shalf[0] = FE_REAL_ONE;
    self.beta_g_r1_shalf[0] = FE_REAL_ONE;

    let first_half = self.length_g >> 1;
    let second_half = self.length_g - first_half;

    for i in 0..first_half {
      for j in 0..1 << i {
        self.beta_g_r0_fhalf[j | (1 << i)] = self.beta_g_r0_fhalf[j] * self.r_0[i];
        self.beta_g_r0_fhalf[j] = self.beta_g_r0_fhalf[j] * self.one_minus_r_0[i];
        self.beta_g_r1_fhalf[j | (1 << i)] = self.beta_g_r1_fhalf[j] * self.r_1[i];
        self.beta_g_r1_fhalf[j] = self.beta_g_r1_fhalf[j] * self.one_minus_r_1[i];
      }
    }

    for i in 0..second_half {
      for j in 0..1 << i {
        self.beta_g_r0_shalf[j | (1 << i)] = self.beta_g_r0_shalf[j] * self.r_0[i + first_half];
        self.beta_g_r0_shalf[j] = self.beta_g_r0_shalf[j] * self.one_minus_r_0[i + first_half];
        self.beta_g_r1_shalf[j | (1 << i)] = self.beta_g_r1_shalf[j] * self.r_1[i + first_half];
        self.beta_g_r1_shalf[j] = self.beta_g_r1_shalf[j] * self.one_minus_r_1[i + first_half];
      }
    }

    let mask_fhalf = (1 << first_half) - 1;

    let mut intermediates0 = vec![FE_ZERO; 1 << self.length_g];
    let mut intermediates1 = vec![FE_ZERO; 1 << self.length_g];

    //todo
    //	#pragma omp parallel for

    for i in 0..(1 << self.length_g) {
      let u = self.a_c.circuit[self.sumcheck_layer_id].gates[i].u;
      let v = self.a_c.circuit[self.sumcheck_layer_id].gates[i].v;
      let ty = self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty;
      match ty {
        0 => {
          //add gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          intermediates0[i] = self.circuit_value[self.sumcheck_layer_id - 1][v] * tmp;
          intermediates1[i] = tmp;
        }
        2 => {}
        1 => {
          //mult gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          intermediates0[i] = self.circuit_value[self.sumcheck_layer_id - 1][v] * tmp;
        }
        5 => {
          //sum gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          intermediates1[i] = tmp;
        }
        12 => {
          //exp sum gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          intermediates1[i] = tmp;
        }
        4 => {
          //direct relay gate
          let tmp = self.beta_g_r0_fhalf[u & mask_fhalf] * self.beta_g_r0_shalf[u >> first_half]
            + self.beta_g_r1_fhalf[u & mask_fhalf] * self.beta_g_r1_shalf[u >> first_half];
          intermediates1[i] = tmp;
        }
        6 => {
          //NOT gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          intermediates1[i] = tmp;
        }
        7 => {
          //minus gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          intermediates0[i] = self.circuit_value[self.sumcheck_layer_id - 1][v] * tmp;
          intermediates1[i] = tmp;
        }
        8 => {
          //XOR gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          let tmp_v = tmp * self.circuit_value[self.sumcheck_layer_id - 1][v];
          intermediates0[i] = tmp_v;
          intermediates1[i] = tmp;
        }
        13 => {
          //bit-test gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          let tmp_v = tmp * self.circuit_value[self.sumcheck_layer_id - 1][v];
          intermediates0[i] = tmp_v;
          intermediates1[i] = tmp;
        }
        9 => {
          //NAAB gate
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          let tmp_v = tmp * self.circuit_value[self.sumcheck_layer_id - 1][v];
          intermediates1[i] = tmp_v;
        }
        10 => {
          //relay gate
          let a = self.beta_g_r0_fhalf[i & mask_fhalf];
          let b = self.beta_g_r0_shalf[i >> first_half];
          let c = self.beta_g_r1_fhalf[i & mask_fhalf];
          let d = self.beta_g_r1_shalf[i >> first_half];
          let tmp = a * b + c * d;
          intermediates0[i] = tmp;
        }
        14 => {
          //custom comb
          let tmp = self.beta_g_r0_fhalf[i & mask_fhalf] * self.beta_g_r0_shalf[i >> first_half]
            + self.beta_g_r1_fhalf[i & mask_fhalf] * self.beta_g_r1_shalf[i >> first_half];
          intermediates1[i] = tmp;
        }
        _ => {
          eprintln!(
            "Warning Unknown gate {}",
            self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty
          )
        }
      }
    }

    for i in 0..1 << self.length_g {
      let u = self.a_c.circuit[self.sumcheck_layer_id].gates[i].u;
      let v = self.a_c.circuit[self.sumcheck_layer_id].gates[i].v;

      match self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty {
        0 => {
          //add gate
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_v_array[u].b = self.add_v_array[u].b + intermediates0[i];
          self.add_mult_sum[u].b = self.add_mult_sum[u].b + intermediates1[i];
        }
        2 => {}
        1 => {
          //mult gate
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_mult_sum[u].b = self.add_mult_sum[u].b + intermediates0[i];
        }
        5 => {
          //sum gate
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          for j in u..v {
            self.add_mult_sum[j].b = self.add_mult_sum[j].b + intermediates1[i];
          }
        }

        12 =>
        //exp sum gate
        {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          let mut tmp = intermediates1[i];
          for j in u..v {
            self.add_mult_sum[j].b = self.add_mult_sum[j].b + tmp;
            tmp = tmp + tmp;
          }
        }
        14 => {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          let tmp = intermediates1[i];
          for j in 0..self.a_c.circuit[self.sumcheck_layer_id].gates[i].parameter_length {
            let src = self.a_c.circuit[self.sumcheck_layer_id].gates[i].src[j];
            let weight = self.a_c.circuit[self.sumcheck_layer_id].gates[i].weight[j];
            self.add_mult_sum[src].b = self.add_mult_sum[src].b + weight * tmp;
          }
        }
        4 =>
        //direct relay gate
        {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_mult_sum[u].b = self.add_mult_sum[u].b + intermediates1[i];
        }
        6 =>
        //NOT gate
        {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_mult_sum[u].b = self.add_mult_sum[u].b - intermediates1[i];
          self.add_v_array[u].b = self.add_v_array[u].b + intermediates1[i];
        }
        7 =>
        //minus gate
        {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_v_array[u].b = self.add_v_array[u].b - (intermediates0[i]);
          self.add_mult_sum[u].b = self.add_mult_sum[u].b + intermediates1[i];
        }
        8 =>
        //XOR gate
        {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_v_array[u].b = self.add_v_array[u].b + intermediates0[i];
          self.add_mult_sum[u].b =
            self.add_mult_sum[u].b + intermediates1[i] - intermediates0[i] - intermediates0[i];
        }
        13 =>
        //bit-test gate
        {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_mult_sum[u].b = self.add_mult_sum[u].b - intermediates0[i] + intermediates1[i];
        }
        9 =>
        //NAAB gate
        {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_v_array[u].b = self.add_v_array[u].b + intermediates1[i];
          self.add_mult_sum[u].b = self.add_mult_sum[u].b - intermediates1[i];
        }
        10 =>
        //relay gate
        {
          if !self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] {
            self.ctx.gate_meet[self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty] = true;
          }
          self.add_mult_sum[u].b = self.add_mult_sum[u].b + intermediates0[i];
        }
        _ => println!(
          "Warning Unknown gate {}",
          self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty
        ),
      }
    }
    let time_span = t0.elapsed();
    self.total_time += time_span.as_secs_f64();
  }

  pub fn sumcheck_phase1_update(
    &mut self,
    previous_random: FieldElement,
    current_bit: usize,
  ) -> QuadraticPoly {
    let t0 = time::Instant::now();

    for i in 0..self.total_uv >> 1 {
      let g_zero = i << 1;
      let g_one = i << 1 | 1;
      if current_bit == 0 {
        self.ctx.v_mult_add_new[i].b = self.v_mult_add[g_zero].b;
        self.ctx.v_mult_add_new[i].a = self.v_mult_add[g_one].b - self.ctx.v_mult_add_new[i].b;

        self.ctx.add_v_array_new[i].b = self.add_v_array[g_zero].b;
        self.ctx.add_v_array_new[i].a = self.add_v_array[g_one].b - self.ctx.add_v_array_new[i].b;

        self.ctx.add_mult_sum_new[i].b = self.add_mult_sum[g_zero].b;
        self.ctx.add_mult_sum_new[i].a =
          self.add_mult_sum[g_one].b - self.ctx.add_mult_sum_new[i].b;
      } else {
        self.ctx.v_mult_add_new[i].b =
          self.v_mult_add[g_zero].a * previous_random + self.v_mult_add[g_zero].b;
        self.ctx.v_mult_add_new[i].a = self.v_mult_add[g_one].a * previous_random
          + self.v_mult_add[g_one].b
          - self.ctx.v_mult_add_new[i].b;

        self.ctx.add_v_array_new[i].b =
          self.add_v_array[g_zero].a * previous_random + self.add_v_array[g_zero].b;
        self.ctx.add_v_array_new[i].a = self.add_v_array[g_one].a * previous_random
          + self.add_v_array[g_one].b
          - self.ctx.add_v_array_new[i].b;

        self.ctx.add_mult_sum_new[i].b =
          self.add_mult_sum[g_zero].a * previous_random + self.add_mult_sum[g_zero].b;
        self.ctx.add_mult_sum_new[i].a = self.add_mult_sum[g_one].a * previous_random
          + self.add_mult_sum[g_one].b
          - self.ctx.add_mult_sum_new[i].b;
      }
    }

    swap(&mut self.v_mult_add, &mut self.ctx.v_mult_add_new);
    swap(&mut self.add_v_array, &mut self.ctx.add_v_array_new);
    swap(&mut self.add_mult_sum, &mut self.ctx.add_mult_sum_new);

    //todo
    //#pragma omp parallel for
    for i in 0..(self.total_uv >> 1) {
      self.ctx.rets_prev[i].a = self.add_mult_sum[i].a * self.v_mult_add[i].a;
      self.ctx.rets_prev[i].b = self.add_mult_sum[i].a * self.v_mult_add[i].b
        + self.add_mult_sum[i].b * self.v_mult_add[i].a
        + self.add_v_array[i].a;
      self.ctx.rets_prev[i].c =
        self.add_mult_sum[i].b * self.v_mult_add[i].b + self.add_v_array[i].b;
    }

    let tot = self.total_uv >> 1;
    let mut iter = 1;
    while (1 << iter) <= (self.total_uv >> 1) {
      //todo
      //#pragma omp parallel for
      for j in 0..(tot >> iter) {
        let rets_prev_idx = j << 1;
        self.ctx.rets_cur[j] =
          self.ctx.rets_prev[rets_prev_idx] + self.ctx.rets_prev[rets_prev_idx + 1];
      }
      std::mem::swap(&mut self.ctx.rets_prev, &mut self.ctx.rets_cur);
      iter += 1;
    }
    let ret = self.ctx.rets_prev[0];

    self.total_uv >>= 1;

    let time_span = t0.elapsed();
    self.total_time += time_span.as_secs_f64();

    ret
  }

  pub fn sumcheck_phase2_init(&mut self, r_u: &[FieldElement], one_minus_r_u: &[FieldElement]) {
    let t0 = time::Instant::now();

    let first_half = self.length_u >> 1;
    let second_half = self.length_u - first_half;

    self.beta_u_fhalf[0] = FE_REAL_ONE;
    self.beta_u_shalf[0] = FE_REAL_ONE;

    for i in 0..first_half {
      for j in 0..(1 << i) {
        self.beta_u_fhalf[j | (1 << i)] = self.beta_u_fhalf[j] * r_u[i];
        self.beta_u_fhalf[j] = self.beta_u_fhalf[j] * one_minus_r_u[i];
      }
    }

    for i in 0..second_half {
      for j in 0..(1 << i) {
        self.beta_u_shalf[j | (1 << i)] = self.beta_u_shalf[j] * r_u[i + first_half];
        self.beta_u_shalf[j] = self.beta_u_shalf[j] * one_minus_r_u[i + first_half];
      }
    }

    let mask_fhalf = (1 << first_half) - 1;
    let first_g_half = self.length_g >> 1;
    let mask_g_fhalf = (1 << (self.length_g >> 1)) - 1;

    self.total_uv = self.a_c.circuit[self.sumcheck_layer_id - 1].gates.len();
    let total_g = self.a_c.circuit[self.sumcheck_layer_id].gates.len();

    for i in 0..self.total_uv {
      self.add_mult_sum[i].a = FE_ZERO;
      self.add_mult_sum[i].b = FE_ZERO;
      self.add_v_array[i].a = FE_ZERO;
      self.add_v_array[i].b = FE_ZERO;

      self.v_mult_add[i] =
        LinearPoly::new_single_input(self.circuit_value[self.sumcheck_layer_id - 1][i]);
    }

    let mut intermediates0 = vec![FE_ZERO; total_g];
    let mut intermediates1 = vec![FE_ZERO; total_g];

    //todo
    //#pragma omp parallel for
    for i in 0..total_g {
      let ty = self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty;
      let u = self.a_c.circuit[self.sumcheck_layer_id].gates[i].u;

      let tmp_u = self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
      let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf] * self.beta_g_r0_shalf[i >> first_g_half]
        + self.beta_g_r1_fhalf[i & mask_g_fhalf] * self.beta_g_r1_shalf[i >> first_g_half];

      match ty {
        0 => {
          // add gate
          intermediates0[i] = tmp_g * tmp_u;
          intermediates1[i] = intermediates0[i] * self.v_u;
        }
        1 => {
          // mult gate
          intermediates0[i] = tmp_g * tmp_u * self.v_u;
        }
        5 | 12 | 14 => {
          // sum gate, exp sum gate, custom comb gate
          intermediates0[i] = tmp_g * self.v_u;
        }
        6 => {
          // not gate
          intermediates0[i] = tmp_g * tmp_u - tmp_g * tmp_u * self.v_u;
        }
        7 => {
          // minus gate
          intermediates0[i] = tmp_g * tmp_u;
          intermediates1[i] = intermediates0[i] * self.v_u;
        }
        8 => {
          // xor gate
          let tmp_v_u = tmp_g * tmp_u * self.v_u;
          intermediates0[i] = tmp_g * tmp_u - tmp_v_u - tmp_v_u;
          intermediates1[i] = tmp_v_u;
        }
        9 => {
          // NAAB gate
          intermediates0[i] = tmp_g * tmp_u - self.v_u * tmp_u;
        }
        10 => {
          // relay gate
          intermediates0[i] = tmp_g * tmp_u * self.v_u;
        }
        2 | 4 => { /* No operation required */ }
        _ => {
          println!("Warning Unknown gate {}", ty);
        }
      }
    }

    for i in 0..total_g {
      let ty = self.a_c.circuit[self.sumcheck_layer_id].gates[i].ty;
      let u = self.a_c.circuit[self.sumcheck_layer_id].gates[i].u;
      let v = self.a_c.circuit[self.sumcheck_layer_id].gates[i].v;
      match ty {
        1 =>
        //mult gate
        {
          self.add_mult_sum[v].b = self.add_mult_sum[v].b + intermediates0[i];
        }
        0 =>
        //add gate
        {
          self.add_mult_sum[v].b = self.add_mult_sum[v].b + intermediates0[i];
          self.add_v_array[v].b = intermediates1[i] + self.add_v_array[v].b;
        }
        2 => {}
        4 => {}
        5 =>
        //sum gate
        {
          for j in u..v {
            let tmp_u = self.beta_u_fhalf[j & mask_fhalf] * self.beta_u_shalf[j >> first_half];
            self.add_v_array[0].b = self.add_v_array[0].b + intermediates0[i] * tmp_u;
          }
        }
        12 =>
        //exp sum gate
        {
          let mut tmp_g_vu = intermediates0[i];

          for j in u..v {
            let tmp_u = self.beta_u_fhalf[j & mask_fhalf] * self.beta_u_shalf[j >> first_half];
            self.add_v_array[0].b = self.add_v_array[0].b + tmp_g_vu * tmp_u;
            tmp_g_vu = tmp_g_vu + tmp_g_vu;
          }
        }
        14 =>
        //custom comb gate
        {
          let tmp_g_vu = intermediates0[i];

          for j in 0..self.a_c.circuit[self.sumcheck_layer_id].gates[i].parameter_length {
            let src = self.a_c.circuit[self.sumcheck_layer_id].gates[i].src[j];
            let tmp_u = self.beta_u_fhalf[src & mask_fhalf] * self.beta_u_shalf[src >> first_half];
            let weight = self.a_c.circuit[self.sumcheck_layer_id].gates[i].weight[j];
            self.add_v_array[0].b = self.add_v_array[0].b + tmp_g_vu * tmp_u * weight;
          }
        }
        6 =>
        //not gate
        {
          self.add_v_array[v].b = self.add_v_array[v].b + intermediates0[i];
        }
        7 =>
        //minus gate
        {
          self.add_mult_sum[v].b = self.add_mult_sum[v].b - intermediates0[i];
          self.add_v_array[v].b = intermediates1[i] + self.add_v_array[v].b;
        }
        8 =>
        //xor gate
        {
          self.add_mult_sum[v].b = self.add_mult_sum[v].b + intermediates0[i];
          self.add_v_array[v].b = self.add_v_array[v].b + intermediates1[i];
        }
        13 =>
        //bit-test gate
        {
          self.add_mult_sum[v].b = self.add_mult_sum[v].b - intermediates0[i];
          self.add_v_array[v].b = self.add_v_array[v].b + intermediates0[i];
        }
        9 =>
        //NAAB gate
        {
          self.add_mult_sum[v].b = self.add_mult_sum[v].b + intermediates0[i];
        }
        10 =>
        //relay gate
        {
          self.add_v_array[v].b = self.add_v_array[v].b + intermediates0[i];
        }
        _ => {
          println!("Warning Unknown gate {}", ty);
        }
      }
    }
    let time_span = t0.elapsed();
    self.total_time += time_span.as_secs_f64();
  }

  pub fn sumcheck_phase2_update(
    &mut self,
    previous_random: FieldElement,
    current_bit: usize,
  ) -> QuadraticPoly {
    let t0 = time::Instant::now();
    let mut ret = QuadraticPoly::zero();

    //todo
    //#pragma omp parallel for

    for i in 0..(self.total_uv >> 1) {
      let g_zero = i << 1;
      let g_one = i << 1 | 1;

      if current_bit == 0 {
        self.ctx.v_mult_add_new[i].b = self.v_mult_add[g_zero].b;
        self.ctx.v_mult_add_new[i].a = self.v_mult_add[g_one].b - self.ctx.v_mult_add_new[i].b;

        self.ctx.add_v_array_new[i].b = self.add_v_array[g_zero].b;
        self.ctx.add_v_array_new[i].a = self.add_v_array[g_one].b - self.ctx.add_v_array_new[i].b;

        self.ctx.add_mult_sum_new[i].b = self.add_mult_sum[g_zero].b;
        self.ctx.add_mult_sum_new[i].a =
          self.add_mult_sum[g_one].b - self.ctx.add_mult_sum_new[i].b;
      } else {
        self.ctx.v_mult_add_new[i].b =
          self.v_mult_add[g_zero].a * previous_random + self.v_mult_add[g_zero].b;
        self.ctx.v_mult_add_new[i].a = self.v_mult_add[g_one].a * previous_random
          + self.v_mult_add[g_one].b
          - self.ctx.v_mult_add_new[i].b;

        self.ctx.add_v_array_new[i].b =
          self.add_v_array[g_zero].a * previous_random + self.add_v_array[g_zero].b;
        self.ctx.add_v_array_new[i].a = self.add_v_array[g_one].a * previous_random
          + self.add_v_array[g_one].b
          - self.ctx.add_v_array_new[i].b;

        self.ctx.add_mult_sum_new[i].b =
          self.add_mult_sum[g_zero].a * previous_random + self.add_mult_sum[g_zero].b;
        self.ctx.add_mult_sum_new[i].a = self.add_mult_sum[g_one].a * previous_random
          + self.add_mult_sum[g_one].b
          - self.ctx.add_mult_sum_new[i].b;
      }
      ret.a = ret.a + self.add_mult_sum[i].a * self.v_mult_add[i].a;
      ret.b = ret.b
        + self.add_mult_sum[i].a * self.v_mult_add[i].b
        + self.add_mult_sum[i].b * self.v_mult_add[i].a
        + self.add_v_array[i].a;
      ret.c = ret.c + self.add_mult_sum[i].b * self.v_mult_add[i].b + self.add_v_array[i].b;
    }
    swap(&mut self.v_mult_add, &mut self.ctx.v_mult_add_new);
    swap(&mut self.add_v_array, &mut self.ctx.add_v_array_new);
    swap(&mut self.add_mult_sum, &mut self.ctx.add_mult_sum_new);

    //parallel addition tree
    //todo
    //#pragma omp parallel for
    for i in 0..(self.total_uv >> 1) {
      self.ctx.rets_prev[i].a = self.add_mult_sum[i].a * self.v_mult_add[i].a;
      self.ctx.rets_prev[i].b = self.add_mult_sum[i].a * self.v_mult_add[i].b
        + self.add_mult_sum[i].b * self.v_mult_add[i].a
        + self.add_v_array[i].a;
      self.ctx.rets_prev[i].c =
        self.add_mult_sum[i].b * self.v_mult_add[i].b + self.add_v_array[i].b;
    }

    let tot = self.total_uv >> 1;
    let mut iter = 1;
    while (1 << iter) <= (self.total_uv >> 1) {
      //todo
      //#pragma omp parallel for
      for j in 0..(tot >> iter) {
        self.ctx.rets_cur[j] = self.ctx.rets_prev[j * 2] + self.ctx.rets_prev[j * 2 + 1];
      }
      //todo
      //#pragma omp barrier
      swap(&mut self.ctx.rets_prev, &mut self.ctx.rets_cur);
      iter += 1;
    }
    ret = self.ctx.rets_prev[0];

    self.total_uv >>= 1;

    let time_span = t0.elapsed();
    self.total_time += time_span.as_secs_f64();

    ret
  }

  pub fn sumcheck_finalize(
    &mut self,
    previous_random: FieldElement,
  ) -> (FieldElement, FieldElement) {
    self.v_v = self.v_mult_add[0].eval(previous_random);
    (self.v_u, self.v_v)
  }
}
