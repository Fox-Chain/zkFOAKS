use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

// use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;

use crate::circuit_fast_track::Gate;
use crate::circuit_fast_track::Layer;
use crate::circuit_fast_track::LayeredCircuit;
use crate::prover::zk_prover;

use std::time::SystemTime;
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
pub struct zk_verifier {
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
    pub prover: Option<*mut zk_prover>,    // The prover

    VPD_randomness: Vec<FieldElement>,
    one_minus_VPD_randomness: Vec<FieldElement>,
}

impl zk_verifier {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_prover(&mut self, prover__: *mut zk_prover) {
        self.prover = Some(prover__);
    }

    pub fn read_circuit(&mut self, path: &String, meta_path: &String) {
        let circuit_file = File::open(path).unwrap();
        let circuit_reader = BufReader::new(circuit_file);

        let mut lines_iter = circuit_reader.lines().map(|l| l.unwrap());
        let d: usize = lines_iter.next().unwrap().parse().unwrap();

        println!("d: {:?}", d);

        self.aritmetic_circuit.circuit = vec![Layer::new(); d + 1];
        self.aritmetic_circuit.total_depth = d + 1;
        println!(
            "aritmetic_circuit.circuit[0]: {:?}",
            self.aritmetic_circuit.circuit[0]
        );

        let mut max_bit_length: isize = -1;
        let mut n_pad: usize;
        for i in 1..=d {
            let pad_requirement: usize;

            let next_line = lines_iter.next().unwrap();
            let mut next_line_splited = next_line.split_whitespace();
            let mut number_gates: usize = next_line_splited.next().unwrap().parse().unwrap();
            println!("number_gates: {}", number_gates);
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
                    self.aritmetic_circuit.circuit[i].gates = vec![Gate::new(); 2];
                } else {
                    self.aritmetic_circuit.circuit[i].gates = vec![Gate::new(); n_pad];
                }
            } else {
                if number_gates == 1 {
                    self.aritmetic_circuit.circuit[0].gates = vec![Gate::new(); 2];
                    self.aritmetic_circuit.circuit[1].gates = vec![Gate::new(); 2];
                } else {
                    self.aritmetic_circuit.circuit[0].gates = vec![Gate::new(); n_pad];
                    self.aritmetic_circuit.circuit[1].gates = vec![Gate::new(); n_pad];
                }
            }
            let mut max_gate = -1;
            let mut previous_g: isize = -1;

            for j in 0..number_gates {
                let ty: usize = next_line_splited.next().unwrap().parse().unwrap();
                let g: isize = next_line_splited.next().unwrap().parse().unwrap();
                let u: usize = next_line_splited.next().unwrap().parse().unwrap();
                let mut v: usize = next_line_splited.next().unwrap().parse().unwrap();

                if j % number_gates / 4 == 0 {
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

            max_gate = previous_g;
            let mut cnt = 0;
            while max_gate > 0 {
                cnt += 1;
                max_gate >>= 1;
            }
            max_gate = 1;
            while cnt > 0 {
                max_gate <<= 1;
                cnt -= 1;
            }
            let mut mx_gate = max_gate;
            while mx_gate > 0 {
                cnt += 1;
                mx_gate >>= 1;
            }
            if number_gates == 1 {
                //add a dummy gate to avoid ill-defined layer.
                if i != 1 {
                    self.aritmetic_circuit.circuit[i].gates[max_gate as usize] =
                        Gate::from_params(2, 0, 0);
                    self.aritmetic_circuit.circuit[i].bit_length = cnt;
                } else {
                    self.aritmetic_circuit.circuit[0].gates[max_gate as usize] =
                        Gate::from_params(2, 0, 0);
                    self.aritmetic_circuit.circuit[0].bit_length = cnt;
                    self.aritmetic_circuit.circuit[1].gates[max_gate as usize] =
                        Gate::from_params(4, 1, 0);
                    self.aritmetic_circuit.circuit[1].bit_length = cnt;
                }
            } else {
                self.aritmetic_circuit.circuit[i].bit_length = cnt - 1;
                if i == 1 {
                    self.aritmetic_circuit.circuit[0].bit_length = cnt - 1;
                }
            }
            //todo: improve error handling
            println!(
                "layer {}, bit_length {}",
                i, self.aritmetic_circuit.circuit[i].bit_length
            );
            if self.aritmetic_circuit.circuit[i].bit_length as isize > max_bit_length {
                max_bit_length = self.aritmetic_circuit.circuit[i].bit_length as isize;
            }

            //can uncomment the below break sentence to break the for loop
            //break;
        }
        self.aritmetic_circuit.circuit[0].is_parallel = false;

        let meta_file = File::open(meta_path).unwrap();
        let meta_reader = BufReader::new(meta_file);

        let mut meta_lines_iter = meta_reader.lines().map(|l| l.unwrap());
        for i in 1..=d {
            let meta_line = meta_lines_iter.next().unwrap();
            let mut meta_line_splited = meta_line.split_whitespace();

            let is_para: usize = meta_line_splited.next().unwrap().parse().unwrap();
            self.aritmetic_circuit.circuit[i].block_size =
                meta_line_splited.next().unwrap().parse().unwrap();
            self.aritmetic_circuit.circuit[i].repeat_num =
                meta_line_splited.next().unwrap().parse().unwrap();
            self.aritmetic_circuit.circuit[i].log_block_size =
                meta_line_splited.next().unwrap().parse().unwrap();
            self.aritmetic_circuit.circuit[i].log_repeat_num =
                meta_line_splited.next().unwrap().parse().unwrap();

            //println!("meta_line: {:?}", meta_line);

            if is_para != 0 {
                assert!(
                    1 << self.aritmetic_circuit.circuit[i].log_repeat_num
                        == self.aritmetic_circuit.circuit[i].repeat_num
                );
            }
            if is_para != 0 {
                self.aritmetic_circuit.circuit[i].is_parallel = true;
            } else {
                self.aritmetic_circuit.circuit[i].is_parallel = false;
            }
        }
        unsafe {
            let x = self.prover.unwrap();
            (*x).init_array(max_bit_length.try_into().unwrap());
        }

        println!("max_bit_length:{}", max_bit_length);
        Self::init_array(self, max_bit_length);
    }

