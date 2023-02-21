use crate::lc;
use crate::r1cs::*;
use ark_ff::BigInteger256;
use ark_test_curves::bls12_381::Fr;

// Constraint: 1 * (first_prev - second after) = 0
pub fn push_element_check_matrix_gen(
    first_prev_0_in: u64,
    first_prev_1_in: u64,
    first_prev_2_in: u64,
    first_prev_3_in: u64,
    second_after_0_in: u64,
    second_after_1_in: u64,
    second_after_2_in: u64,
    second_after_3_in: u64,
) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let first_prev = Fr::from(BigInteger256::new([
        first_prev_0_in,
        first_prev_1_in,
        first_prev_2_in,
        first_prev_3_in,
    ]));
    let second_after = Fr::from(BigInteger256::new([
        second_after_0_in,
        second_after_1_in,
        second_after_2_in,
        second_after_3_in,
    ]));
    let one = Fr::from(1u64);

    let first_prev = cs.new_witness_variable(|| Ok(first_prev)).unwrap();
    let second_after = cs.new_witness_variable(|| Ok(second_after)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();

    cs.enforce_constraint(
        lc!() + (one, Variable::One),
        lc!() + first_prev - second_after,
        lc!() + out,
    )
    .unwrap();
    cs.finalize();

    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // 1, out, first_prev, second_after
    // A [1, 0, 0, 0]
    // B [0, 0, 1, -1]
    // C [0, 1, 0, 0]
    assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 0)]);
    assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 2), (Fr::from(-1), 3)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}

