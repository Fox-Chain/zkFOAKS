use crate::lc;
use crate::r1cs::*;
use ark_ff::BigInteger256;
use std::{fs::File, io::Write};
// pub mod r1cs_to_qap;
// use r1cs_to_qap::*;
// adding this line will cause error in src\r1cs\constraint_system.rs:343:43
// transformed_lc.extend((lc * coeff).0.into_iter());
//                        ^^^^^^^^^^^^ cannot infer type
//use ark_groth16::*;
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

    //assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // one, out, x
    // A [0,0,1]
    // B [-1,0,1]
    // C [0,1,0]
    //assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 2)]);
    // TODO: check -1 in Fp
    //assert_eq!(matrices.b[0], vec![(Fr::one(), 0), (Fr::one(), 2)]);
    //assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    matrices
}

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

/*---------------------------circuit refer from polygon zkevm----------------------------*/

// Constraint: (1-lastAccess)*(addr'[0..3]-addr[0..3])*(addr'[4..7]-addr[4..7])=0
// mid_1 = (1-lastAccess)*(addr'[0..3]-addr[0..3])
// out = mid_1*(addr'[4..7]-addr[4..7])
pub fn addr_inc_check_matrix_gen(
    x_in: u64,
    addr_p_in_0: u64,
    addr_p_in_1: u64,
    addr_p_in_2: u64,
    addr_p_in_3: u64,
    addr_in_0: u64,
    addr_in_1: u64,
    addr_in_2: u64,
    addr_in_3: u64,
) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let x = Fr::from(x_in);

    let addr_p_in_0: u128 = addr_p_in_0 as u128;
    let addr_p_in_1: u128 = addr_p_in_1 as u128;
    let addr_p_in_2: u128 = addr_p_in_2 as u128;
    let addr_p_in_3: u128 = addr_p_in_3 as u128;
    let addr_in_0: u128 = addr_in_0 as u128;
    let addr_in_1: u128 = addr_in_1 as u128;
    let addr_in_2: u128 = addr_in_2 as u128;
    let addr_in_3: u128 = addr_in_3 as u128;

    let mut addr_p_0: u128 = addr_p_in_1 << 63 + addr_p_in_0;
    let addr_p_0 = Fr::from(addr_p_0);
    let mut addr_p_1: u128 = addr_p_in_3 << 63 + addr_p_in_2;
    let addr_p_1 = Fr::from(addr_p_1);

    let mut addr_0: u128 = addr_in_1 << 63 + addr_in_0;
    let addr_0 = Fr::from(addr_0);
    let mut addr_1: u128 = addr_in_3 << 63 + addr_in_2;
    let addr_1 = Fr::from(addr_1);

    let one = Fr::from(1u64);

    let mid_1 = Fr::from((one - x) * (addr_p_0 - addr_0));

    let lastAccess = cs.new_witness_variable(|| Ok(x)).unwrap();
    let addr_p_1 = cs.new_witness_variable(|| Ok(addr_p_1)).unwrap();
    let addr_p_0 = cs.new_witness_variable(|| Ok(addr_p_0)).unwrap();
    let addr_1 = cs.new_witness_variable(|| Ok(addr_1)).unwrap();
    let addr_0 = cs.new_witness_variable(|| Ok(addr_0)).unwrap();
    let mid_1 = cs.new_witness_variable(|| Ok(mid_1)).unwrap();

    let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
    cs.enforce_constraint(
        lc!() + (one, Variable::One) - lastAccess,
        lc!() + addr_p_0 - addr_0,
        lc!() + mid_1,
    )
    .unwrap();
    cs.enforce_constraint(lc!() + mid_1, lc!() + addr_p_1 - addr_1, lc!() + out)
        .unwrap();
    cs.finalize();
    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    matrices
}
// ) -> ConstraintMatrices<Fr> {
// // Constraint: (1-lastAccess)*(addr'-addr)=0
// pub fn addr_inc_check_matrix_gen(
//     x_in: u64,
//     addr_p_in: u64,
//     addr_in: u64,
// ) -> ConstraintMatrices<Fr> {
//     let cs = ConstraintSystem::<Fr>::new_ref();
//     let x = Fr::from(x_in);
//     let addr_p = Fr::from(addr_p_in);
//     let addr = Fr::from(addr_in);
//     let one = Fr::from(1u64);
//     let lastAccess = cs.new_witness_variable(|| Ok(x)).unwrap();
//     let addr_p = cs.new_witness_variable(|| Ok(addr_p)).unwrap();
//     let addr = cs.new_witness_variable(|| Ok(addr)).unwrap();
//     let out = cs.new_input_variable(|| Ok(Fr::from(0u64))).unwrap();
//     cs.enforce_constraint(
//         lc!() + (one, Variable::One) - lastAccess,
//         lc!() + addr_p - addr,
//         lc!() + out,
//     )
//     .unwrap();
//     cs.finalize();

//     assert!(cs.is_satisfied().is_ok());
//     let matrices = cs.to_matrices().unwrap();
//     // 1, out, lastAccess, addr', addr
//     // A [1,0,-1,0,0,]
//     // B [0,0,0,1,-1]
//     // C [0,1,0,0,0]
//     assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
//     matrices
// }

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
    //   [0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
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
    assert_eq!(matrices.c[1], vec![(Fr::from(1u64), 3)]);
    assert_eq!(matrices.c[2], vec![(Fr::from(1u64), 4)]);
    assert_eq!(matrices.c[3], vec![(Fr::from(1u64), 1)]);
    matrices
}
