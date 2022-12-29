use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;

use crate::circuit_fast_track::Layer;
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
#[derive(Default, Debug)]
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

    pub fn read_circuit(&mut self, path: &String, meta_path: &String) {
        println!("{} {}", path, meta_path);
        let circuit_path = Path::new(path);
        println!("{:?}", circuit_path);
        let circuit_file = File::open(path).unwrap();
        println!("{:?}", circuit_file);
        let mut circuit_reader = BufReader::new(circuit_file);

        let mut lines_iter = circuit_reader.lines().map(|l| l.unwrap());
        let d: usize = lines_iter.next().unwrap().parse().unwrap();

        println!("{:?}", d);

        self.aritmetic_circuit.circuit = Vec::with_capacity(d);
        self.aritmetic_circuit.total_depth = d + 1;

        let max_bit_length = -1;
        let mut n_pad: usize;
        for i in 1..=d {
            let pad_requirement: usize;

            let next_line = lines_iter.next().unwrap();
            let mut next_line_splited = next_line.split_whitespace();
            let n: usize = next_line_splited.next().unwrap().parse().unwrap();
            println!("{}", n);
            if d > 3 {
                pad_requirement = 17;
            } else {
                pad_requirement = 15;
            }

            if i == 1 && n < (1 << pad_requirement) {
                n_pad = (1 << pad_requirement);
            } else {
                n_pad = n;
            }

            if i != 1 {
                if n == 1 {
                    self.aritmetic_circuit.circuit[i].gates = Vec::with_capacity(2);
                } else {
                    self.aritmetic_circuit.circuit[i].gates = Vec::with_capacity(n_pad);
                }
            } else {
                if n == 1 {
                    self.aritmetic_circuit.circuit[0].gates = Vec::with_capacity(2);
                    self.aritmetic_circuit.circuit[1].gates = Vec::with_capacity(2);
                } else {
                    //self.aritmetic_circuit.circuit[0].gates = Vec::with_capacity(n_pad);
                    //self.aritmetic_circuit.circuit[1].gates = Vec::with_capacity(n_pad);
                }
            }
            let max_gate = -1;
            let previous_g = -1;

            let ty: u32 = next_line_splited.next().unwrap().parse().unwrap();
            let g: u32 = next_line_splited.next().unwrap().parse().unwrap();
            let u: usize = next_line_splited.next().unwrap().parse().unwrap();
            let v: usize = next_line_splited.next().unwrap().parse().unwrap();

            println!("{} {} {} {}", ty, g, u, v,);
            //println!(
            //"{}",
            //(1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
            //);

            break;
        }
    }
}
