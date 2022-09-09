use serde::Deserialize;

use std::error::Error;

use std::io::BufReader;
use std::path::Path;
mod r1cs;
use crate::r1cs::*;
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct User {
    fingerprint: u8,
    location: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Data {
    data: Vec<User>,
}
// fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<Data> {
//     // Open the file in read-only mode with buffer.
//     let file = File::open(path)?;
//     let reader = BufReader::new(file);

//     // Read the JSON contents of the file as an instance of `User`.
//     let u = serde_json::from_reader(reader)?;

//     // Return the `User`.
//     Ok(u)
// }
fn sqr(x: u8) -> u8 {
    x + x
}
use std::{fs::File, io::Write};

use ark_test_curves::bls12_381::Fr;
fn main() {
    // let u = read_user_from_file("./src/data/test.json").unwrap();
    // let the_file = "./src/data/test.json";
    // let u: Data = serde_json::from_str(the_file).expect("JSON was not well-formatted");
    let file = File::open("./src/data/trace.json").expect("file should open read only");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("file should be proper JSON");
    let data = json.get("data").expect("file should have data key");
    println!("{:#?}", data);
    let a_in: u8 = serde_json::from_value(data["a"].clone()).unwrap();
    let b_in: u8 = serde_json::from_value(data["b"].clone()).unwrap();
    let c_in: u8 = serde_json::from_value(data["c"].clone()).unwrap();
    let mat = matrix_gen(a_in, b_in, c_in);

    let lastAccess_check = lastAccess_matrix_gen(1);
    let mut file_mat = File::create("./mat.txt").expect("error");
    let output_mat = format!("{:#?}", lastAccess_check);
    file_mat.write_all(output_mat.as_bytes());
}
// A*B=C
fn matrix_gen(a_in: u8, b_in: u8, c_in: u8) -> ConstraintMatrices<Fr> {
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
fn lastAccess_matrix_gen(x: u8) -> ConstraintMatrices<Fr> {
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
