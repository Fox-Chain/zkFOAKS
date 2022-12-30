use crate::circuit_fast_track::LayeredCircuit;
use crate::polynomial::{LinearPoly, QuadraticPoly};

use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;

use std::time::SystemTime;

static mut INV_2: FieldElement = FieldElement::zero();
static mut V_mult_add_new: Vec<FieldElement> = Vec::new();
static mut addV_array_new: Vec<FieldElement> = Vec::new();
static mut add_mult_sum_new: Vec<FieldElement> = Vec::new();
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

pub struct zk_prover<'a> {
    //poly_prover: PolyCommitProver,
    /** @name Basic
    	* Basic information and variables about the arithmetic circuit*/
    //< two random gates v_u and v_v queried by V in each layer    v_u: FieldElement,
    v_v: FieldElement,
    u_v: FieldElement,
    pub total_uv: i32,
    pub aritmetic_circuit: Option<&'a LayeredCircuit>, //	c++ code: layered_circuit *C;
    pub circuit_value: Vec<Vec<FieldElement>>,
    sumcheck_layer_id: u32,
    length_g: u32,
    length_u: u32,
    length_v: u32,

    /** @name Randomness
    	* Some randomness or values during the proof phase. */
    alpha: FieldElement,
    beta: FieldElement,

    //< c++ code: const prime_field::field_element *r_0, *r_1; How to deal with "const"
    r_0: Vec<FieldElement>,
    r_1: Vec<FieldElement>,
    one_minus_r_0: Vec<FieldElement>,
    one_minus_r_1: Vec<FieldElement>,

    add_v_array: Vec<LinearPoly>,
    v_mult_add: Vec<LinearPoly>,
    beta_g_r0_fhalf: Vec<FieldElement>,
    beta_g_r0_shalf: Vec<FieldElement>,
    beta_g_r1_fhalf: Vec<FieldElement>,
    beta_g_r1_shalf: Vec<FieldElement>,
    beta_u_fhalf: Vec<FieldElement>,
    beta_u_shalf: Vec<FieldElement>,
    beta_u: Vec<FieldElement>,
    beta_v: Vec<FieldElement>,
    beta_g: Vec<FieldElement>,
    add_mult_sum: Vec<LinearPoly>,

    total_time: u64,
}

