use crate::circuit_fast_track::LayeredCircuit;
use crate::polynomial::{LinearPoly, QuadraticPoly};

use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;

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
    total_uv: u32,
    aritmetic_circuit: Vec<LayeredCircuit<'a>>, //	c++ code: layered_circuit *C;
    circuit_value: Vec<FieldElement>,
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

static mut INV_2: FieldElement = FieldElement::zero();

impl<'a> ZKProver<'a> {
    pub fn init_zkprover(half_length: usize) -> Self {
        Self {
            beta_g_r0_fhalf: Vec::with_capacity(1 << half_length),
            beta_g_r0_shalf: Vec::with_capacity(1 << half_length),
            beta_g_r1_fhalf: Vec::with_capacity(1 << half_length),
            beta_g_r1_shalf: Vec::with_capacity(1 << half_length),
            beta_u_fhalf: Vec::with_capacity(1 << half_length),
            beta_u_shalf: Vec::with_capacity(1 << half_length),
            add_mult_sum: Vec::with_capacity(1 << half_length),
            v_mult_add: Vec::with_capacity(1 << half_length),
            add_v_array: Vec::with_capacity(1 << half_length),
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
    pub fn get_circuit(from_verifier: &LayeredCircuit) {
        let C = from_verifier;
        // INV_2 = FieldElement::from_real(2);
    }

    pub fn V_res(
        one_minus_r_0: FieldElement,
        r_0: FieldElement,
        output_raw: FieldElement,
        r_0_size: FieldElement,
        output_size: FieldElement,
    ) {
    }

    pub fn evaluate() {}

    pub fn get_witness() {}

    pub fn sumcheck_init() {}

    pub fn init_array(
        max_bit_length: usize,
        ProverContext {
            V_mult_add_new,
            addV_array_new,
            add_mult_sum_new,
            gate_meet,
            rets_prev,
            rets_cur,
            zkprover,
        }: &mut ProverContext<'a>,
    ) {
        *gate_meet = [false; 15];
        let half_length = (max_bit_length >> 1) + 1;
        *zkprover = Self::init_zkprover(half_length);
        *V_mult_add_new = Vec::with_capacity(1 << half_length);
        *addV_array_new = Vec::with_capacity(1 << half_length);
        *add_mult_sum_new = Vec::with_capacity(1 << half_length);
        *rets_prev = Vec::with_capacity(1 << half_length);
        *rets_cur = Vec::with_capacity(1 << half_length);
    }
    pub fn delete_self() {}

    pub fn sumcheck_phase1_init() {}

    pub fn sumcheck_phase1_update() {}

    pub fn sumcheck_phase2_init() {}

    pub fn sumcheck_phase2_update() {}
}

pub struct ProverContext<'a> {
    V_mult_add_new: Vec<FieldElement>,
    addV_array_new: Vec<FieldElement>,
    add_mult_sum_new: Vec<FieldElement>,
    gate_meet: [bool; 15],
    rets_prev: Vec<QuadraticPoly>,
    rets_cur: Vec<QuadraticPoly>,
    zkprover: ZKProver<'a>,
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
