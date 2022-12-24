use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;

use crate::circuit_fast_track::LayeredCircuit;
use crate::prover::zk_prover;
enum gate_types {
    add = 0,
    mult = 1,
    dummy = 2,
    sum = 5,
    exp_sum = 12,
    direct_relay = 4,
    not_gate = 6,
    minus = 7,
    xor_gate = 8,
    bit_test = 13,
    relay = 10,
    custom_linear_comb = 14,
    input = 3,
}
#[derive(Default)]

pub struct zk_verifier<'a> {
    pub proof_size: u32,
    pub v_time: u64,
    //poly_verifier: PolyCommitVerifier,
    /** @name Randomness&Const
    	* Storing randomness or constant for simplifying computation*/
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

    pub aritmetic_circuit: LayeredCircuit, // The circuit
    pub prover: Option<&'a zk_prover<'a>>, // The prover

    VPD_randomness: Vec<FieldElement>,
    one_minus_VPD_randomness: Vec<FieldElement>,
}

impl<'a> zk_verifier<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_prover(&mut self, prover__: &'a zk_prover) {
        self.prover = Some(prover__);
    }
}
