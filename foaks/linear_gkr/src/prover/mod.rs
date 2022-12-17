use crate::circuit_fast_track::LayeredCircuit;
use crate::polynomial::{LinearPoly, QuadraticPoly};

use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;

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

pub struct ZKProver<'a> {
    poly_prover: PolyCommitProver,
    /** @name Basic
    	* Basic information and variables about the arithmetic circuit*/
    //< two random gates v_u and v_v queried by V in each layer    v_u: FieldElement,
    v_v: FieldElement,
    pub total_uv: i32,
    pub aritmetic_circuit: Option<LayeredCircuit<'a>>, //	c++ code: layered_circuit *C;
    pub circuit_value: [Vec<FieldElement>; 1000000],
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

impl<'a> ZKProver<'a> {
    pub fn new(half_length: usize) -> Self {
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
            poly_prover: todo!(),
            v_v: todo!(),
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

    pub fn get_circuit(&mut self, from_verifier: Option<LayeredCircuit<'a>>) {
        self.aritmetic_circuit = from_verifier;
        unsafe {
            INV_2 = FieldElement::from_real(2);
        }
    }

    pub fn V_res(
        one_minus_r_0: FieldElement,
        r_0: FieldElement,
        output_raw: FieldElement,
        r_0_size: FieldElement,
        output_size: FieldElement,
    ) {
    }

    pub fn evaluate() {
        unimplemented!()
    }

    pub fn get_witness(&mut self, inputs: Vec<FieldElement>, n: u32) {
        self.circuit_value[0] =
            Vec::with_capacity(1 << self.aritmetic_circuit.as_ref().unwrap().circuit[0].bit_length);
        self.circuit_value[0] = inputs;
    }
    pub fn sumcheck_init() {}

    pub fn delete_self() {}

    pub fn sumcheck_phase1_init() {}

    pub fn sumcheck_phase1_update() {}

    pub fn sumcheck_phase2_init() {}

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
}
