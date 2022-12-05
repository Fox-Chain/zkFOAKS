use crate::lc;
use crate::r1cs::*;
use ark_ff::BigInteger256;
use ark_test_curves::bls12_381::Fr;

// Constraint: (x)*(x-1)=0
// x∈(is_push)
pub fn stack_boolean_check(x_in: u64) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let x = Fr::from(x_in);
    let one = Fr::from(1u64);
    let x = cs.new_witness_variable(|| Ok(x)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
    cs.enforce_constraint(lc!() + x, lc!() + x - (one, Variable::One), lc!() + out)
        .unwrap();
    cs.finalize();

    //assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // one, out, x
    // A [0,0,1]
    // B [-1,0,1]
    // C [0,1,0]
    assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 2)]);
    assert_eq!(matrices.b[0], vec![(Fr::from(-1), 0), (Fr::from(1u64), 2)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}

// constraint: (is_push - 1)*push_num = 0
pub fn push_num_check(is_push: u64, push_num: u64) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let is_push = Fr::from(is_push);
    let push_num = Fr::from(push_num);
    let is_push = cs.new_witness_variable(|| Ok(is_push)).unwrap();
    let push_num = cs.new_witness_variable(|| Ok(push_num)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
    let one = Fr::from(1u64);
    cs.enforce_constraint(
        lc!() + is_push - (one, Variable::One),
        lc!() + push_num,
        lc!() + out,
    )
    .unwrap();
    cs.finalize();
    let matrices = cs.to_matrices().unwrap();

    // 1,out,is_push,push_num
    // A[-1,0,1,0]
    // B[0,0,0,1]
    // C[0,1,0,0]
    assert_eq!(matrices.a[0], vec![(Fr::from(-1), 0), (Fr::from(1u64), 2)]);
    assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 3)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}
