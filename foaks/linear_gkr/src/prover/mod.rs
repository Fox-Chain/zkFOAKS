use crate::circuit_fast_track::LayeredCircuit;
use crate::polynomial::{LinearPoly, QuadraticPoly};

use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;

use std::mem::swap;
use std::time::SystemTime;

static mut INV_2: FieldElement = FieldElement::zero();
static mut v_mult_add_new: Vec<LinearPoly> = Vec::new();
static mut add_v_array_new: Vec<LinearPoly> = Vec::new();
static mut add_mult_sum_new: Vec<LinearPoly> = Vec::new();
static mut gate_meet: [bool; 15] = [false; 15];
static mut rets_prev: Vec<QuadraticPoly> = Vec::new();
static mut rets_cur: Vec<QuadraticPoly> = Vec::new();

pub fn from_string(s: &str) -> FieldElement {
    let mut ret = FieldElement::from_real(0);

    for byte in s.bytes() {
        let digit = byte - b'0';
        ret = ret * FieldElement::from_real(10) + FieldElement::from_real(digit.into());
    }

    ret
}

#[derive(Default, Debug)]
pub struct zk_prover {
    //poly_prover: PolyCommitProver,
    /** @name Basic
    	* Basic information and variables about the arithmetic circuit*/
    //< two random gates v_u and v_v queried by V in each layer    v_u: FieldElement,
    v_v: FieldElement,
    v_u: FieldElement,
    pub total_uv: usize,
    pub aritmetic_circuit: Option<*mut LayeredCircuit>, //	c++ code: layered_circuit *C;
    pub circuit_value: Vec<Vec<FieldElement>>,
    sumcheck_layer_id: usize,
    length_g: usize,
    length_u: usize,
    length_v: usize,

    /** @name Randomness
    	* Some randomness or values during the proof phase. */
    alpha: FieldElement,
    beta: FieldElement,

    //< c++ code: const prime_field::field_element *r_0, *r_1; How to deal with "const"
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
    beta_u: Vec<FieldElement>,
    beta_v: Vec<FieldElement>,
    beta_g: Vec<FieldElement>,
    pub add_mult_sum: Vec<LinearPoly>,

    pub total_time: u64,
}

impl zk_prover {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn new2() -> Self {
        Self {
            circuit_value: Vec::with_capacity(1000000),
            ..Default::default()
        }
    }

    pub fn init_array(&mut self, max_bit_length: usize) {
        let half_length = (max_bit_length >> 1) + 1;

        unsafe {
            //gate_meet size: 15 or 14
            gate_meet = [false; 15];
            v_mult_add_new = vec![LinearPoly::zero(); 1 << half_length];
            add_v_array_new = vec![LinearPoly::zero(); 1 << half_length];
            add_mult_sum_new = vec![LinearPoly::zero(); 1 << half_length];
            rets_prev = vec![QuadraticPoly::zero(); 1 << half_length];
            rets_cur = vec![QuadraticPoly::zero(); 1 << half_length];
        }

        Self::init_zkprover(self, half_length);
    }

    pub fn init_zkprover(&mut self, half_length: usize) {
        self.beta_g_r0_fhalf = vec![FieldElement::zero(); 1 << half_length];
        self.beta_g_r0_shalf = vec![FieldElement::zero(); 1 << half_length];
        self.beta_g_r1_fhalf = vec![FieldElement::zero(); 1 << half_length];
        self.beta_g_r1_shalf = vec![FieldElement::zero(); 1 << half_length];
        self.beta_u_fhalf = vec![FieldElement::zero(); 1 << half_length];
        self.beta_u_shalf = vec![FieldElement::zero(); 1 << half_length];
        self.add_mult_sum = vec![LinearPoly::zero(); 1 << half_length];
        self.v_mult_add = vec![LinearPoly::zero(); 1 << half_length];
        self.add_v_array = vec![LinearPoly::zero(); 1 << half_length];
        println!("Init!");
    }

    pub fn get_circuit(&mut self, from_verifier: *mut LayeredCircuit) {
        self.aritmetic_circuit = Some(from_verifier);
        unsafe {
            INV_2 = FieldElement::from_real(2);
        }
    }

