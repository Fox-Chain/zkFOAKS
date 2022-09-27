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
    // one, out, x
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

// Constraint: (1-mOp'*mWr')(1-lastAccess)(val[0..7]'-val[0..7])=0
// Constraint: (1-mOp'*mWr')(1-lastAccess)(val'[0..3]-val[0..3])(val'[4..7]-val[4..7])=0
// mid_1 = mOp'*mWr'
// mid_2 = (1-mid_1)(1-lastAccess)
// mid_3 = mid_2*(val'[0..3]-val[0..3])
// out = mid_3*(val'[4..7]-val[4..7])
pub fn update_value_check_matrix_gen(
    mOp_in: u64,
    mWr_in: u64,
    lastAccess_in: u64,
    val_p0: u64,
    val_p1: u64,
    val_p2: u64,
    val_p3: u64,
    val_p4: u64,
    val_p5: u64,
    val_p6: u64,
    val_p7: u64,
    val_0: u64,
    val_1: u64,
    val_2: u64,
    val_3: u64,
    val_4: u64,
    val_5: u64,
    val_6: u64,
    val_7: u64,
) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let mOp = Fr::from(mOp_in);
    let mWr = Fr::from(mWr_in);
    let mid1 = mOp.clone() * mWr.clone();

    let lastAccess = Fr::from(lastAccess_in);
    let one = Fr::from(1u128);
    let val_p0: u128 = val_p0 as u128;
    let val_p1: u128 = val_p1 as u128;
    let val_p2: u128 = val_p2 as u128;
    let val_p3: u128 = val_p3 as u128;
    let val_p4: u128 = val_p0 as u128;
    let val_p5: u128 = val_p1 as u128;
    let val_p6: u128 = val_p2 as u128;
    let val_p7: u128 = val_p3 as u128;
    let val_0: u128 = val_0 as u128;
    let val_1: u128 = val_1 as u128;
    let val_2: u128 = val_2 as u128;
    let val_3: u128 = val_3 as u128;
    let val_4: u128 = val_0 as u128;
    let val_5: u128 = val_1 as u128;
    let val_6: u128 = val_2 as u128;
    let val_7: u128 = val_3 as u128;
    let val_p1: u128 = val_p0 << 96 + val_p1 << 64 + val_p2 << 32 + val_p3;
    let val_p1 = Fr::from(val_p1);
    let val_1: u128 = val_0 << 96 + val_1 << 64 + val_2 << 32 + val_3;
    let val_1 = Fr::from(val_1);
    let val_p2: u128 = val_p4 << 96 + val_p5 << 64 + val_p6 << 32 + val_p7;
    let val_p2 = Fr::from(val_p2);
    let val_2: u128 = val_4 << 96 + val_5 << 64 + val_6 << 32 + val_7;
    let val_2 = Fr::from(val_2);
    let mid_2 = Fr::from((one - mid1) * (one - lastAccess)); // all Fr::from is of type Fp256<FrParameters>
    let mid_3 = Fr::from(mid_2 * (val_p2 - val_2));

    let mid_1 = cs.new_witness_variable(|| Ok(mOp * mWr)).unwrap(); // all  variable is type r1cs::Variable

    let mid_2 = cs.new_witness_variable(|| Ok(mid_2)).unwrap();
    let mid_3 = cs.new_witness_variable(|| Ok(mid_3)).unwrap();
    let mOp = cs.new_witness_variable(|| Ok(mOp)).unwrap();
    let mWr = cs.new_witness_variable(|| Ok(mWr)).unwrap();
    let lastAccess = cs.new_witness_variable(|| Ok(lastAccess)).unwrap();
    let val_p1 = cs.new_witness_variable(|| Ok(val_p1)).unwrap();
    let val_1 = cs.new_witness_variable(|| Ok(val_1)).unwrap();
    let val_p2 = cs.new_witness_variable(|| Ok(val_p2)).unwrap();
    let val_2 = cs.new_witness_variable(|| Ok(val_2)).unwrap();

    let out = cs.new_input_variable(|| Ok(Fr::from(0u128))).unwrap();

    cs.enforce_constraint(lc!() + mOp, lc!() + mWr, lc!() + mid_1)
        .unwrap();
    cs.enforce_constraint(
        lc!() + (one, Variable::One) - mid_1,
        lc!() + (one, Variable::One) - lastAccess,
        lc!() + mid_2,
    )
    .unwrap();
    cs.enforce_constraint(lc!() + mid_2, lc!() + val_p1 - val_1, lc!() + mid_3)
        .unwrap();
    cs.enforce_constraint(lc!() + mid_3, lc!() + val_p2 - val_2, lc!() + out)
        .unwrap();
    cs.finalize();

    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // 1, out, mid_1, mid_2, mid_3, mOpr, mWr, lastAccess, val_p1, val_1, val_p2, val_2
    // A [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0]
    //   [1, 0, -1, 0, 0, 1, 0, 0, 0, 0, 0, 0]
    //   [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]
    //   [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]
    // B [0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0]
    //   [1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0]
    //   [0, 0, 0, 0, 0, 0, 0, 0, 1, -1, 0, 0]
    //   [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, -1]
    // C [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    //   [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]
    //   [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]
    //   [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]

    assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 5)]);
    assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 6)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 2)]);

    assert_eq!(matrices.a[2], vec![(Fr::from(1u64), 3)]);
    assert_eq!(matrices.a[3], vec![(Fr::from(1u64), 4)]);

    assert_eq!(matrices.c[1], vec![(Fr::from(1u64), 3)]);
    assert_eq!(matrices.c[2], vec![(Fr::from(1u64), 4)]);
    assert_eq!(matrices.c[3], vec![(Fr::from(1u64), 1)]);
    matrices
}

