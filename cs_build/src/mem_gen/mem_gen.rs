use crate::lc;
use crate::r1cs::*;
use ark_ff::BigInteger256;
use ark_test_curves::bls12_381::Fr;

// Constraint: (1+mWr*mWr8)(val_src-val_dst) = 0
// mid_1 = mWr*mWr8
// out = (1+mid_1)*(val_src-val_dst)
pub fn val_check_matrix_gen(
    mWr_in: u64,
    mWr8_in: u64,
    val_src_0_in: u64,
    val_src_1_in: u64,
    val_src_2_in: u64,
    val_src_3_in: u64,
    val_dst_0_in: u64,
    val_dst_1_in: u64,
    val_dst_2_in: u64,
    val_dst_3_in: u64,
) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let mWr = Fr::from(mWr_in);
    let mWr8 = Fr::from(mWr8_in);
    let mid_1 = mWr * mWr8;
    let val_src = Fr::from(BigInteger256::new([
        val_src_0_in,
        val_src_1_in,
        val_src_2_in,
        val_src_3_in,
    ]));
    let val_dst = Fr::from(BigInteger256::new([
        val_dst_0_in,
        val_dst_1_in,
        val_dst_2_in,
        val_dst_3_in,
    ]));
    let one = Fr::from(1u64);

    let mWr = cs.new_witness_variable(|| Ok(mWr)).unwrap();
    let mWr8 = cs.new_witness_variable(|| Ok(mWr8)).unwrap();
    let mid_1 = cs.new_witness_variable(|| Ok(mid_1)).unwrap();
    let val_src = cs.new_witness_variable(|| Ok(val_src)).unwrap();
    let val_dst = cs.new_witness_variable(|| Ok(val_dst)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
    cs.enforce_constraint(lc!() + mWr, lc!() + mWr8, lc!() + mid_1)
        .unwrap();
    cs.enforce_constraint(
        lc!() + (one, Variable::One) + mid_1,
        lc!() + val_src - val_dst,
        lc!() + out,
    )
    .unwrap();
    cs.finalize();

    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // 1, out, mWr, mWr8, mid_1, val_src, val_dst
    // A [0,0,1,0,0,0,0]
    // [1,0,0,0,1,0,0]
    // B [0,0,0,1,0,0,0]
    // [0,0,0,0,0,1,-1]
    // C [0,0,0,0,1,0,0]
    // [0,1,0,0,0,0,0]
    assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 2)]);
    assert_eq!(
        matrices.a[1],
        vec![(Fr::from(1u64), 0), (Fr::from(1u64), 4)]
    );
    assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 3)]);
    assert_eq!(matrices.b[1], vec![(Fr::from(1u64), 5), (Fr::from(-1), 6)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 4)]);
    assert_eq!(matrices.c[1], vec![(Fr::from(1u64), 1)]);
    matrices
}

// Constraint: (1-mWr)mWr8 = 0
pub fn mWr_mWr8_check_matrix_gen(mWr_in: u64, mWr8_in: u64) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let mWr = Fr::from(mWr_in);
    let mWr8 = Fr::from(mWr8_in);
    let one = Fr::from(1u64);
    let mWr = cs.new_witness_variable(|| Ok(mWr)).unwrap();
    let mWr8 = cs.new_witness_variable(|| Ok(mWr8)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
    cs.enforce_constraint(
        lc!() + (one, Variable::One) - mWr,
        lc!() + mWr8,
        lc!() + out,
    )
    .unwrap();
    cs.finalize();

    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // 1, out, mOp, mWr
    // A [1,0,-1,0]
    // B [0,0,0,1]
    // C [0,1,0,0]
    assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 0), (Fr::from(-1), 2)]);
    assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 3)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}
