use serde::Deserialize;

use std::error::Error;

use std::io::BufReader;
use std::path::Path;
mod mem_gen;
mod r1cs;
use crate::mem_gen::*;
use crate::r1cs::*;

use ark_ff::PrimeField;
use ark_test_curves::bls12_381::Fr;
use std::{fs::File, io::Write};
//use mem_gen::{mem_gen, r1cs_to_qap, *};
// use crate::mem_gen::generate_qap;
use crate::mem_gen::mem_gen::*;
// use crate::mem_gen::r1cs_to_qap::{LibsnarkReduction, R1CStoQAP};
// use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn main() {}
