//#![feature(core_intrinsics)]

use std::borrow::Borrow;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::mem;
use std::time;
use std::time::SystemTime;

use infrastructure::constants::SLICE_NUMBER;
// use poly_commitment::PolyCommitProver;
use prime_field::FieldElement;
use prime_field::VecFieldElement;

use crate::circuit_fast_track::Gate;
use crate::circuit_fast_track::Layer;
use crate::circuit_fast_track::LayeredCircuit;
use crate::polynomial::QuadraticPoly;
use crate::prover::zk_prover;

//Todo: Debug variable

static mut Q_EVAL_REAL: Vec<FieldElement> = Vec::new();
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
    pub proof_size: usize,
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
                            //u >= 0 && u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                            u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                        );
                        assert!(
                            v > u && v <= (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                        );
                    } else {
                        //if !(u >= 0 && u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length))
                        if !(u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)) {
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
                            //u >= 0 && u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                            u < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                        );
                        //if !(v >= 0 && v < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length))
                        if !(v < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)) {
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
                            //v >= 0 && v < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
                            v < (1 << self.aritmetic_circuit.circuit[i - 1].bit_length)
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

                //
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
            //
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
        let mut verification_time = 0;
        let mut predicates_calc_time = 0;
        let mut verification_rdl_time = 0;

        //Below function is not implemented neither in virgo repo nor orion repo
        //prime_field::init_random();

        //Below function is not implemented neither in virgo repo nor orion repo
        //self.prover.unwrap().proof_init();

        // unsafe {
        let zkp = self.prover.unwrap();
        let result = (*zkp).evaluate();
        // }
        let mut alpha = FieldElement::from_real(1);
        let mut beta = FieldElement::from_real(0);
        //	random_oracle oracle; // Orion dont use this here
        let capacity =
            self.aritmetic_circuit.circuit[self.aritmetic_circuit.total_depth - 1].bit_length;
        let mut r_0 = Self::generate_randomness(capacity);
        let mut r_1 = Self::generate_randomness(capacity);
        let mut one_minus_r_0 = VecFieldElement::new(capacity);
        let mut one_minus_r_1 = VecFieldElement::new(capacity);

        for i in 0..capacity {
            one_minus_r_0
                .vec
                .push(FieldElement::from_real(1) - r_0.vec[i]);
            one_minus_r_1
                .vec
                .push(FieldElement::from_real(1) - r_1.vec[i]);
        }
        let t_a = SystemTime::now();
        println!("Calc V_output(r)");
        // unsafe{
        let mut a_0 = (*self.prover.unwrap()).V_res(
            one_minus_r_0.clone(),
            r_0.clone(),
            result,
            capacity,
            1 << capacity,
        );
        // }
        let t_b = SystemTime::now();
        let time_span = (t_b.duration_since(t_a)).unwrap();
        println!("microsecs: {}", time_span.as_micros());
        a_0 = alpha * a_0;
        let mut alpha_beta_sum = a_0;
        let direct_relay_value: FieldElement;

        for i in (self.aritmetic_circuit.total_depth - 1)..1 {
            let rho = FieldElement::new_random();

            (*self.prover.unwrap()).sumcheck_init(
                i,
                self.aritmetic_circuit.circuit[i].bit_length,
                self.aritmetic_circuit.circuit[i - 1].bit_length,
                self.aritmetic_circuit.circuit[i - 1].bit_length,
                alpha,
                beta,
                r_0.clone(),
                r_1.clone(),
                &one_minus_r_0,
                &one_minus_r_1,
            );

            (*self.prover.unwrap()).sumcheck_phase1_init();

            let mut previous_random = FieldElement::from_real(0);
            //next level random
            let r_u = Self::generate_randomness(self.aritmetic_circuit.circuit[i - 1].bit_length);
            let mut r_v =
                Self::generate_randomness(self.aritmetic_circuit.circuit[i - 1].bit_length);

            let direct_relay_value = alpha * Self::direct_relay(self, i, &r_0, &r_u)
                + beta * Self::direct_relay(self, i, &r_1, &r_u);

            if i == 1 {
                for j in 0..self.aritmetic_circuit.circuit[i - 1].bit_length {
                    r_v.vec[j] = FieldElement::zero();
                }
            }

            //V should test the maskR for two points, V does random linear combination of these points first
            let random_combine = Self::generate_randomness(1).vec[0];

            //Every time all one test to V, V needs to do a linear combination for security.
            let linear_combine = Self::generate_randomness(1).vec[0]; // mem leak

            let mut one_minus_r_u =
                VecFieldElement::new(self.aritmetic_circuit.circuit[i - 1].bit_length);
            let mut one_minus_r_v =
                VecFieldElement::new(self.aritmetic_circuit.circuit[i - 1].bit_length);

            for j in 0..(self.aritmetic_circuit.circuit[i - 1].bit_length) {
                one_minus_r_u
                    .vec
                    .push(FieldElement::from_real(1) - r_u.vec[j]);
                one_minus_r_v
                    .vec
                    .push(FieldElement::from_real(1) - r_v.vec[j]);
            }

            for j in 0..(self.aritmetic_circuit.circuit[i - 1].bit_length) {
                let poly = (*self.prover.unwrap()).sumcheck_phase1_update(previous_random, j);

                self.proof_size += mem::size_of::<QuadraticPoly>();
                previous_random = r_u.vec[j];
                //todo: Debug eval() fn
                if poly.eval(&FieldElement::zero()) + poly.eval(&FieldElement::real_one())
                    != alpha_beta_sum
                {
                    //todo: Improve error handling
                    println!(
                        "Verification fail, phase1, circuit {}, current bit {}",
                        i, j
                    );
                    //todo: return false ==  panic!() ???
                    //return false;
                    panic!()
                } else {
                    //println!(
                    //  "Verification fail, phase1, circuit {}, current bit {}",
                    //i, j
                    //);
                }
                alpha_beta_sum = poly.eval(&r_u.vec[j].clone());
            }
            //	std::cerr << "Bound v start" << std::endl;

            (*self.prover.unwrap()).sumcheck_phase2_init(
                previous_random,
                r_u.clone(),
                one_minus_r_u.clone(),
            );
            let mut previous_random = FieldElement::zero();
            for j in 0..self.aritmetic_circuit.circuit[i - 1].bit_length {
                if i == 1 {
                    r_v.vec[j] = FieldElement::zero();
                }
                let poly = (*self.prover.unwrap()).sumcheck_phase2_update(previous_random, j);
                self.proof_size += mem::size_of::<QuadraticPoly>();
                //poly.c = poly.c; ???

                previous_random = r_v.vec[j].clone();

                if poly.eval(&FieldElement::zero())
                    + poly.eval(&FieldElement::real_one())
                    + direct_relay_value * (*self.prover.unwrap()).v_u
                    != alpha_beta_sum
                {
                    //todo: Improve error handling
                    println!(
                        "Verification fail, phase2, circuit {}, current bit {}",
                        i, j
                    );
                    //todo: return false ==  panic!() ???
                    //return false;
                    panic!()
                } else {
                    //println!(
                    //  "Verification fail, phase1, circuit {}, current bit {}",
                    //i, j
                    //);
                }
                alpha_beta_sum =
                    poly.eval(&r_v.vec[j]) + direct_relay_value * (*self.prover.unwrap()).v_u;
            }
            //Add one more round for maskR
            //quadratic_poly poly p->sumcheck_finalroundR(previous_random, C.current[i - 1].bit_length);

            let final_claims = (*self.prover.unwrap()).sumcheck_finalize(previous_random);

            let v_u = final_claims.0;
            let v_v = final_claims.1;

            let predicates_calc = time::Instant::now();
            Self::beta_init(
                self,
                i,
                alpha,
                beta,
                &r_0,
                &r_1,
                &r_u,
                &r_v,
                &one_minus_r_0,
                &one_minus_r_1,
                &one_minus_r_u,
                &one_minus_r_v,
            );

            let predicates_value = Self::predicates(
                self,
                i,
                r_0.clone(),
                r_1.clone(),
                r_u.clone(),
                r_v.clone(),
                alpha,
                beta,
            );

            //todo

            let predicates_calc_span = predicates_calc.elapsed();
            if self.aritmetic_circuit.circuit[i].is_parallel == false {
                verification_rdl_time += predicates_calc_span.as_millis();
            }
            verification_time += predicates_calc_span.as_millis();
            predicates_calc_time += predicates_calc_span.as_millis();

            let mult_value = predicates_value[1];
            let add_value = predicates_value[0];
            let not_value = predicates_value[6];
            let minus_value = predicates_value[7];
            let xor_value = predicates_value[8];
            let naab_value = predicates_value[9];
            let sum_value = predicates_value[5];
            let relay_value = predicates_value[10];
            let exp_sum_value = predicates_value[12];
            let bit_test_value = predicates_value[13];
            let custom_comb_value = predicates_value[14];

            let mut r = Vec::new();
            for j in 0..self.aritmetic_circuit.circuit[i - 1].bit_length {
                r.push(r_u.vec[j].clone());
            }
            for j in 0..self.aritmetic_circuit.circuit[i - 1].bit_length {
                r.push(r_v.vec[j].clone());
            }

            if alpha_beta_sum
                != (add_value * (v_u + v_v)
                    + mult_value * v_u * v_v
                    + not_value * (FieldElement::real_one() - v_u)
                    + minus_value * (v_u - v_v)
                    + xor_value * (v_u + v_v - FieldElement::from_real(2) * v_u * v_v)
                    + naab_value * (v_v - v_u * v_v)
                    + sum_value * v_u
                    + custom_comb_value * v_u
                    + relay_value * v_u
                    + exp_sum_value * v_u
                    + bit_test_value * (FieldElement::real_one() - v_v) * v_u)
                    + direct_relay_value * v_u
            {
                //Todo: improve error handling
                println!("Verification fail, semi final, circuit level {}", i,);
                panic!();
            }
            let tmp_alpha = Self::generate_randomness(1);
            let tmp_beta = Self::generate_randomness(1);
            alpha = tmp_alpha.vec[0];
            beta = tmp_beta.vec[0];

            if (i != 1) {
                alpha_beta_sum = alpha * v_u + beta * v_v;
            } else {
                alpha_beta_sum = v_u;
            }
            r_0 = r_u;
            r_1 = r_v;
            one_minus_r_0 = one_minus_r_u;
            one_minus_r_1 = one_minus_r_v;
        }

        println!("GKR Prove Time: {}", (*self.prover.unwrap()).total_time);
        let all_sum = vec![FieldElement::zero(); SLICE_NUMBER];
        println!(
            "GKR witness size: {}",
            1 << self.aritmetic_circuit.circuit[0].bit_length
        );

        //Todo!: Implement this function in "poly_commitment" module
        //let merkle_root_l = (*self.prover.unwrap()).poly_prover.commit_private_array(
        //  (*self.prover.unwrap()).circuit_value[0],
        //  self.aritmetic_circuit.circuit[0].bit_length,
        //  );

        todo!()
    }

    pub fn generate_randomness(size: usize) -> VecFieldElement {
        let k = size;
        let mut ret = VecFieldElement::new(k);

        for i in 0..k {
            ret.vec.push(FieldElement::new_random());
        }
        ret
    }

    pub fn direct_relay(
        &mut self,
        depth: usize,
        r_g: &VecFieldElement,
        r_u: &VecFieldElement,
    ) -> FieldElement {
        if depth != 1 {
            let ret = FieldElement::from_real(0);
            return ret;
        } else {
            let mut ret = FieldElement::from_real(1);
            for i in 0..(self.aritmetic_circuit.circuit[depth].bit_length) {
                ret = ret
                    * (FieldElement::from_real(1) - r_g.vec[i] - r_u.vec[i]
                        + FieldElement::from_real(2) * r_g.vec[i] * r_u.vec[i]);
            }
            return ret;
        }
    }

    pub fn beta_init(
        &mut self,
        depth: usize,
        alpha: FieldElement,
        beta: FieldElement,
        r_0: &VecFieldElement,
        r_1: &VecFieldElement,
        r_u: &VecFieldElement,
        r_v: &VecFieldElement,
        one_minus_r_0: &VecFieldElement,
        one_minus_r_1: &VecFieldElement,
        one_minus_r_u: &VecFieldElement,
        one_minus_r_v: &VecFieldElement,
    ) {
        let debug_mode = false;
        if !self.aritmetic_circuit.circuit[depth].is_parallel || debug_mode {
            self.beta_g_r0_first_half[0] = alpha;
            self.beta_g_r1_first_half[0] = beta;
            self.beta_g_r0_second_half[0] = FieldElement::from_real(1);
            self.beta_g_r1_second_half[0] = FieldElement::from_real(1);

            let mut first_half_len = self.aritmetic_circuit.circuit[depth].bit_length / 2;
            let mut second_half_len =
                self.aritmetic_circuit.circuit[depth].bit_length - first_half_len;

            for i in 0..first_half_len {
                let r0 = r_0.vec[i];
                let r1 = r_1.vec[i];
                let or0 = one_minus_r_0.vec[i];
                let or1 = one_minus_r_1.vec[i];

                for j in 0..(1 << i) {
                    self.beta_g_r0_first_half[j | (1 << i)] = self.beta_g_r0_first_half[j] * r0;
                    self.beta_g_r1_first_half[j | (1 << i)] = self.beta_g_r1_first_half[j] * r1;
                }

                for j in 0..(1 << i) {
                    self.beta_g_r0_first_half[j] = self.beta_g_r0_first_half[j] * or0;
                    self.beta_g_r1_first_half[j] = self.beta_g_r1_first_half[j] * or1;
                }
            }

            for i in 0..second_half_len {
                let r0 = r_0.vec[i + first_half_len];
                let r1 = r_1.vec[i + first_half_len];
                let or0 = one_minus_r_0.vec[i + first_half_len];
                let or1 = one_minus_r_1.vec[i + first_half_len];

                for j in 0..(1 << i) {
                    self.beta_g_r0_second_half[j | (1 << i)] = self.beta_g_r0_second_half[j] * r0;
                    self.beta_g_r1_second_half[j | (1 << i)] = self.beta_g_r1_second_half[j] * r1;
                }

                for j in 0..(1 << i) {
                    self.beta_g_r0_second_half[j] = self.beta_g_r0_second_half[j] * or0;
                    self.beta_g_r1_second_half[j] = self.beta_g_r1_second_half[j] * or1;
                }
            }

            self.beta_u_first_half[0] = FieldElement::real_one();
            self.beta_v_first_half[0] = FieldElement::real_one();
            self.beta_u_second_half[0] = FieldElement::real_one();
            self.beta_v_second_half[0] = FieldElement::real_one();
            let first_half_len = self.aritmetic_circuit.circuit[depth - 1].bit_length / 2;
            let second_half_len =
                self.aritmetic_circuit.circuit[depth - 1].bit_length - first_half_len;

            for i in 0..first_half_len {
                let ru = r_u.vec[i];
                let rv = r_v.vec[i];
                let oru = one_minus_r_u.vec[i];
                let orv = one_minus_r_v.vec[i];

                for j in 0..(1 << i) {
                    self.beta_u_first_half[j | (1 << i)] = self.beta_u_first_half[j] * ru;
                    self.beta_v_first_half[j | (1 << i)] = self.beta_v_first_half[j] * rv;
                }

                for j in 0..(1 << i) {
                    self.beta_u_first_half[j] = self.beta_u_first_half[j] * oru;
                    self.beta_v_first_half[j] = self.beta_v_first_half[j] * orv;
                }
            }

            for i in 0..second_half_len {
                let ru = r_u.vec[i + first_half_len];
                let rv = r_v.vec[i + first_half_len];
                let oru = one_minus_r_u.vec[i + first_half_len];
                let orv = one_minus_r_v.vec[i + first_half_len];

                for j in 0..(1 << i) {
                    self.beta_u_second_half[j | (1 << i)] = self.beta_u_second_half[j] * ru;
                    self.beta_v_second_half[j | (1 << i)] = self.beta_v_second_half[j] * rv;
                }

                for j in 0..(1 << i) {
                    self.beta_u_second_half[j] = self.beta_u_second_half[j] * oru;
                    self.beta_v_second_half[j] = self.beta_v_second_half[j] * orv;
                }
            }
        }

        if self.aritmetic_circuit.circuit[depth].is_parallel {
            self.beta_g_r0_block_first_half[0] = alpha;
            self.beta_g_r1_block_first_half[0] = beta;
            self.beta_g_r0_block_second_half[0] = FieldElement::from_real(1);
            self.beta_g_r1_block_second_half[0] = FieldElement::from_real(1);

            let first_half_len = self.aritmetic_circuit.circuit[depth - 1].log_block_size / 2;
            let second_half_len =
                self.aritmetic_circuit.circuit[depth - 1].log_block_size - first_half_len;

            for i in 0..first_half_len {
                let r0 = r_0.vec[i + first_half_len];
                let r1 = r_1.vec[i + first_half_len];
                let or0 = one_minus_r_0.vec[i + first_half_len];
                let or1 = one_minus_r_1.vec[i + first_half_len];

                for j in 0..(1 << i) {
                    self.beta_g_r0_block_first_half[j | (1 << i)] =
                        self.beta_g_r0_block_first_half[j] * r0;
                    self.beta_g_r1_block_first_half[j | (1 << i)] =
                        self.beta_g_r1_block_first_half[j] * r1;
                }

                for j in 0..(1 << i) {
                    self.beta_g_r0_block_first_half[j] = self.beta_g_r0_block_first_half[j] * or0;
                    self.beta_g_r1_block_first_half[j] = self.beta_g_r1_block_first_half[j] * or1;
                }
            }

            for i in 0..second_half_len {
                let r0 = r_0.vec[i + first_half_len];
                let r1 = r_1.vec[i + first_half_len];
                let or0 = one_minus_r_0.vec[i + first_half_len];
                let or1 = one_minus_r_1.vec[i + first_half_len];

                for j in 0..(1 << i) {
                    self.beta_g_r0_block_second_half[j | (1 << i)] =
                        self.beta_g_r0_block_second_half[j] * r0;
                    self.beta_g_r1_block_second_half[j | (1 << i)] =
                        self.beta_g_r1_block_second_half[j] * r1;
                }

                for j in 0..(1 << 1) {
                    self.beta_g_r0_block_second_half[j] = self.beta_g_r0_block_second_half[j] * or0;
                    self.beta_g_r1_block_second_half[j] = self.beta_g_r1_block_second_half[j] * or1;
                }
            }

            self.beta_u_block_first_half[0] = FieldElement::real_one();
            self.beta_v_block_first_half[0] = FieldElement::real_one();
            self.beta_u_block_second_half[0] = FieldElement::real_one();
            self.beta_v_block_second_half[0] = FieldElement::real_one();
            let first_half_len = self.aritmetic_circuit.circuit[depth - 1].bit_length / 2;
            let second_half_len =
                self.aritmetic_circuit.circuit[depth - 1].bit_length - first_half_len;

            for i in 0..first_half_len {
                let ru = r_u.vec[i];
                let rv = r_v.vec[i];
                let oru = one_minus_r_u.vec[i];
                let orv = one_minus_r_v.vec[i];

                for j in 0..(1 << i) {
                    self.beta_u_block_first_half[j | (1 << i)] =
                        self.beta_u_block_first_half[j] * ru;
                    self.beta_v_block_first_half[j | (1 << i)] =
                        self.beta_v_block_first_half[j] * rv;
                }

                for j in 0..(1 << i) {
                    self.beta_u_block_first_half[j] = self.beta_u_block_first_half[j] * oru;
                    self.beta_v_block_first_half[j] = self.beta_v_block_first_half[j] * orv;
                }
            }

            for i in 0..second_half_len {
                let ru = r_u.vec[i + first_half_len];
                let rv = r_v.vec[i + first_half_len];
                let oru = one_minus_r_u.vec[i + first_half_len];
                let orv = one_minus_r_v.vec[i + first_half_len];

                for j in 0..(1 << i) {
                    self.beta_u_block_second_half[j | (1 << i)] =
                        self.beta_u_block_second_half[j] * ru;
                    self.beta_v_block_second_half[j | (1 << i)] =
                        self.beta_v_block_second_half[j] * rv;
                }

                for j in 0..(1 << i) {
                    self.beta_u_block_second_half[j] = self.beta_u_block_second_half[j] * oru;
                    self.beta_v_block_second_half[j] = self.beta_v_block_second_half[j] * orv;
                }
            }
        }
    }

    pub fn predicates(
        &mut self,
        depth: usize,
        r_0: VecFieldElement,
        r_1: VecFieldElement,
        r_u: VecFieldElement,
        r_v: VecFieldElement,
        alpha: FieldElement,
        beta: FieldElement,
    ) -> Vec<FieldElement> {
        let gate_type_count = 15;
        let mut ret_para = vec![FieldElement::zero(); gate_type_count];
        let mut ret = vec![FieldElement::zero(); gate_type_count];

        for i in 0..gate_type_count {
            ret[i] = FieldElement::zero();
            ret_para[i] = FieldElement::zero();
        }

        if depth == 1 {
            return ret;
        }

        let debug_mode = false;
        if self.aritmetic_circuit.circuit[depth].is_parallel {
            let first_half_g = self.aritmetic_circuit.circuit[depth].log_block_size / 2;
            let first_half_uv = self.aritmetic_circuit.circuit[depth - 1].log_block_size / 2;

            let mut one_block_alpha = vec![FieldElement::zero(); gate_type_count];
            let mut one_block_beta = vec![FieldElement::zero(); gate_type_count];

            for i in 0..gate_type_count {
                one_block_alpha.push(FieldElement::zero());
                one_block_beta.push(FieldElement::zero());
            }

            assert!(
                (1 << self.aritmetic_circuit.circuit[depth].log_block_size)
                    == self.aritmetic_circuit.circuit[depth].block_size
            );

            for i in 0..self.aritmetic_circuit.circuit[depth].log_block_size {
                let mut g = i;
                let mut u = self.aritmetic_circuit.circuit[depth].gates[i].u;
                let mut v = self.aritmetic_circuit.circuit[depth].gates[i].v;
                g = g & ((1 << self.aritmetic_circuit.circuit[depth].log_block_size) - 1);
                u = u & ((1 << self.aritmetic_circuit.circuit[depth - 1].log_block_size) - 1);
                v = v & ((1 << self.aritmetic_circuit.circuit[depth - 1].log_block_size) - 1);

                match self.aritmetic_circuit.circuit[depth].gates[i].ty {
                    0 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        let uv_value = (self.beta_u_block_first_half[u_first_half]
                            * self.beta_u_block_second_half[u_second_half])
                            * (self.beta_v_block_first_half[v_first_half]
                                * self.beta_v_block_second_half[v_second_half]);
                        one_block_alpha[0] = one_block_alpha[0]
                            + (self.beta_g_r0_block_first_half[g_first_half]
                                * self.beta_g_r0_block_second_half[g_second_half])
                                * uv_value;
                        one_block_beta[0] = one_block_beta[0]
                            + (self.beta_g_r1_block_first_half[g_first_half]
                                * self.beta_g_r1_block_second_half[g_second_half])
                                * uv_value;
                    }
                    1 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        let uv_value = (self.beta_u_block_first_half[u_first_half]
                            * self.beta_u_block_second_half[u_second_half])
                            * (self.beta_v_block_first_half[v_first_half]
                                * self.beta_v_block_second_half[v_second_half]);
                        one_block_alpha[1] = one_block_alpha[1]
                            + (self.beta_g_r0_block_first_half[g_first_half]
                                * self.beta_g_r0_block_second_half[g_second_half])
                                * uv_value;
                        one_block_beta[1] = one_block_beta[1]
                            + (self.beta_g_r1_block_first_half[g_first_half]
                                * self.beta_g_r1_block_second_half[g_second_half])
                                * uv_value;
                    }
                    2 => {}
                    3 => {}
                    4 => {}
                    5 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);

                        let beta_g_val_alpha = self.beta_g_r0_block_first_half[g_first_half]
                            * self.beta_g_r0_block_second_half[g_second_half];
                        let beta_g_val_beta = self.beta_g_r1_block_first_half[g_first_half]
                            * self.beta_g_r1_block_second_half[g_second_half];
                        let beta_v_0 =
                            self.beta_v_block_first_half[0] * self.beta_v_block_second_half[0];
                        for j in u..v {
                            let u_first_half = j & ((1 << first_half_uv) - 1);
                            let u_second_half = j >> first_half_uv;
                            one_block_alpha[5] = one_block_alpha[5]
                                + beta_g_val_alpha
                                    * beta_v_0
                                    * (self.beta_u_block_first_half[u_first_half]
                                        * self.beta_u_block_second_half[u_second_half]);
                            one_block_beta[5] = one_block_beta[5]
                                + beta_g_val_beta
                                    * beta_v_0
                                    * (self.beta_u_block_first_half[u_first_half]
                                        * self.beta_u_block_second_half[u_second_half]);
                        }
                    }
                    12 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);

                        let beta_g_val_alpha = self.beta_g_r0_block_first_half[g_first_half]
                            * self.beta_g_r0_block_second_half[g_second_half];
                        let beta_g_val_beta = self.beta_g_r1_block_first_half[g_first_half]
                            * self.beta_g_r1_block_second_half[g_second_half];
                        let mut beta_v_0 =
                            self.beta_v_block_first_half[0] * self.beta_v_block_second_half[0];
                        for j in u..=v {
                            let u_first_half = j & ((1 << first_half_uv) - 1);
                            let u_second_half = j >> first_half_uv;
                            one_block_alpha[12] = one_block_alpha[12]
                                + beta_g_val_alpha
                                    * beta_v_0
                                    * (self.beta_u_block_first_half[u_first_half]
                                        * self.beta_u_block_second_half[u_second_half]);
                            one_block_beta[12] = one_block_beta[12]
                                + beta_g_val_beta
                                    * beta_v_0
                                    * (self.beta_u_block_first_half[u_first_half]
                                        * self.beta_u_block_second_half[u_second_half]);

                            beta_v_0 = beta_v_0 + beta_v_0;
                        }
                    }
                    6 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = g >> first_half_g;
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        let uv_value = (self.beta_u_block_first_half[u_first_half]
                            * self.beta_u_block_second_half[u_second_half])
                            * (self.beta_v_block_first_half[v_first_half]
                                * self.beta_v_block_second_half[v_second_half]);
                        one_block_alpha[6] = one_block_alpha[6]
                            + (self.beta_g_r0_block_first_half[g_first_half]
                                * self.beta_g_r0_block_second_half[g_second_half])
                                * uv_value;
                        one_block_beta[6] = one_block_beta[6]
                            + (self.beta_g_r1_block_first_half[g_first_half]
                                * self.beta_g_r1_block_second_half[g_second_half])
                                * uv_value;
                    }
                    7 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        let uv_value = (self.beta_u_block_first_half[u_first_half]
                            * self.beta_u_block_second_half[u_second_half])
                            * (self.beta_v_block_first_half[v_first_half]
                                * self.beta_v_block_second_half[v_second_half]);
                        one_block_alpha[7] = one_block_alpha[7]
                            + (self.beta_g_r0_block_first_half[g_first_half]
                                * self.beta_g_r0_block_second_half[g_second_half])
                                * uv_value;
                        one_block_beta[7] = one_block_beta[7]
                            + (self.beta_g_r1_block_first_half[g_first_half]
                                * self.beta_g_r1_block_second_half[g_second_half])
                                * uv_value;
                    }
                    8 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        let uv_value = (self.beta_u_block_first_half[u_first_half]
                            * self.beta_u_block_second_half[u_second_half])
                            * (self.beta_v_block_first_half[v_first_half]
                                * self.beta_v_block_second_half[v_second_half]);
                        one_block_alpha[8] = one_block_alpha[8]
                            + (self.beta_g_r0_block_first_half[g_first_half]
                                * self.beta_g_r0_block_second_half[g_second_half])
                                * uv_value;
                        one_block_beta[8] = one_block_beta[8]
                            + (self.beta_g_r1_block_first_half[g_first_half]
                                * self.beta_g_r1_block_second_half[g_second_half])
                                * uv_value;
                    }
                    9 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        let uv_value = (self.beta_u_block_first_half[u_first_half]
                            * self.beta_u_block_second_half[u_second_half])
                            * (self.beta_v_block_first_half[v_first_half]
                                * self.beta_v_block_second_half[v_second_half]);
                        one_block_alpha[9] = one_block_alpha[9]
                            + (self.beta_g_r0_block_first_half[g_first_half]
                                * self.beta_g_r0_block_second_half[g_second_half])
                                * uv_value;
                        one_block_beta[9] = one_block_beta[9]
                            + (self.beta_g_r1_block_first_half[g_first_half]
                                * self.beta_g_r1_block_second_half[g_second_half])
                                * uv_value;
                    }
                    10 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        let uv_value = (self.beta_u_block_first_half[u_first_half]
                            * self.beta_u_block_second_half[u_second_half])
                            * (self.beta_v_block_first_half[v_first_half]
                                * self.beta_v_block_second_half[v_second_half]);
                        one_block_alpha[10] = one_block_alpha[10]
                            + (self.beta_g_r0_block_first_half[g_first_half]
                                * self.beta_g_r0_block_second_half[g_second_half])
                                * uv_value;
                        one_block_beta[10] = one_block_beta[10]
                            + (self.beta_g_r1_block_first_half[g_first_half]
                                * self.beta_g_r1_block_second_half[g_second_half])
                                * uv_value;
                    }
                    13 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        let uv_value = (self.beta_u_block_first_half[u_first_half]
                            * self.beta_u_block_second_half[u_second_half])
                            * (self.beta_v_block_first_half[v_first_half]
                                * self.beta_v_block_second_half[v_second_half]);
                        one_block_alpha[13] = one_block_alpha[13]
                            + (self.beta_g_r0_block_first_half[g_first_half]
                                * self.beta_g_r0_block_second_half[g_second_half])
                                * uv_value;
                        one_block_beta[13] = one_block_beta[13]
                            + (self.beta_g_r1_block_first_half[g_first_half]
                                * self.beta_g_r1_block_second_half[g_second_half])
                                * uv_value;
                    }
                    _ => {}
                }
            }
            let one = FieldElement::real_one();
            for i in 0..self.aritmetic_circuit.circuit[depth].repeat_num {
                let mut prefix_alpha = one;
                let mut prefix_beta = one;
                let mut prefix_alpha_v0 = one;
                let mut prefix_beta_v0 = one;

                for j in 0..self.aritmetic_circuit.circuit[depth].log_repeat_num {
                    if (i >> j) > 0 {
                        let uv_value = r_u.vec
                            [j + self.aritmetic_circuit.circuit[depth - 1].log_block_size]
                            * r_v.vec[j + self.aritmetic_circuit.circuit[depth - 1].log_block_size];
                        prefix_alpha = prefix_alpha
                            * r_0.vec[j + self.aritmetic_circuit.circuit[depth].log_block_size]
                            * uv_value;
                        prefix_beta = prefix_beta
                            * r_1.vec[j + self.aritmetic_circuit.circuit[depth].log_block_size]
                            * uv_value;
                    } else {
                        let uv_value = (one
                            - r_u.vec
                                [j + self.aritmetic_circuit.circuit[depth - 1].log_block_size])
                            * (one
                                - r_v.vec
                                    [j + self.aritmetic_circuit.circuit[depth - 1].log_block_size]);
                        prefix_alpha = prefix_alpha
                            * (one
                                - r_0.vec
                                    [j + self.aritmetic_circuit.circuit[depth].log_block_size])
                            * uv_value;
                        prefix_beta = prefix_beta
                            * (one
                                - r_1.vec
                                    [j + self.aritmetic_circuit.circuit[depth].log_block_size])
                            * uv_value;
                    }
                }
                for j in 0..self.aritmetic_circuit.circuit[depth].log_repeat_num {
                    if (i >> j) > 0 {
                        let uv_value = r_u.vec
                            [j + self.aritmetic_circuit.circuit[depth - 1].log_block_size]
                            * (one
                                - r_v.vec
                                    [j + self.aritmetic_circuit.circuit[depth - 1].log_block_size]);
                        prefix_alpha_v0 = prefix_alpha_v0
                            * r_0.vec[j + self.aritmetic_circuit.circuit[depth].log_block_size]
                            * uv_value;
                        prefix_beta_v0 = prefix_beta_v0
                            * r_1.vec[j + self.aritmetic_circuit.circuit[depth].log_block_size]
                            * uv_value;
                    } else {
                        let uv_value = (one
                            - r_u.vec
                                [j + self.aritmetic_circuit.circuit[depth - 1].log_block_size])
                            * (one
                                - r_v.vec
                                    [j + self.aritmetic_circuit.circuit[depth - 1].log_block_size]);
                        prefix_alpha_v0 = prefix_alpha_v0
                            * (one
                                - r_0.vec
                                    [j + self.aritmetic_circuit.circuit[depth].log_block_size])
                            * uv_value;
                        prefix_beta_v0 = prefix_beta_v0
                            * (one
                                - r_1.vec
                                    [j + self.aritmetic_circuit.circuit[depth].log_block_size])
                            * uv_value;
                    }
                }
                for j in 0..gate_type_count {
                    if (j == 6 || j == 10 || j == 5 || j == 12) {
                        ret_para[j] = ret_para[j]
                            + prefix_alpha_v0 * one_block_alpha[j]
                            + prefix_beta_v0 * one_block_beta[j];
                    } else {
                        ret_para[j] = ret_para[j]
                            + prefix_alpha * one_block_alpha[j]
                            + prefix_beta * one_block_beta[j];
                    }
                }
            }
            if (!debug_mode) {
                ret = ret_para.clone();
            }
        }
        if !self.aritmetic_circuit.circuit[depth].is_parallel || debug_mode {
            let first_half_g = self.aritmetic_circuit.circuit[depth].bit_length / 2;
            let first_half_uv = self.aritmetic_circuit.circuit[depth - 1].bit_length / 2;

            //Todo: Debug tmp_u_val
            let mut tmp_u_val = vec![
                FieldElement::zero();
                1 << self.aritmetic_circuit.circuit[depth - 1].bit_length
            ];
            let zero_v = self.beta_v_first_half[0] * self.beta_v_second_half[0];
            let mut relay_set = false;
            for i in 0..(1 << self.aritmetic_circuit.circuit[depth].bit_length) {
                let g = i;
                let u = self.aritmetic_circuit.circuit[depth].gates[i].u;
                let v = self.aritmetic_circuit.circuit[depth].gates[i].v;

                let g_first_half = g & ((1 << first_half_g) - 1);
                let g_second_half = (g >> first_half_g);
                let u_first_half = u & ((1 << first_half_uv) - 1);
                let u_second_half = u >> first_half_uv;
                let v_first_half = v & ((1 << first_half_uv) - 1);
                let v_second_half = v >> first_half_uv;

                match (self.aritmetic_circuit.circuit[depth].gates[i].ty) {
                    0 => {
                        ret[0] = ret[0]
                            + (self.beta_g_r0_first_half[g_first_half]
                                * self.beta_g_r0_second_half[g_second_half]
                                + self.beta_g_r1_first_half[g_first_half]
                                    * self.beta_g_r1_second_half[g_second_half])
                                * (self.beta_u_first_half[u_first_half]
                                    * self.beta_u_second_half[u_second_half])
                                * (self.beta_v_first_half[v_first_half]
                                    * self.beta_v_second_half[v_second_half]);
                    }
                    1 => {
                        ret[1] = ret[1]
                            + (self.beta_g_r0_first_half[g_first_half]
                                * self.beta_g_r0_second_half[g_second_half]
                                + self.beta_g_r1_first_half[g_first_half]
                                    * self.beta_g_r1_second_half[g_second_half])
                                * (self.beta_u_first_half[u_first_half]
                                    * self.beta_u_second_half[u_second_half])
                                * (self.beta_v_first_half[v_first_half]
                                    * self.beta_v_second_half[v_second_half]);
                    }
                    2 => {}
                    3 => {}
                    4 => {}
                    5 => {
                        let beta_g_val = self.beta_g_r0_first_half[g_first_half]
                            * self.beta_g_r0_second_half[g_second_half]
                            + self.beta_g_r1_first_half[g_first_half]
                                * self.beta_g_r1_second_half[g_second_half];
                        let beta_v_0 = self.beta_v_first_half[0] * self.beta_v_second_half[0];
                        for j in u..v {
                            let u_first_half = j & ((1 << first_half_uv) - 1);
                            let u_second_half = j >> first_half_uv;
                            ret[5] = ret[5]
                                + beta_g_val
                                    * beta_v_0
                                    * (self.beta_u_first_half[u_first_half]
                                        * self.beta_u_second_half[u_second_half]);
                        }
                    }
                    12 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);

                        let beta_g_val = self.beta_g_r0_first_half[g_first_half]
                            * self.beta_g_r0_second_half[g_second_half]
                            + self.beta_g_r1_first_half[g_first_half]
                                * self.beta_g_r1_second_half[g_second_half];
                        let mut beta_v_0 = self.beta_v_first_half[0] * self.beta_v_second_half[0];
                        for j in u..=v {
                            let u_first_half = j & ((1 << first_half_uv) - 1);
                            let u_second_half = j >> first_half_uv;
                            ret[12] = ret[12]
                                + beta_g_val
                                    * beta_v_0
                                    * (self.beta_u_first_half[u_first_half]
                                        * self.beta_u_second_half[u_second_half]);
                            beta_v_0 = beta_v_0 + beta_v_0;
                        }
                    }
                    14 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);

                        let beta_g_val = self.beta_g_r0_first_half[g_first_half]
                            * self.beta_g_r0_second_half[g_second_half]
                            + self.beta_g_r1_first_half[g_first_half]
                                * self.beta_g_r1_second_half[g_second_half];
                        let beta_v_0 = self.beta_v_first_half[0] * self.beta_v_second_half[0];
                        for j in 0..self.aritmetic_circuit.circuit[depth].gates[i].parameter_length
                        {
                            let src = self.aritmetic_circuit.circuit[depth].gates[i].src[j];
                            let u_first_half = src & ((1 << first_half_uv) - 1);
                            let u_second_half = src >> first_half_uv;
                            let weight = self.aritmetic_circuit.circuit[depth].gates[i].weight[j];
                            ret[14] = ret[14]
                                + beta_g_val
                                    * beta_v_0
                                    * (self.beta_u_first_half[u_first_half]
                                        * self.beta_u_second_half[u_second_half])
                                    * weight;
                        }
                    }
                    6 => {
                        ret[6] = ret[6]
                            + (self.beta_g_r0_first_half[g_first_half]
                                * self.beta_g_r0_second_half[g_second_half]
                                + self.beta_g_r1_first_half[g_first_half]
                                    * self.beta_g_r1_second_half[g_second_half])
                                * (self.beta_u_first_half[u_first_half]
                                    * self.beta_u_second_half[u_second_half])
                                * (self.beta_v_first_half[v_first_half]
                                    * self.beta_v_second_half[v_second_half]);
                    }
                    7 => {
                        ret[7] = ret[7]
                            + (self.beta_g_r0_first_half[g_first_half]
                                * self.beta_g_r0_second_half[g_second_half]
                                + self.beta_g_r1_first_half[g_first_half]
                                    * self.beta_g_r1_second_half[g_second_half])
                                * (self.beta_u_first_half[u_first_half]
                                    * self.beta_u_second_half[u_second_half])
                                * (self.beta_v_first_half[v_first_half]
                                    * self.beta_v_second_half[v_second_half]);
                    }
                    8 => {
                        ret[8] = ret[8]
                            + (self.beta_g_r0_first_half[g_first_half]
                                * self.beta_g_r0_second_half[g_second_half]
                                + self.beta_g_r1_first_half[g_first_half]
                                    * self.beta_g_r1_second_half[g_second_half])
                                * (self.beta_u_first_half[u_first_half]
                                    * self.beta_u_second_half[u_second_half])
                                * (self.beta_v_first_half[v_first_half]
                                    * self.beta_v_second_half[v_second_half]);
                    }
                    9 => {
                        ret[9] = ret[9]
                            + (self.beta_g_r0_first_half[g_first_half]
                                * self.beta_g_r0_second_half[g_second_half]
                                + self.beta_g_r1_first_half[g_first_half]
                                    * self.beta_g_r1_second_half[g_second_half])
                                * (self.beta_u_first_half[u_first_half]
                                    * self.beta_u_second_half[u_second_half])
                                * (self.beta_v_first_half[v_first_half]
                                    * self.beta_v_second_half[v_second_half]);
                    }
                    10 => {
                        if (relay_set == false) {
                            tmp_u_val = vec![
                                FieldElement::zero();
                                1 << self.aritmetic_circuit.circuit[depth - 1]
                                    .bit_length
                            ];

                            for i in 0..(1 << self.aritmetic_circuit.circuit[depth - 1].bit_length)
                            {
                                let u_first_half = i & ((1 << first_half_uv) - 1);
                                let u_second_half = i >> first_half_uv;
                                tmp_u_val[i] = self.beta_u_first_half[u_first_half]
                                    * self.beta_u_second_half[u_second_half];
                            }

                            relay_set = true;
                        }
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        ret[10] = ret[10]
                            + (self.beta_g_r0_first_half[g_first_half]
                                * self.beta_g_r0_second_half[g_second_half]
                                + self.beta_g_r1_first_half[g_first_half]
                                    * self.beta_g_r1_second_half[g_second_half])
                                * tmp_u_val[u];
                    }
                    13 => {
                        let g_first_half = g & ((1 << first_half_g) - 1);
                        let g_second_half = (g >> first_half_g);
                        let u_first_half = u & ((1 << first_half_uv) - 1);
                        let u_second_half = u >> first_half_uv;
                        let v_first_half = v & ((1 << first_half_uv) - 1);
                        let v_second_half = v >> first_half_uv;
                        ret[13] = ret[13]
                            + (self.beta_g_r0_first_half[g_first_half]
                                * self.beta_g_r0_second_half[g_second_half]
                                + self.beta_g_r1_first_half[g_first_half]
                                    * self.beta_g_r1_second_half[g_second_half])
                                * (self.beta_u_first_half[u_first_half]
                                    * self.beta_u_second_half[u_second_half])
                                * (self.beta_v_first_half[v_first_half]
                                    * self.beta_v_second_half[v_second_half]);
                    }
                    _ => {}
                }
            }
            ret[10] = ret[10] * zero_v;
        }
        for i in 0..gate_type_count {
            if self.aritmetic_circuit.circuit[depth].is_parallel {
                assert!(ret[i] == ret_para[i]);
            }
        }
        ret
    }

    pub fn delete_self() {}
}