    // this function calculate time ?
    // 	prime_field::field_element a_0 = p -> V_res(one_minus_r_0, r_0, result, C.circuit[C.total_depth - 1].bit_length, (1 << (C.circuit[C.total_depth - 1].bit_length)));

    pub fn V_res(
        &mut self,
        one_minus_r_0: Vec<FieldElement>,
        r_0: Vec<FieldElement>,
        output_raw: Vec<FieldElement>,
        r_0_size: usize,
        output_size: usize,
    ) -> FieldElement {
        let t0 = SystemTime::now();
        let mut outputsize = output_size;
        let mut output = vec![FieldElement::zero(); outputsize];
        for i in 0..output_size {
            output.push(output_raw[i]);
        }
        for i in 0..r_0_size {
            for j in 0..(outputsize >> 1) {
                output[j] = (output[j << 1] * one_minus_r_0[i] + output[j << 1 | 1] * r_0[i]);
            }
            outputsize >>= 1;
        }
        let t1 = SystemTime::now();
        let time_span = (t1.duration_since(t0)).unwrap();
        self.total_time += time_span.as_secs();
        let res = output[0];
        res
    }

    pub unsafe fn evaluate(&mut self) -> Vec<FieldElement> {
        //let mut depth: usize;
        //unsafe {
        let t0 = SystemTime::now();
        self.circuit_value.push(vec![
            FieldElement::zero();
            1 << (*self.aritmetic_circuit.unwrap()).circuit[0]
                .bit_length
        ]);
        let halt = 1 << (*self.aritmetic_circuit.unwrap()).circuit[0].bit_length;
        for i in 0..halt {
            let g = i;
            //todo: Could delete below variable, never used
            let u = (*self.aritmetic_circuit.unwrap()).circuit[0].gates[g].u;
            let ty = (*self.aritmetic_circuit.unwrap()).circuit[0].gates[g].ty;
            assert!(ty == 3 || ty == 2);
        }
        assert!((*self.aritmetic_circuit.unwrap()).total_depth < 1000000);
        let depth = (*self.aritmetic_circuit.unwrap()).total_depth;

        for i in 1..depth {
            println!("len: {}", self.circuit_value.len());
            println!("cap: {}", self.circuit_value.capacity());
            self.circuit_value.push(vec![
                FieldElement::zero();
                1 << (*self.aritmetic_circuit.unwrap()).circuit[i]
                    .bit_length
            ]);
            for j in 0..(*self.aritmetic_circuit.unwrap()).circuit[i].bit_length {
                let g = j;
                let ty: usize = (*self.aritmetic_circuit.unwrap()).circuit[i].gates[g].ty;
                let u = (*self.aritmetic_circuit.unwrap()).circuit[i].gates[g].u;
                let v = (*self.aritmetic_circuit.unwrap()).circuit[i].gates[g].v;

                if ty == 0 {
                    self.circuit_value[i][g] =
                        self.circuit_value[i - 1][u] + self.circuit_value[i - 1][v];
                } else if ty == 1 {
                    assert!(
                        u >= 0
                            && u < (1
                                << (*self.aritmetic_circuit.unwrap()).circuit[i - 1].bit_length),
                    );
                    assert!(
                        v >= 0
                            && v < (1
                                << (*self.aritmetic_circuit.unwrap()).circuit[i - 1].bit_length),
                    );
                    self.circuit_value[i][g] =
                        self.circuit_value[i - 1][u] * self.circuit_value[i - 1][v];
                } else if ty == 2 {
                    self.circuit_value[i][g] = FieldElement::from_real(0);
                } else if ty == 3 {
                    // It is suppose to be input gate, it just read the 'u' input, what about 'v' input
                    self.circuit_value[i][g] = FieldElement::from_real(u.try_into().unwrap());
                } else if ty == 4 {
                    self.circuit_value[i][g] = self.circuit_value[i - 1][u];
                } else if ty == 5 {
                    self.circuit_value[i][g] = FieldElement::from_real(0);
                    for k in u..v {
                        self.circuit_value[i][g] =
                            self.circuit_value[i][g] + self.circuit_value[i - 1][k];
                    }
                } else if ty == 6 {
                    self.circuit_value[i][g] =
                        FieldElement::from_real(1) - self.circuit_value[i - 1][u];
                } else if ty == 7 {
                    self.circuit_value[i][g] =
                        self.circuit_value[i - 1][u] - self.circuit_value[i - 1][v];
                } else if ty == 8 {
                    let x = self.circuit_value[i - 1][u];
                    let y = self.circuit_value[i - 1][v];
                    self.circuit_value[i][g] = x + y - FieldElement::from_real(2) * x * y;
                } else if ty == 9 {
                    assert!(
                        u >= 0
                            && u < (1
                                << (*self.aritmetic_circuit.unwrap()).circuit[i - 1].bit_length)
                    );
                    assert!(
                        v >= 0
                            && v < (1
                                << (*self.aritmetic_circuit.unwrap()).circuit[i - 1].bit_length)
                    );
                    let x = self.circuit_value[i - 1][u];
                    let y = self.circuit_value[i - 1][v];
                    self.circuit_value[i][g] = y - x * y;
                } else if ty == 10 {
                    self.circuit_value[i][g] = self.circuit_value[i - 1][u];
                } else if ty == 12 {
                    self.circuit_value[i][g] = FieldElement::from_real(0);
                    assert!(v - u + 1 <= 60);
                    for k in u..=v {
                        self.circuit_value[i][g] = self.circuit_value[i][g]
                            + self.circuit_value[i - 1][k] * FieldElement::from_real(1 << (k - u));
                    }
                } else if ty == 13 {
                    assert!(u == v);
                    assert!(
                        u >= 0
                            && u < (1
                                << (*self.aritmetic_circuit.unwrap()).circuit[i - 1].bit_length),
                    );
                    self.circuit_value[i][g] = self.circuit_value[i - 1][u]
                        * (FieldElement::from_real(1) - self.circuit_value[i - 1][v]);
                } else if ty == 14 {
                    self.circuit_value[i][g] = FieldElement::from_real(0);
                    for k in
                        0..(*self.aritmetic_circuit.unwrap()).circuit[i].gates[g].parameter_length
                    {
                        let weight =
                            (*self.aritmetic_circuit.unwrap()).circuit[i].gates[g].weight[k];
                        let idx = (*self.aritmetic_circuit.unwrap()).circuit[i].gates[g].src[k];
                        self.circuit_value[i][g] =
                            self.circuit_value[i][g] + self.circuit_value[i - 1][idx] * weight;
                    }
                } else {
                    assert!(false);
                }
            }
        }

        let t1 = SystemTime::now();
        let time_span = t1.duration_since(t0);
        println!("total evaluation time: {:?} seconds", time_span.unwrap());

        let depth = (*self.aritmetic_circuit.unwrap()).total_depth;
        //}
        self.circuit_value.pop().unwrap()
        //prletln!("total evaluation time: ");
    }

