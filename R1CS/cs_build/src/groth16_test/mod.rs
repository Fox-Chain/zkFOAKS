// use ark_ec::PairingEngine;
// use ark_ff::UniformRand;
// use ark_groth16::{
//     create_random_proof, generate_random_parameters, prepare_verifying_key, rerandomize_proof,
//     verify_proof,
// };
// use ark_std::test_rng;
// pub mod r1cs_to_qap;
// use r1cs_to_qap::R1CStoQAP;

// use core::ops::{Add, AddAssign, MulAssign};

// use ark_ff::{Field, Zero};
// use ark_relations::r1cs::{Result as R1CSResult, SynthesisError};
// use ark_relations::{
//     lc,
//     r1cs::{ConstraintSynthesizer, ConstraintSystemRef},
// };

// struct MySillyCircuit<F: Field> {
//     a: Option<F>,
//     b: Option<F>,
// }

// impl<ConstraintF: Field> ConstraintSynthesizer<ConstraintF> for MySillyCircuit<ConstraintF> {
//     // Generate a sample constraint: (a+b)*b=c
//     fn generate_constraints(
//         self,
//         cs: ConstraintSystemRef<ConstraintF>,
//     ) -> Result<(), SynthesisError> {
//         let a = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
//         let b = cs.new_witness_variable(|| self.b.ok_or(SynthesisError::AssignmentMissing))?;
//         let c = cs.new_input_variable(|| {
//             let mut a = self.a.ok_or(SynthesisError::AssignmentMissing)?;

//             let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;

//             a.add_assign(&b);
//             a.mul_assign(&b);
//             Ok(a)
//         })?;

//         cs.enforce_constraint(lc!() + a + b, lc!() + b, lc!() + c)?;

//         Ok(())
//     }
// }

// fn test_prove_and_verify<E>(n_iters: usize)
// where
//     E: PairingEngine,
// {
//     let rng = &mut test_rng();

//     let params =
//         generate_random_parameters::<E, _, _>(MySillyCircuit { a: None, b: None }, rng).unwrap();

//     let pvk = prepare_verifying_key::<E>(&params.vk);

//     for _ in 0..n_iters {
//         let a = E::Fr::rand(rng);
//         let b = E::Fr::rand(rng);
//         //a = a.add(b.clone());
//         let mut c = a + b;
//         c.mul_assign(&b);

//         let proof = create_random_proof(
//             MySillyCircuit {
//                 a: Some(a),
//                 b: Some(b),
//             },
//             &params,
//             rng,
//         )
//         .unwrap();
//         let result: R1CSResult<bool> = verify_proof(&pvk, &proof, &[c]);
//         println!("{:#?}", result);
//         assert!(!verify_proof(&pvk, &proof, &[a]).unwrap());
//     }
// }

// fn test_rerandomize<E>()
// where
//     E: PairingEngine,
// {
//     // First create an arbitrary Groth16 in the normal way

//     let rng = &mut test_rng();

//     let params =
//         generate_random_parameters::<E, _, _>(MySillyCircuit { a: None, b: None }, rng).unwrap();

//     let pvk = prepare_verifying_key::<E>(&params.vk);

//     let a = E::Fr::rand(rng);
//     let b = E::Fr::rand(rng);
//     // let c = a * &b;
//     let mut c = a + b;
//     c.mul_assign(&b);
//     // Create the initial proof
//     let proof1 = create_random_proof(
//         MySillyCircuit {
//             a: Some(a),
//             b: Some(b),
//         },
//         &params,
//         rng,
//     )
//     .unwrap();

//     // Rerandomize the proof, then rerandomize that
//     let proof2 = rerandomize_proof(rng, &params.vk, &proof1);
//     let proof3 = rerandomize_proof(rng, &params.vk, &proof2);

//     // Check correctness: a rerandomized proof validates when the original validates
//     assert!(verify_proof(&pvk, &proof1, &[c]).unwrap());
//     assert!(verify_proof(&pvk, &proof2, &[c]).unwrap());
//     assert!(verify_proof(&pvk, &proof3, &[c]).unwrap());

//     // Check soundness: a rerandomized proof fails to validate when the original fails to validate
//     assert!(!verify_proof(&pvk, &proof1, &[E::Fr::zero()]).unwrap());
//     assert!(!verify_proof(&pvk, &proof2, &[E::Fr::zero()]).unwrap());
//     assert!(!verify_proof(&pvk, &proof3, &[E::Fr::zero()]).unwrap());

//     // Check that the proofs are not equal as group elements
//     assert!(proof1 != proof2);
//     assert!(proof1 != proof3);
//     assert!(proof2 != proof3);
// }

// mod bls12_377 {
//     use super::{test_prove_and_verify, test_rerandomize};
//     use ark_bls12_377::Bls12_377;

//     #[test]
//     fn prove_and_verify() {
//         test_prove_and_verify::<Bls12_377>(1);
//     }

//     #[test]
//     fn rerandomize() {
//         test_rerandomize::<Bls12_377>();
//     }
// }
