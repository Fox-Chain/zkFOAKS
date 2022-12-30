use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;

use crate::circuit_fast_track::Gate;
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
        // println!("{} {}", path, meta_path);
        let circuit_path = Path::new(path);
        // println!("{:?}", circuit_path);
        let circuit_file = File::open(path).unwrap();
        // println!("{:?}", circuit_file);
        let circuit_reader = BufReader::new(circuit_file);

        let mut lines_iter = circuit_reader.lines().map(|l| l.unwrap());
        let d: usize = lines_iter.next().unwrap().parse().unwrap();

        println!("d: {:?}", d);

        // NEED WORK HERE !!!
        self.aritmetic_circuit.circuit = vec![Layer::new(); 2];
        self.aritmetic_circuit.total_depth = d + 1;
        println!(
            "aritmetic_circuit.circuit: {:?}",
            self.aritmetic_circuit.circuit
        );

        let max_bit_length = -1;
        let mut n_pad: usize;
        for i in 1..=d {
            let pad_requirement: usize;

            let next_line = lines_iter.next().unwrap();
            let mut next_line_splited = next_line.split_whitespace();
            let mut number_gates: usize = next_line_splited.next().unwrap().parse().unwrap();
            println!("n: {}", number_gates);
            if d > 3 {
                pad_requirement = 17;
            } else {
                pad_requirement = 15;
            }
            if i == 1 && number_gates < (1 << pad_requirement) {
                n_pad = 1 << pad_requirement;
                println!("n_pad: {}", n_pad)
            } else {
                n_pad = number_gates;
            }

            if i != 1 {
                // The circuit vector have issue none of the circuit[].gate work. Break when i = 1
                // Error when try to print self.aritmetic_circuit.circuit[0]
                if number_gates == 1 {
                    self.aritmetic_circuit.circuit[i].gates = vec![Gate::new(); 2];
                } else {
                    self.aritmetic_circuit.circuit[i].gates = vec![Gate::new(); n_pad];
                }
            } else {
                if number_gates == 1 {
                    self.aritmetic_circuit.circuit[0].gates = vec![Gate::new(); 2];
                    self.aritmetic_circuit.circuit[1].gates = vec![Gate::new(); 2];
                    // println!("{:?}", self.aritmetic_circuit.circuit[1].gates);
                } else {
                    //println!("check i: {}", i);
                    //let _gate = Layer {
                    //  gates: Vec::with_capacity(2),
                    //  src_expander_c_mempool: todo!(),
                    //  src_expander_d_mempool: todo!(),
                    // weight_expander_c_mempool: todo!(),
                    //  weight_expander_d_mempool: todo!(),
                    //  bit_length: todo!(),
                    // u_gates: todo!(),
                    // v_gates: todo!(),
                    // is_parallel: todo!(),
                    // block_size: todo!(),
                    // log_block_size: todo!(),
                    //  repeat_num: todo!(),
                    // log_repeat_num: todo!(),
                    // };
                    //self.aritmetic_circuit.circuit.push(_gate);
                    self.aritmetic_circuit.circuit[0].gates = vec![Gate::new(); n_pad];
                    self.aritmetic_circuit.circuit[1].gates = vec![Gate::new(); n_pad];
                }
            }
            let max_gate = -1;
            let mut previous_g: isize = -1;

            for j in 0..number_gates {
                let ty: u32 = next_line_splited.next().unwrap().parse().unwrap();
                let g: isize = next_line_splited.next().unwrap().parse().unwrap();
                let u: usize = next_line_splited.next().unwrap().parse().unwrap();
                let mut v: usize = next_line_splited.next().unwrap().parse().unwrap();

                if j % 100 == 0 {
                    println!("ty:{} g:{} u:{} v:{}", ty, g, u, v,);
                }

                if ty != 3 {
                    if ty == 5 {
                        assert!(
                            u >= 0 && u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                        );
                        assert!(
                            v > u && v <= (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                        );
                    } else {
                        if !(u >= 0 && u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length))
                        {
                            println!(
                                "{} {} {} {} {} ",
                                ty,
                                g,
                                u,
                                v,
                                (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                            );
                        }
                        assert!(
                            u >= 0 && u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                        );
                        if !(v >= 0 && v < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length))
                        {
                            println!(
                                "{} {} {} {} {} ",
                                ty,
                                g,
                                u,
                                v,
                                (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                            );
                        }
                        assert!(
                            v >= 0 && v < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                        );
                    }
                }
                if ty == 6 {
                    if v != 0 {
                        //todo: improve error handling
                        println!("WARNING, v!=0 for NOT gate")
                    }
                    v = 0;
                }
                if ty == 10 {
                    if v != 0 {
                        //todo: improve error handling
                        println!("WARNING, v!=0 for relay gate {}", i)
                    }
                    v = 0;
                }
                if ty == 13 {
                    assert!(u == v);
                }
                if g != previous_g + 1 {
                    //todo: improve error handling
                    println!(
                        "Error, gates must be in sorted order, and full [0, 2^n - 1]. {} {} {} {}",
                        i, j, g, previous_g
                    );
                    panic!();
                }
                previous_g = g;
                if i != 1 {
                    self.aritmetic_circuit.circuit[i].gates[g as usize] =
                        Gate::from_params(ty, u, v);
                } else {
                    assert!(ty == 2 || ty == 3);
                    self.aritmetic_circuit.circuit[1].gates[g as usize] =
                        Gate::from_params(4, g as usize, 0);
                    self.aritmetic_circuit.circuit[0].gates[g as usize] =
                        Gate::from_params(ty, u, v);
                }

                //break;
            }
            if i == 1 {
                for g in number_gates..n_pad {
                    self.aritmetic_circuit.circuit[1].gates[g] = Gate::from_params(4, g, 0);
                    self.aritmetic_circuit.circuit[0].gates[g] = Gate::from_params(3, 0, 0);
                }
                number_gates = n_pad;
                previous_g = n_pad as isize - 1;
                println!(
                    "aritmetic_circuit.circuit[1].gates[0]: {:?}",
                    self.aritmetic_circuit.circuit[1].gates[0]
                );
            }
            //can comment the below break sentence to let the loop continue
            break;
        }
    }
}
