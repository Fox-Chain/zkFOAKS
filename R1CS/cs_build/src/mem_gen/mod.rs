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

// Constraint: (lastAccess)*(lastAccess-1)=0
pub fn lastAccess_matrix_gen(x: u8) -> ConstraintMatrices<Fr> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    let lastAccess = Fr::from(x);
    let one = Fr::from(1u8);
    let LA = cs.new_witness_variable(|| Ok(lastAccess)).unwrap();
    let out = cs.new_input_variable(|| Ok(Fr::from(0u8))).unwrap();
    cs.enforce_constraint(lc!() + LA, lc!() + LA - (one, Variable::One), lc!() + out)
        .unwrap();
    cs.finalize();
    // let mut file_cs = File::create("./cs.txt").expect("error");
    // let output_cs = format!("{:#?}", cs);
    // file_cs.write_all(output_cs.as_bytes());
    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    // let mut file_matrices = File::create("./matrices.txt").expect("error");
    // let output_matrices = format!("{:#?}", matrices);
    // file_matrices.write_all(output_matrices.as_bytes());

    // A [0,0,1]
    // B [-1,0,1]
    // C [0,1,0]
    assert_eq!(matrices.a[0], vec![(Fr::from(1u8), 2)]);
    // TODO: check -1 in Fp
    //assert_eq!(matrices.b[0], vec![(Fr::one(), 0), (Fr::one(), 2)]);
    assert_eq!(matrices.c[0], vec![(Fr::from(1u8), 1)]);
    assert_eq!(1, 1);
    matrices
}
