//! Core interface for working with various relations that are useful in
//! zkSNARKs. At the moment, we only implement APIs for working with Rank-1
//! Constraint Systems (R1CS).

//#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    unused,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    missing_docs
)]
#![deny(unsafe_code)]

#[macro_use]
extern crate ark_std;

pub mod r1cs;

pub fn snark_proof_generation() {
    use ark_bls12_381::Bls12_381;
    use ark_groth16::Groth16;
    use ark_snark::SNARK;

    //define circuit

    // define rng

    // define pk(proving key),vk(verifying key)

    // define public input

    // define proof using Groth16::prove

    // verify proof using Groth16::verify
}
