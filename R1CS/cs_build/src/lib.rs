// //! Core interface for working with various relations that are useful in
// //! zkSNARKs. At the moment, we only implement APIs for working with Rank-1
// //! Constraint Systems (R1CS).

// //#![cfg_attr(not(feature = "std"), no_std)]
// #![warn(
//     unused,
//     future_incompatible,
//     nonstandard_style,
//     rust_2018_idioms,
//     missing_docs
// )]
// #![deny(unsafe_code)]

// #[macro_use]
// extern crate ark_std;

pub mod groth16;
// pub mod r1cs;

// // impl ConstraintSysthesizer<F> for r1cs{
// //     fn generate_constraints(self,
// //         cs: ConstraintSystemRef<ConstraintF>,
// //     )->Result<(),SynthesisError>{
// //         Ok(())
// //     }
// // }

// // pub fn snark_proof_generation() {
// //     use ark_bls12_381::Bls12_381;
// //     use ark_groth16::Groth16;
// //     use ark_snark::SNARK;

// //     //define circuit
// //     let circuit_defining_cs ->ConstraintSynthesizer<F> = generate_circuit();
// //     // define rng
// //     let mut rng = ark_std::test_rng();

// //     // define pk(proving key),vk(verifying key)
// //     let (pk, vk) =
// //             Groth16::<Bls12_381>::circuit_specific_setup(circuit_defining_cs, &mut rng).unwrap();

// //     // define public input
// //     let circuit_to_verify_against = circuit_defining_cs;
// //     let public_input = [
// //         circuit_to_verify_against.initial_root.unwrap(),
// //         circuit_to_verify_against.final_root.unwrap(),
// //     ];
// //     // define proof using Groth16::prove
// //     let proof = Groth16::prove(&pk, circuit_to_verify_against, &mut rng).unwrap();

// //     // verify proof using Groth16::verify
// //     let valid_proof = Groth16::verify(&vk, &public_input, &proof).unwrap();
// //     assert!(valid_proof);
// // }