impl<'a> zk_prover<'a> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn new_2(half_length: usize) -> Self {
        Self {
            beta_g_r0_fhalf: todo!(),
            beta_g_r0_shalf: todo!(),
            beta_g_r1_fhalf: todo!(),
            beta_g_r1_shalf: todo!(),
            beta_u_fhalf: todo!(),
            beta_u_shalf: todo!(),
            add_mult_sum: todo!(),
            v_mult_add: todo!(),
            add_v_array: todo!(),
            //poly_prover: todo!(),
            v_v: todo!(),
            u_v: todo!(),
            total_uv: todo!(),
            aritmetic_circuit: todo!(),
            circuit_value: todo!(),
            sumcheck_layer_id: todo!(),
            length_g: todo!(),
            length_u: todo!(),
            length_v: todo!(),
            alpha: todo!(),
            beta: todo!(),
            r_0: todo!(),
            r_1: todo!(),
            one_minus_r_0: todo!(),
            one_minus_r_1: todo!(),
            beta_u: todo!(),
            beta_v: todo!(),
            beta_g: todo!(),
            total_time: todo!(),
        }
    }

    pub fn init_array(&mut self, max_bit_length: usize) {
        let half_length = (max_bit_length >> 1) + 1;

        unsafe {
            gate_meet = [false; 15];
            V_mult_add_new = Vec::with_capacity(1 << half_length);
            addV_array_new = Vec::with_capacity(1 << half_length);
            add_mult_sum_new = Vec::with_capacity(1 << half_length);
            rets_prev = Vec::with_capacity(1 << half_length);
            rets_cur = Vec::with_capacity(1 << half_length);
        }

        Self::init_zkprover(self, half_length);
    }

    pub fn init_zkprover(&mut self, half_length: usize) {
        self.beta_g_r0_fhalf = Vec::with_capacity(1 << half_length);
        self.beta_g_r0_shalf = Vec::with_capacity(1 << half_length);
        self.beta_g_r1_fhalf = Vec::with_capacity(1 << half_length);
        self.beta_g_r1_shalf = Vec::with_capacity(1 << half_length);
        self.beta_u_fhalf = Vec::with_capacity(1 << half_length);
        self.beta_u_shalf = Vec::with_capacity(1 << half_length);
        self.add_mult_sum = Vec::with_capacity(1 << half_length);
        self.v_mult_add = Vec::with_capacity(1 << half_length);
        self.add_v_array = Vec::with_capacity(1 << half_length);
    }

    pub fn get_circuit(&mut self, from_verifier: &'a LayeredCircuit) {
        self.aritmetic_circuit = Some(from_verifier);
        unsafe {
            INV_2 = FieldElement::from_real(2);
        }
    }

    // this function calculate time ?
    // 	prime_field::field_element a_0 = p -> V_res(one_minus_r_0, r_0, result, C.circuit[C.total_depth - 1].bit_length, (1 << (C.circuit[C.total_depth - 1].bit_length)));

    pub fn V_res(
        one_minus_r_0: FieldElement,
        r_0: FieldElement,
        output_raw: FieldElement,
        r_0_size: u64,
        output_size: u64,
    ) {
        let sys_time = SystemTime::now();
        let mut output: FieldElement;
        output = FieldElement::from_real(output_size);
        // for i in 0..output_size {
        //     output[i] = output_raw[i];
        // }
        // for i in 0..r_0_size {

        // }
    }

    pub fn evaluate(&mut self) -> Vec<FieldElement> {
        todo!();
        let t0 = SystemTime::now();
        let halt = 1 << self.aritmetic_circuit.as_ref().unwrap().circuit[0].bit_length;
        for i in 0..halt {
            let g = i;
            let u = self.aritmetic_circuit.as_ref().unwrap().circuit[0].gates[0].u;
            let ty = self.aritmetic_circuit.as_ref().unwrap().circuit[0].gates[0].ty;
            assert!(ty == 3 || ty == 2);
        }
        assert!(self.aritmetic_circuit.as_ref().unwrap().total_depth < 1000000);
        for i in 0..self.aritmetic_circuit.as_ref().unwrap().total_depth {
            self.circuit_value[i] = Vec::with_capacity(
                1 << self.aritmetic_circuit.as_ref().unwrap().circuit[i].bit_length,
            );
            let halt = self.aritmetic_circuit.as_ref().unwrap().circuit[i].bit_length;
            for j in 0..halt {
                let g = j;
                let ty: u32 = self.aritmetic_circuit.as_ref().unwrap().circuit[i].gates[g].ty;
                let u = self.aritmetic_circuit.as_ref().unwrap().circuit[i].gates[g].u;
                let v = self.aritmetic_circuit.as_ref().unwrap().circuit[i].gates[g]
                    .v
                    .try_into()
                    .unwrap();
                if (ty == 0) {
                    self.circuit_value[i][g] =
                        self.circuit_value[i - 1][u] + self.circuit_value[i - 1][v];
                } else if (ty == 1) {
                    assert!(
                        u >= 0
                            && u < (1
                                << self.aritmetic_circuit.as_ref().unwrap().circuit[i - 1]
                                    .bit_length),
                    );
                    assert!(
                        v >= 0
                            && v < (1
                                << self.aritmetic_circuit.as_ref().unwrap().circuit[i - 1]
                                    .bit_length),
                    );
                    self.circuit_value[i][g] =
                        self.circuit_value[i - 1][u] * self.circuit_value[i - 1][v];
                } else if (ty == 2) {
                    self.circuit_value[i][g] = FieldElement::from_real(0);
                } else if (ty == 3) {
                    self.circuit_value[i][g] = FieldElement::from_real(u.try_into().unwrap());
                } else if (ty == 4) {
                    self.circuit_value[i][g] = self.circuit_value[i - 1][u];
                } else if (ty == 5) {
                    self.circuit_value[i][g] = FieldElement::from_real(0);
                    for k in u..v {
                        self.circuit_value[i][g] =
                            self.circuit_value[i][g] + self.circuit_value[i - 1][k];
                    }
                } else if (ty == 6) {
                    self.circuit_value[i][g] =
                        FieldElement::from_real(1) - self.circuit_value[i - 1][u];
                } else if (ty == 7) {
                    self.circuit_value[i][g] =
                        self.circuit_value[i - 1][u] - self.circuit_value[i - 1][v];
                } else if (ty == 8) {
                } else if (ty == 9) {
                } else if (ty == 10) {
                    self.circuit_value[i][g] = self.circuit_value[i - 1][u];
                } else if (ty == 12) {
                    self.circuit_value[i][g] = FieldElement::from_real(0);
                    assert!(v - u + 1 <= 60);
                    for k in u..=v {
                        self.circuit_value[i][g] = self.circuit_value[i][g]
                            + self.circuit_value[i - 1][k] * FieldElement::from_real(1 << (k - u));
                    }
                } else if (ty == 13) {
                    assert!(u == v);
                    assert!(
                        u >= 0
                            && u < (1
                                << self.aritmetic_circuit.as_ref().unwrap().circuit[i - 1]
                                    .bit_length),
                    );
                    self.circuit_value[i][g] = self.circuit_value[i - 1][u]
                        * (FieldElement::from_real(1) - self.circuit_value[i - 1][v]);
                } else if (ty == 14) {
                    self.circuit_value[i][g] = FieldElement::from_real(0);
                    for k in 0..self.aritmetic_circuit.as_ref().unwrap().circuit[i].gates[g]
                        .parameter_length
                    {
                        unimplemented!()
                    }
                } else {
                    assert!(false);
                }
            }
        }
        let t1 = SystemTime::now();
        let life_span = t1.duration_since(t0);
        let index = self.aritmetic_circuit.as_ref().unwrap().total_depth;
        todo!()
    }

    pub fn get_witness(&mut self, inputs: Vec<FieldElement>, n: u32) {
        // Do we really need this line of code?
        //self.circuit_value[0] =
        // Vec::with_capacity(1 << self.aritmetic_circuit.unwrap().circuit[0].bit_length);
        self.circuit_value[0] = inputs;
        // todo()
        //self.circuit_value[0] = inputs[..n].to_vec();
    }

    pub fn sumcheck_init(
        &mut self,
        sumcheck_layer_id: u32,
        length_g: u32,
        length_u: u32,
        length_v: u32,
        alpha: FieldElement,
        beta: FieldElement,
        r_0: Vec<FieldElement>,
        r_1: Vec<FieldElement>,
        one_minus_r_0: Vec<FieldElement>,
        one_minus_r_1: Vec<FieldElement>,
    ) {
        self.r_0 = r_0;
        self.r_1 = r_1;
        self.alpha = alpha;
        self.beta = beta;
        self.sumcheck_layer_id = sumcheck_layer_id;
        self.length_g = length_g;
        self.length_u = length_u;
        self.length_v = length_v;
        self.one_minus_r_0 = one_minus_r_0;
        self.one_minus_r_1 = one_minus_r_1;
    }
    pub fn init_total_time(&mut self, val: u64) {
        self.total_time = val;
    }
}

pub fn delete_self() {
    self::delete_self();
}

pub fn sumcheck_phase1_init() {
    let t0 = SystemTime::now();
}

pub fn sumcheck_phase1_update() {}

pub fn sumcheck_phase2_init() {}

pub fn sumcheck_phase2_update() {}

#[cfg(test)]
mod tests {
    use crate::{
        prover::{from_string, zk_prover},
        verifier::zk_verifier,
    };

    #[test]
    fn prover_verifer_interaction() {
        let mut zkv = zk_verifier::new();
        let mut zkp = zk_prover::new();
        zkp.init_total_time(5);
        println!("{:?}", zkp.total_time);

        zkv.get_prover(&mut zkp);
        println!("{:?}", zkv.prover.unwrap().total_time);

        //todo()
        //zkp.init_total_time(50);
        //println!("{:?}", zkv.prover.unwrap().total_time);
    }
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