    pub fn get_witness(&mut self, inputs: Vec<FieldElement>, n: u32) {
        // Do we really need this line of code?
        //self.circuit_value[0] =
        // Vec::with_capacity(1 << (*self.aritmetic_circuit.unwrap()).circuit[0].bit_length);
        self.circuit_value[0] = inputs;
        // todo()
        //self.circuit_value[0] = inputs[..n].to_vec();
    }

    pub fn sumcheck_init(
        &mut self,
        sumcheck_layer_id: usize,
        length_g: usize,
        length_u: usize,
        length_v: usize,
        alpha: FieldElement,
        beta: FieldElement,
        r_0: &Vec<FieldElement>,
        r_1: &Vec<FieldElement>,
        one_minus_r_0: &Vec<FieldElement>,
        one_minus_r_1: &Vec<FieldElement>,
    ) {
        self.r_0 = r_0.clone();
        self.r_1 = r_1.clone();
        self.alpha = alpha;
        self.beta = beta;
        self.sumcheck_layer_id = sumcheck_layer_id;
        self.length_g = length_g;
        self.length_u = length_u;
        self.length_v = length_v;
        self.one_minus_r_0 = one_minus_r_0.clone();
        self.one_minus_r_1 = one_minus_r_1.clone();
    }
    pub fn init_total_time(&mut self, val: u64) {
        self.total_time = val;
    }