    pub fn init_array(&mut self, max_bit_length: isize) {
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

    //Decided to implemente the verify() function from orion repo
    pub unsafe fn verify_orion(&mut self, output_path: &String) {
        self.proof_size = 0;
        //there is a way to compress binlinear pairing element
        let verification_time = 0;
        let predicates_calc_time = 0;
        let verification_rdl_time = 0;

        //Below function is not implemented neither in virgo repo nor orion repo
        //prime_field::init_random();

        //Below function is not implemented neither in virgo repo nor orion repo
        //self.prover.unwrap().proof_init();

        // unsafe {
        let zkp = self.prover.unwrap();
        let result = (*zkp).evaluate();
        // }
        let alpha = FieldElement::from_real(1);
        let beta = FieldElement::from_real(0);
        //	random_oracle oracle; // Orion dont use this here
        let capacity =
            self.aritmetic_circuit.circuit[self.aritmetic_circuit.total_depth - 1].bit_length;
        let r_0 = Self::generate_randomness(capacity);
        let r_1 = Self::generate_randomness(capacity);
        let mut one_minus_r_0 = vec![FieldElement::zero(); capacity];
        let mut one_minus_r_1 = vec![FieldElement::zero(); capacity];

        for i in 0..capacity {
            one_minus_r_0.push(FieldElement::from_real(1) - r_0[i]);
            one_minus_r_1.push(FieldElement::from_real(1) - r_1[i]);
        }
        let t_a = SystemTime::now();
        println!("Calc V_output(r)");
        // unsafe{
        let mut a_0 =
            (*self.prover.unwrap()).V_res(one_minus_r_0, r_0, result, capacity, (1 << capacity));
        // }
        let t_b = SystemTime::now();
        let time_span = (t_b.duration_since(t_a)).unwrap();
        println!("microsecs: {}", time_span.as_micros());
        a_0 = alpha * a_0;
        let alpha_beta_sum = a_0;
        let direct_relay_value: FieldElement;

        for i in (self.aritmetic_circuit.total_depth - 1)..1 {
            let rho = FieldElement::new_random();
            //todo: solve bug
            //(*self.prover.unwrap()).sumcheck_init(
            //  i,
            //  self.aritmetic_circuit.circuit[i].bit_length,
            //self.aritmetic_circuit.circuit[i - 1].bit_length,
            //self.aritmetic_circuit.circuit[i - 1].bit_length,
            //alpha,
            //beta,
            //r_0,
            //r_1,
            //one_minus_r_0,
            //one_minus_r_1,
            //);

            (*self.prover.unwrap()).sumcheck_phase1_init();

            let previous_random = FieldElement::from_real(0);
            //next level random
            let r_u = Self::generate_randomness(self.aritmetic_circuit.circuit[i - 1].bit_length);
            let mut r_v =
                Self::generate_randomness(self.aritmetic_circuit.circuit[i - 1].bit_length);

            //todo: solve bug
            //let direct_relay_value = alpha * Self::direct_relay(self, i, r_0.clone(), r_u.clone())
            //  + beta * Self::direct_relay(self, i, r_1.clone(), r_u.clone());

            if (i == 1) {
                for j in 0..self.aritmetic_circuit.circuit[i - 1].bit_length {
                    r_v[j] = FieldElement::zero();
                }
            }
        }

        todo!()
    }

    pub fn generate_randomness(size: usize) -> Vec<FieldElement> {
        let k = size;
        let mut ret = vec![FieldElement::zero(); k];

        for i in 0..k {
            ret.push(FieldElement::new_random());
        }
        ret
    }

    pub fn direct_relay(
        &self,
        depth: usize,
        r_g: Vec<FieldElement>,
        r_u: Vec<FieldElement>,
    ) -> FieldElement {
        if depth != 1 {
            FieldElement::from_real(0)
        } else {
            let mut ret = FieldElement::from_real(1);
            for i in 0..(self.aritmetic_circuit.circuit[depth].bit_length) {
                ret = ret
                    * (FieldElement::from_real(1) - r_g[i] - r_u[i]
                        + FieldElement::from_real(2) * r_g[i] * r_u[i]);
            }
            ret
        }
    }

    pub fn delete_self() {}
}