// Constraint: (1-mOp'*mWr')lastAccess(val[0..7]')=0
// Constraint: (1-mOp'*mWr')lastAccess(val'[0..3])(val'[4..7])=0
// mid_1 = mOp'*mWr'
// mid_2 = (1-mid_1)*lastAccess
// mid_3 = mid_2*(val'[0..3])
// out = mid_3*(val'[4..7])
pub fn update_value_check_mul_matrix_gen(
    mOp_in: u64,
    mWr_in: u64,
    lastAccess_in: u64,
    val_p0: u64,
    val_p1: u64,
    val_p2: u64,
    val_p3: u64,
    val_p4: u64,
    val_p5: u64,
    val_p6: u64,
    val_p7: u64,
) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let mOp = Fr::from(mOp_in);
    let mWr = Fr::from(mWr_in);
    let mid1 = mOp.clone() * mWr.clone();

    let lastAccess = Fr::from(lastAccess_in);
    let one = Fr::from(1u128);
    let val_p0: u128 = val_p0 as u128;
    let val_p1: u128 = val_p1 as u128;
    let val_p2: u128 = val_p2 as u128;
    let val_p3: u128 = val_p3 as u128;
    let val_p4: u128 = val_p0 as u128;
    let val_p5: u128 = val_p1 as u128;
    let val_p6: u128 = val_p2 as u128;
    let val_p7: u128 = val_p3 as u128;

    let val_p1: u128 = val_p0 << 96 + val_p1 << 64 + val_p2 << 32 + val_p3;
    let val_p1 = Fr::from(val_p1);
    let val_p2: u128 = val_p4 << 96 + val_p5 << 64 + val_p6 << 32 + val_p7;
    let val_p2 = Fr::from(val_p2);

    let mid_2 = Fr::from((one - mid1) * lastAccess); // all Fr::from is of type Fp256<FrParameters>
    let mid_3 = Fr::from(mid_2 * (val_p1));

    let mid_1 = cs.new_witness_variable(|| Ok(mOp * mWr)).unwrap(); // all  variable is type r1cs::Variable

    let mid_2 = cs.new_witness_variable(|| Ok(mid_2)).unwrap();
    let mid_3 = cs.new_witness_variable(|| Ok(mid_3)).unwrap();
    let mOp = cs.new_witness_variable(|| Ok(mOp)).unwrap();
    let mWr = cs.new_witness_variable(|| Ok(mWr)).unwrap();
    let lastAccess = cs.new_witness_variable(|| Ok(lastAccess)).unwrap();
    let val_p1 = cs.new_witness_variable(|| Ok(val_p1)).unwrap();
    let val_p2 = cs.new_witness_variable(|| Ok(val_p2)).unwrap();

    let out = cs.new_input_variable(|| Ok(Fr::from(0u128))).unwrap();

    cs.enforce_constraint(lc!() + mOp, lc!() + mWr, lc!() + mid_1)
        .unwrap();
    cs.enforce_constraint(
        lc!() + (one, Variable::One) - mid_1,
        lc!() + lastAccess,
        lc!() + mid_2,
    )
    .unwrap();
    cs.enforce_constraint(lc!() + mid_2, lc!() + val_p1, lc!() + mid_3)
        .unwrap();
    cs.enforce_constraint(lc!() + mid_3, lc!() + val_p2, lc!() + out)
        .unwrap();
    cs.finalize();

    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();

    // one, out, mid_1, mid_2, mid_3, mOp, mWr, lastAccess, val_p1, val_p2
    // A [0, 0, 0, 0, 0, 1, 0, 0, 0, 0]
    //   [1, 0, -1, 0, 0, 0, 0, 0, 0, 0]
    //   [0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
    //   [0, 0, 0, 0, 1, 0, 0, 0, 0, 0]
    // B [0, 0, 0, 0, 0, 0, 1, 0, 0, 0]
    //   [0, 0, 0, 0, 0, 0, 0, 1, 0, 0]
    //   [0, 0, 0, 0, 0, 0, 0, 0, 1, 0]
    //   [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
    // C [0, 0, 1, 0, 0, 0, 0, 0, 0, 0]
    //   [0, 0, 0, 1, 0, 1, 0, 0, 0, 0]
    //   [0, 0, 0, 0, 1, 0, 0, 0, 0, 0]
    //   [0, 1, 0, 0, 0, 0, 0, 0, 0, 0]
    assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 5)]);
    assert_eq!(matrices.a[2], vec![(Fr::from(1u64), 3)]);
    assert_eq!(matrices.a[3], vec![(Fr::from(1u64), 4)]);

    assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 6)]);
    assert_eq!(matrices.b[1], vec![(Fr::from(1u64), 7)]);
    assert_eq!(matrices.b[2], vec![(Fr::from(1u64), 8)]);
    assert_eq!(matrices.b[3], vec![(Fr::from(1u64), 9)]);

    assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 2)]);
    assert_eq!(
        matrices.c[1],
        vec![(Fr::from(1u64), 3), (Fr::from(1u64), 5)]
    );
    assert_eq!(matrices.c[2], vec![(Fr::from(1u64), 4)]);
    assert_eq!(matrices.c[3], vec![(Fr::from(1u64), 1)]);
}