    pub unsafe fn sumcheck_phase1_init(&mut self) {
        let t0 = SystemTime::now();
        self.total_uv =
            1 << (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id - 1].bit_length;
        let zero = FieldElement::zero();
        for i in 0..self.total_uv {
            //todo! linear_poly != FieldElement
            //v_mult_add[i] = self.circuit_value[self.sumcheck_layer_id - 1][i];
            self.add_v_array[i].a = zero;
            self.add_v_array[i].b = zero;
            self.add_mult_sum[i].a = zero;
            self.add_mult_sum[i].b = zero;
        }

        self.beta_g_r0_fhalf[0] = self.alpha;
        self.beta_g_r1_fhalf[0] = self.beta;
        self.beta_g_r0_shalf[0] = FieldElement::real_one();
        self.beta_g_r1_shalf[0] = FieldElement::real_one();

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

        let mask_fhalf = (1 << first_half) - 1;
        let mut intermediates0 = vec![FieldElement::zero(); 1 << self.length_g];
        let mut intermediates1 = vec![FieldElement::zero(); 1 << self.length_g];

        //todo
        //	#pragma omp parallel for

        for i in 0..1 << self.length_g {
            let u = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].u;
            let v = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].v;

            match (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].ty {
                0 => {
                    //add gate
                    let tmp = self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half];
                    intermediates0[i] = self.circuit_value[self.sumcheck_layer_id - 1][v] * tmp;
                    intermediates1[i] = tmp;
                }
                1 => {
                    //mult gate
                    let tmp = (self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half]);
                    intermediates0[i] = self.circuit_value[self.sumcheck_layer_id - 1][v] * tmp;
                }
                2 => {}
                5 => {
                    //sum gate
                    let tmp = self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half];
                    intermediates1[i] = tmp;
                }
                12 => {
                    //exp sum gate
                    let tmp = self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half];
                    intermediates1[i] = tmp;
                }
                4 => {
                    //direct relay gate
                    let tmp = (self.beta_g_r0_fhalf[u & mask_fhalf]
                        * self.beta_g_r0_shalf[u >> first_half]
                        + self.beta_g_r1_fhalf[u & mask_fhalf]
                            * self.beta_g_r1_shalf[u >> first_half]);
                    intermediates1[i] = tmp;
                }
                6 => {
                    //NOT gate
                    let tmp = (self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half]);
                    intermediates1[i] = tmp;
                }
                7 => {
                    //minus gate
                    let tmp = (self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half]);
                    intermediates0[i] = self.circuit_value[self.sumcheck_layer_id - 1][v] * tmp;
                    intermediates1[i] = tmp;
                }
                8 => {
                    //XOR gate
                    let tmp = (self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half]);
                    let tmp_V = tmp * self.circuit_value[self.sumcheck_layer_id - 1][v];
                    let tmp_2V = tmp_V + tmp_V;
                    intermediates0[i] = tmp_V;
                    intermediates1[i] = tmp;
                }
                13 => {
                    //bit-test gate
                    let tmp = (self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half]);
                    let tmp_V = tmp * self.circuit_value[self.sumcheck_layer_id - 1][v];
                    intermediates0[i] = tmp_V;
                    intermediates1[i] = tmp;
                }
                9 => {
                    //NAAB gate
                    let tmp = (self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half]);
                    let tmpV = tmp * self.circuit_value[self.sumcheck_layer_id - 1][v];
                    intermediates1[i] = tmpV;
                }
                10 => {
                    //relay gate
                    let tmp = (self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half]);
                    intermediates0[i] = tmp;
                }
                14 => {
                    //custom comb
                    let tmp = self.beta_g_r0_fhalf[i & mask_fhalf]
                        * self.beta_g_r0_shalf[i >> first_half]
                        + self.beta_g_r1_fhalf[i & mask_fhalf]
                            * self.beta_g_r1_shalf[i >> first_half];
                    intermediates1[i] = tmp;
                }
                _ => {
                    println!(
                        "Warning Unknown gate {}",
                        (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i]
                            .ty
                    )
                }
            }
        }
        for i in 0..1 << self.length_g {
            let u = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].u;
            let v = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].v;

            match (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].ty {
                0 => {
                    //add gate
                    if !gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty]
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_v_array[u].b = (self.add_v_array[u].b + intermediates0[i]);
                    self.add_mult_sum[u].b = (self.add_mult_sum[u].b + intermediates1[i]);
                }
                2 => {}
                1 => {
                    //mult gate
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_mult_sum[u].b = (self.add_mult_sum[u].b + intermediates0[i]);
                }
                5 => {
                    //sum gate
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    for j in u..v {
                        self.add_mult_sum[j].b = (self.add_mult_sum[j].b + intermediates1[i]);
                    }
                }

                12 =>
                //exp sum gate
                {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    let mut tmp = intermediates1[i];
                    for j in u..v {
                        self.add_mult_sum[j].b = (self.add_mult_sum[j].b + tmp);
                        tmp = tmp + tmp;
                    }
                }
                14 => {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    let tmp = intermediates1[i];
                    for j in 0..(*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id]
                        .gates[i]
                        .parameter_length
                    {
                        let src = (*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .src[j];
                        let weight = (*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .weight[j];
                        self.add_mult_sum[src].b = self.add_mult_sum[src].b + weight * tmp;
                    }
                }
                4 =>
                //direct relay gate
                {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_mult_sum[u].b = (self.add_mult_sum[u].b + intermediates1[i]);
                }
                6 =>
                //NOT gate
                {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_mult_sum[u].b = (self.add_mult_sum[u].b - intermediates1[i]);
                    self.add_v_array[u].b = (self.add_v_array[u].b + intermediates1[i]);
                }
                7 =>
                //minus gate
                {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_v_array[u].b = (self.add_v_array[u].b - (intermediates0[i]));
                    self.add_mult_sum[u].b = (self.add_mult_sum[u].b + intermediates1[i]);
                }
                8 =>
                //XOR gate
                {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_v_array[u].b = (self.add_v_array[u].b + intermediates0[i]);
                    self.add_mult_sum[u].b = (self.add_mult_sum[u].b + intermediates1[i]
                        - intermediates0[i]
                        - intermediates0[i]);
                }
                13 =>
                //bit-test gate
                {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_mult_sum[u].b =
                        (self.add_mult_sum[u].b - intermediates0[i] + intermediates1[i]);
                }
                9 =>
                //NAAB gate
                {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_v_array[u].b = (self.add_v_array[u].b + intermediates1[i]);
                    self.add_mult_sum[u].b = (self.add_mult_sum[u].b - intermediates1[i]);
                }
                10 =>
                //relay gate
                {
                    if (!gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                        [self.sumcheck_layer_id]
                        .gates[i]
                        .ty])
                    {
                        //prletf("first meet %d gate\n", C -> circuit[self.sumcheck_layer_id].gates[i].ty);
                        gate_meet[(*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .ty] = true;
                    }
                    self.add_mult_sum[u].b = (self.add_mult_sum[u].b + intermediates0[i]);
                }
                _ => println!(
                    "Warning Unknown gate {}",
                    (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].ty
                ),
            }
        }
        let t1 = SystemTime::now();
        let time_span = (t1.duration_since(t0)).unwrap();
        self.total_time += time_span.as_secs();
    }

    //new zk function
    pub unsafe fn sumcheck_phase1_update(
        &mut self,
        previous_random: FieldElement,
        current_bit: usize,
    ) -> QuadraticPoly {
        let t0 = SystemTime::now();
        let mut ret = QuadraticPoly::zero();

        //todo
        //#pragma omp parallel for

        for i in 0..(self.total_uv >> 1) {
            //prime_field::field_element zero_value, one_value; //never used
            let g_zero = i << 1;
            let g_one = i << 1 | 1;
            if (current_bit == 0) {
                v_mult_add_new[i].b = self.v_mult_add[g_zero].b;
                v_mult_add_new[i].a = self.v_mult_add[g_one].b - v_mult_add_new[i].b;

                add_v_array_new[i].b = self.add_v_array[g_zero].b;
                add_v_array_new[i].a = self.add_v_array[g_one].b - add_v_array_new[i].b;

                add_mult_sum_new[i].b = self.add_mult_sum[g_zero].b;
                add_mult_sum_new[i].a = self.add_mult_sum[g_one].b - add_mult_sum_new[i].b;
            } else {
                v_mult_add_new[i].b =
                    (self.v_mult_add[g_zero].a * previous_random + self.v_mult_add[g_zero].b);
                v_mult_add_new[i].a = (self.v_mult_add[g_one].a * previous_random
                    + self.v_mult_add[g_one].b
                    - v_mult_add_new[i].b);

                add_v_array_new[i].b =
                    (self.add_v_array[g_zero].a * previous_random + self.add_v_array[g_zero].b);
                add_v_array_new[i].a = (self.add_v_array[g_one].a * previous_random
                    + self.add_v_array[g_one].b
                    - add_v_array_new[i].b);

                add_mult_sum_new[i].b =
                    (self.add_mult_sum[g_zero].a * previous_random + self.add_mult_sum[g_zero].b);
                add_mult_sum_new[i].a = (self.add_mult_sum[g_one].a * previous_random
                    + self.add_mult_sum[g_one].b
                    - add_mult_sum_new[i].b);
            }
        }
        swap(&mut self.v_mult_add, &mut v_mult_add_new);
        swap(&mut self.add_v_array, &mut add_v_array_new);
        swap(&mut self.add_mult_sum, &mut add_mult_sum_new);

        //parallel addition tree
        //todo
        //#pragma omp parallel for
        for i in 0..(self.total_uv >> 1) {
            rets_prev[i].a = self.add_mult_sum[i].a * self.v_mult_add[i].a;
            rets_prev[i].b = self.add_mult_sum[i].a * self.v_mult_add[i].b
                + self.add_mult_sum[i].b * self.v_mult_add[i].a
                + self.add_v_array[i].a;
            rets_prev[i].c = self.add_mult_sum[i].b * self.v_mult_add[i].b + self.add_v_array[i].b;
        }

        let tot = self.total_uv >> 1;
        let mut iter = 1;
        while (1 << iter) <= (self.total_uv >> 1) {
            //todo
            //#pragma omp parallel for
            for j in 0..(tot >> iter) {
                rets_cur[j] = rets_prev[j * 2] + rets_prev[j * 2 + 1];
            }
            //todo
            //#pragma omp barrier
            swap(&mut rets_prev, &mut rets_cur);
            iter += 1;
        }
        ret = rets_prev[0];

        self.total_uv >>= 1;

        let t1 = SystemTime::now();
        let time_span = (t1.duration_since(t0)).unwrap();
        self.total_time += time_span.as_secs();

        ret
    }

    pub unsafe fn sumcheck_phase2_init(
        &mut self,
        previous_random: FieldElement,
        r_u: Vec<FieldElement>,
        one_minus_r_u: Vec<FieldElement>,
    ) {
        let t0 = SystemTime::now();
        self.v_u = self.v_mult_add[0].eval(previous_random);

        let first_half = self.length_u >> 1;
        let second_half = self.length_u - first_half;

        self.beta_u_fhalf[0] = FieldElement::real_one();
        self.beta_u_shalf[0] = FieldElement::real_one();

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
        let first_g_half = (self.length_g >> 1);
        let mask_g_fhalf = (1 << (self.length_g >> 1)) - 1;

        self.total_uv = (1
            << (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id - 1].bit_length);
        let total_g =
            (1 << (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].bit_length);
        let zero = FieldElement::zero();

        for i in 0..self.total_uv {
            self.add_mult_sum[i].a = zero;
            self.add_mult_sum[i].b = zero;
            self.add_v_array[i].a = zero;
            self.add_v_array[i].b = zero;
            //todo! linear_poly != FieldElement
            //self.v_mult_add[i] = self.circuit_value[self.sumcheck_layer_id - 1][i];
        }

        let mut intermediates0 = vec![FieldElement::zero(); total_g];
        let mut intermediates1 = vec![FieldElement::zero(); total_g];
        //let mut intermediates2 = vec![FieldElement::zero(); total_g]; //never used

        //todo
        //#pragma omp parallel for
        for i in 0..total_g {
            let ty = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].ty;
            let u = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].u;
            let v = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].v;
            match (ty) {
                1 =>
                //mult gate
                {
                    let tmp_u =
                        self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    intermediates0[i] = tmp_g * tmp_u * self.v_u;
                }
                0 =>
                //add gate
                {
                    let tmp_u =
                        self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp_g_u = tmp_g * tmp_u;
                    intermediates0[i] = tmp_g_u;
                    intermediates1[i] = tmp_g_u * self.v_u;
                }
                2 => {}
                4 => {}
                5 =>
                //sum gate
                {
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp_g_vu = tmp_g * self.v_u;
                    intermediates0[i] = tmp_g_vu;
                }
                12 =>
                //exp sum gate
                {
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp_g_vu = tmp_g * self.v_u;
                    intermediates0[i] = tmp_g_vu;
                }
                14 =>
                //custom comb gate
                {
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp_g_vu = tmp_g * self.v_u;
                    intermediates0[i] = tmp_g_vu;
                }

                6 =>
                //not gate
                {
                    let tmp_u =
                        self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp_g_u = tmp_g * tmp_u;
                    intermediates0[i] = tmp_g_u - tmp_g_u * self.v_u;
                }
                7 =>
                //minus gate
                {
                    let tmp_u =
                        self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp = tmp_g * tmp_u;
                    intermediates0[i] = tmp;
                    intermediates1[i] = tmp * self.v_u;
                }
                8 =>
                //xor gate
                {
                    let tmp_u =
                        self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp = tmp_g * tmp_u;
                    let tmp_v_u = tmp * self.v_u;
                    intermediates0[i] = tmp - tmp_v_u - tmp_v_u;
                    intermediates1[i] = tmp_v_u;
                }
                13 =>
                //bit-test gate
                {
                    let tmp_u =
                        self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp = tmp_g * tmp_u;
                    let tmp_v_u = tmp * self.v_u;
                    intermediates0[i] = tmp_v_u;
                }
                9 =>
                //NAAB gate
                {
                    let tmp_u =
                        self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp = tmp_g * tmp_u;
                    intermediates0[i] = tmp - self.v_u * tmp;
                }
                10 =>
                //relay gate
                {
                    let tmp_u =
                        self.beta_u_fhalf[u & mask_fhalf] * self.beta_u_shalf[u >> first_half];
                    let tmp_g = self.beta_g_r0_fhalf[i & mask_g_fhalf]
                        * self.beta_g_r0_shalf[i >> first_g_half]
                        + self.beta_g_r1_fhalf[i & mask_g_fhalf]
                            * self.beta_g_r1_shalf[i >> first_g_half];
                    let tmp = tmp_g * tmp_u;
                    intermediates0[i] = tmp * self.v_u;
                }
                _ => {
                    println!("Warning Unknown gate {}", ty);
                }
            }
        }

        for i in 0..total_g {
            let ty = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].ty;
            let u = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].u;
            let v = (*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id].gates[i].v;
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
                    self.add_v_array[v].b = (intermediates1[i] + self.add_v_array[v].b);
                }
                2 => {}
                4 => {}
                5 =>
                //sum gate
                {
                    for j in u..v {
                        let tmp_u =
                            self.beta_u_fhalf[j & mask_fhalf] * self.beta_u_shalf[j >> first_half];
                        self.add_v_array[0].b = self.add_v_array[0].b + intermediates0[i] * tmp_u;
                    }
                }
                12 =>
                //exp sum gate
                {
                    let mut tmp_g_vu = intermediates0[i];

                    for j in u..v {
                        let tmp_u =
                            self.beta_u_fhalf[j & mask_fhalf] * self.beta_u_shalf[j >> first_half];
                        self.add_v_array[0].b = self.add_v_array[0].b + tmp_g_vu * tmp_u;
                        tmp_g_vu = tmp_g_vu + tmp_g_vu;
                    }
                }
                14 =>
                //custom comb gate
                {
                    let tmp_g_vu = intermediates0[i];

                    for j in 0..(*self.aritmetic_circuit.unwrap()).circuit[self.sumcheck_layer_id]
                        .gates[i]
                        .parameter_length
                    {
                        let src = (*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .src[j];
                        let tmp_u = self.beta_u_fhalf[src & mask_fhalf]
                            * self.beta_u_shalf[src >> first_half];
                        let weight = (*self.aritmetic_circuit.unwrap()).circuit
                            [self.sumcheck_layer_id]
                            .gates[i]
                            .weight[j];
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
                    self.add_v_array[v].b = (intermediates1[i] + self.add_v_array[v].b);
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
    }

    pub fn sumcheck_phase2_update() {}
}

#[cfg(test)]
mod tests {
    use crate::prover::from_string;
    #[test]
    fn prover_from_string() {
        let str = from_string("string");
        assert_eq!(str.real, 7452375);
    }

    #[test]
    fn prover_new() {
        assert_eq!(2 + 2, 4);
    }
}
