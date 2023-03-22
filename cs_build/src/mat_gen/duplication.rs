use crate::lc;
use crate::r1cs::*;
use ark_ff::BigInteger256;
use ark_test_curves::bls12_381::Fr;

// Constraint: 1 * (len_after - len_before - 1) = 0
pub fn check_dup_stack_len(
    len_after: u16,
    len_before: u16,
) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let len_after = Fr::from(len_after);
    let len_before = Fr::from(len_before);
    let one = Fr::from(1u64);

    let len_after = cs.new_witness_variable(|| Ok(len_after)).unwrap();
    let len_before = cs.new_witness_variable(|| Ok(len_before)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u16))).unwrap();

    cs.enforce_constraint(
        lc!() + (one, Variable::One),
        lc!() + len_after - len_before - (one, Variable::One),
        lc!() + out,
    )
    .unwrap();
    cs.finalize();

    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // 1, out, len_after, len_before
    // A [1,0,0,0]
    // B [-1,0,1,-1]
    // C [0,1,0,0]
    assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 0)]);
    assert_eq!(matrices.b[0], vec![(Fr::from(-1), 0), (Fr::from(1u64), 2), (Fr::from(-1), 3)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}