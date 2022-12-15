use crate::circuit_fast_track::LayeredCircuit;
use crate::polynomial::LinearPoly;

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

    add_v_array: LinearPoly,
    v_mult_add: LinearPoly,
    beta_g_r0_fhalf: Vec<FieldElement>,
    beta_g_r0_shalf: Vec<FieldElement>,
    beta_g_r1_fhalf: Vec<FieldElement>,
    beta_g_r1_shalf: Vec<FieldElement>,
    beta_u_fhalf: Vec<FieldElement>,
    beta_u_shalf: Vec<FieldElement>,
    beta_u: Vec<FieldElement>,
    beta_v: Vec<FieldElement>,
    beta_g: Vec<FieldElement>,

    total_time: u64,
}

static mut INV_2: FieldElement = FieldElement::zero();

impl<'a> ZKProver<'a> {
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

    pub fn init_array() {}

    pub fn delete_self() {}

    pub fn sumcheck_phase1_init() {}

    pub fn sumcheck_phase1_update() {}

    pub fn sumcheck_phase2_init() {}

    pub fn sumcheck_phase2_update() {}
}
