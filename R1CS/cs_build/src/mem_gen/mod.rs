use crate::lc;
use crate::r1cs::*;
use std::{fs::File, io::Write};

use ark_test_curves::bls12_381::Fr;

// A*B=C
pub fn matrix_gen(a_in: u8, b_in: u8, c_in: u8) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let a = cs.new_input_variable(|| Ok(Fr::from(a_in))).unwrap();
    println!("a={:#?}", a);
    let b = cs.new_witness_variable(|| Ok(Fr::from(b_in))).unwrap();
    println!("b={:#?}", b);
    let c = cs.new_witness_variable(|| Ok(Fr::from(c_in))).unwrap();
    println!("c={:#?}", c);
    cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)
        .unwrap();
    cs.inline_all_lcs();
    let matrices = cs.to_matrices().unwrap();
    matrices
}

// Constraint: (x)*(x-1)=0
// x∈(lastAccess,mOp,mWr)
pub fn boolean_check_matrix_gen(x_in: u64) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let x = Fr::from(x_in);
    let one = Fr::from(1u64);
    let x = cs.new_witness_variable(|| Ok(x)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
    cs.enforce_constraint(lc!() + x, lc!() + x - (one, Variable::One), lc!() + out)
        .unwrap();
    cs.finalize();
    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();

    // A [0,0,1]
    // B [-1,0,1]
    // C [0,1,0]
    assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 2)]);
    // TODO: check -1 in Fp
    //assert_eq!(matrices.b[0], vec![(Fr::one(), 0), (Fr::one(), 2)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}

// Constraint: (1-lastAccess)*(addr'-addr)=0
pub fn addr_inc_check_matrix_gen(
    x_in: u64,
    addr_p_in: u64,
    addr_in: u64,
) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let x = Fr::from(x_in);
    let addr_p = Fr::from(addr_p_in);
    let addr = Fr::from(addr_in);
    let one = Fr::from(1u64);
    let lastAccess = cs.new_witness_variable(|| Ok(x)).unwrap();
    let addr_p = cs.new_witness_variable(|| Ok(addr_p)).unwrap();
    let addr = cs.new_witness_variable(|| Ok(addr)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
    cs.enforce_constraint(
        lc!() + (one, Variable::One) - lastAccess,
        lc!() + addr_p - addr,
        lc!() + out,
    )
    .unwrap();
    cs.finalize();

    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // 1, out, lastAccess, addr', addr
    // A [1,0,-1,0,0,]
    // B [0,0,0,1,-1]
    // C [0,1,0,0,0]
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}

// Constraint: (1-mOp)*(mWr)=0
pub fn mOp_mWr_check_matrix_gen(mOp_in: u64, mWr_in: u64) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let mOp = Fr::from(mOp_in);
    let mWr = Fr::from(mWr_in);
    let one = Fr::from(1u64);
    let mOp = cs.new_witness_variable(|| Ok(mOp)).unwrap();
    let mWr = cs.new_witness_variable(|| Ok(mWr)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
    cs.enforce_constraint(lc!() + (one, Variable::One) - mOp, lc!() + mWr, lc!() + out)
        .unwrap();
    cs.finalize();

    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // 1, out, mOp, mWr
    // A [1,0,-1,0]
    // B [0,0,0,1]
    // C [0,1,0,0]
    assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 3)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}
