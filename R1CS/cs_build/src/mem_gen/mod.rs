pub mod mem_gen;
pub mod stack_gen;
//pub mod r1cs_to_qap;
use crate::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, OptimizationGoal,
    Result as R1CSResult, SynthesisError, SynthesisMode,
};
use ark_ff::{Field, One, PrimeField, Zero};
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_std::test_rng;
use rand::{
    distributions::{Distribution, Standard},
    rngs::StdRng,
    Rng,
};

use ark_test_curves::bls12_381::Fr;
// use r1cs_to_qap::R1CStoQAP;
// pub fn generate_qap<QAP, F>(
//     cs: ConstraintSystemRef<F>,
//     // t: &F,
// ) -> R1CSResult<(Vec<F>, Vec<F>, Vec<F>, F, usize, usize)>
// where
//     F: PrimeField,
//     QAP: R1CStoQAP,
// {
//     type G<F> = GeneralEvaluationDomain<F>;

//     let matrices = cs.to_matrices().unwrap();
//     let rng: &mut StdRng = &mut test_rng();
//     let domain_size = cs.num_constraints() + cs.num_instance_variables();
//     let domain = G::new(domain_size).ok_or(SynthesisError::PolynomialDegreeTooLarge)?;
//     let domain_size = domain.size();
//     // let t = domain.sample_element_outside_domain(rng);
//     let t = F::rand(rng);

//     let zt = domain.evaluate_vanishing_polynomial(t.clone());

//     let u = domain.evaluate_all_lagrange_coefficients(t.clone());

//     let qap_num_variables = (cs.num_instance_variables() - 1) + cs.num_witness_variables();

//     let mut a = vec![F::zero(); qap_num_variables + 1];
//     let mut b = vec![F::zero(); qap_num_variables + 1];
//     let mut c = vec![F::zero(); qap_num_variables + 1];

//     {
//         let start = 0;
//         let end = cs.num_instance_variables();
//         let num_constraints = cs.num_constraints();
//         a[start..end].copy_from_slice(&u[(start + num_constraints)..(end + num_constraints)]);
//     }

//     for (i, u_i) in u.iter().enumerate().take(cs.num_constraints()) {
//         for &(ref coeff, index) in &matrices.a[i] {
//             a[index] += &(*u_i * coeff);
//         }
//         for &(ref coeff, index) in &matrices.b[i] {
//             b[index] += &(*u_i * coeff);
//         }
//         for &(ref coeff, index) in &matrices.c[i] {
//             c[index] += &(*u_i * coeff);
//         }
//     }

//     Ok((a, b, c, zt, qap_num_variables, domain_size))
// }
