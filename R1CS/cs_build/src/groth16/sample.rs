// //use crate::lc;
// //use crate::r1cs::*;
// use ark_bls12_381::Bls12_381;
// use ark_bls12_381::*;
// use ark_ec::*;
// use ark_ff::fields::{FftParameters, Fp256, Fp256Parameters, FpParameters};
// // use ark_relations::r1cs::{
// //  ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, SynthesisError,
// // };
// use ark_relations::lc;
// use ark_relations::r1cs::*;
// use ark_test_curves::bls12_381::Fr;
// //pub type ConstantFr = ark_test_curves::bls12_381::Fr;
// //pub type ConstraintFr = ark_bls12_381::Fr;
// pub type ConstraintFr = Fp256<ark_bls12_381::FrParameters>;
// // ark_bls12_381::Fr is same type as Fp256<FrParameters>, which is required as the type of public variable

// // TODO: mismatch type of local r1cs module and ark_relations::r1cs module
// #[derive(Debug, Copy, Clone)]
// pub struct sampleModule {
//     a: i8,
//     b: i8,
//     c: i8,
// }

// impl ConstraintSynthesizer<ConstraintFr> for sampleModule {
//     fn generate_constraints(self, cs: ConstraintSystemRef<ConstraintFr>) -> Result<()> {
//         let cs = ConstraintSystem::<ConstraintFr>::new_ref();
//         let a = cs
//             .new_input_variable(|| Ok(ConstraintFr::from(self.a)))
//             .unwrap();
//         println!("a={:#?}", a);
//         let b = cs
//             .new_input_variable(|| Ok(ConstraintFr::from(self.b)))
//             .unwrap();
//         println!("b={:#?}", b);
//         let c = cs
//             .new_witness_variable(|| Ok(ConstraintFr::from(self.c)))
//             .unwrap();
//         println!("c={:#?}", c);
//         cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)
//             .unwrap();
//         cs.inline_all_lcs();
//         //let matrices = cs.to_matrices().unwrap();
//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::*;

//     // #[test]
//     // fn test_struct_correct() {
//     //     let sample = sampleModule { a: 2, b: 3, c: 1 };
//     //     let a = 2;
//     //     assert_eq!(sample.b, 3);
//     //     println!("here")
//     // }

//     #[test]
//     fn proof_generate_correct() {
//         use ark_groth16::Groth16;
//         use ark_snark::SNARK;
//         //use ark_test_curves::bls12_381::Fr;
//         use ark_bls12_381::Bls12_381;
//         let mut rng = ark_std::test_rng();
//         let sample_circuit = sampleModule {
//             // a: ConstraintFr::from(3i8),
//             // b: ConstraintFr::from(2i8),
//             // c: ConstraintFr::from(4i8),
//             a: 3,
//             b: 2,
//             c: 4,
//         };
//         let cs: ark_relations::r1cs::ConstraintSystemRef<ConstraintFr> =
//             ConstraintSystem::new_ref();
//         sample_circuit.generate_constraints(cs.clone()).unwrap();
//         let (pk, vk) =
//             Groth16::<Bls12_381>::circuit_specific_setup(sample_circuit, &mut rng).unwrap();
//         let public_input: [Fp256<FrParameters>; 2] = [
//             ConstraintFr::from(sample_circuit.a),
//             ConstraintFr::from(sample_circuit.b),
//         ];

//         let proof = Groth16::prove(&pk, sample_circuit, &mut rng).unwrap();
//         let valid_proof = Groth16::verify(&vk, &public_input, &proof).unwrap();
//         // assert!(valid_proof);
//     }
// }
