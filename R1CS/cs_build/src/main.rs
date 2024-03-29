use serde::Deserialize;

use std::error::Error;

use std::io::BufReader;
use std::path::Path;
mod mem_gen;
mod r1cs;
use crate::mem_gen::*;
use crate::r1cs::*;

// fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<Data> {
//     // Open the file in read-only mode with buffer.
//     let file = File::open(path)?;
//     let reader = BufReader::new(file);

//     // Read the JSON contents of the file as an instance of `User`.
//     let u = serde_json::from_reader(reader)?;

//     // Return the `User`.
//     Ok(u)
// }

use ark_ff::PrimeField;
use ark_test_curves::bls12_381::Fr;
use std::{fs::File, io::Write};
//use mem_gen::{mem_gen, r1cs_to_qap, *};
// use crate::mem_gen::generate_qap;
use crate::mem_gen::mem_gen::*;
// use crate::mem_gen::r1cs_to_qap::{LibsnarkReduction, R1CStoQAP};
// use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn main() {
    let file = File::open("./src/data/mem_test.json").expect("file should open read only");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("file should be proper JSON");
    let data = json.get("data").expect("file should have data key");
    // println!("{:#?}",data.as_array().unwrap().len());
    let mem_table_len = data.as_array().unwrap().len();

    // Read memory op data from json
    let mOp_in = data.as_array().unwrap()[0]["mOp"].as_u64().unwrap();
    let lastAccess = data.as_array().unwrap()[0]["lastAccess"].as_u64().unwrap();
    let addr_p_0 = data.as_array().unwrap()[1]["address_0"].as_u64().unwrap();
    let addr_p_1 = data.as_array().unwrap()[1]["address_1"].as_u64().unwrap();
    let addr_p_2 = data.as_array().unwrap()[1]["address_2"].as_u64().unwrap();
    let addr_p_3 = data.as_array().unwrap()[1]["address_3"].as_u64().unwrap();
    let addr_0 = data.as_array().unwrap()[0]["address_0"].as_u64().unwrap();
    let addr_1 = data.as_array().unwrap()[0]["address_1"].as_u64().unwrap();
    let addr_2 = data.as_array().unwrap()[0]["address_2"].as_u64().unwrap();
    let addr_3 = data.as_array().unwrap()[0]["address_3"].as_u64().unwrap();
    let mWr_in = data.as_array().unwrap()[0]["mWr"].as_u64().unwrap();
    let mOp_p_in = data.as_array().unwrap()[1]["mOp"].as_u64().unwrap();
    let mWr_p_in = data.as_array().unwrap()[1]["mWr"].as_u64().unwrap();
    let val_p0 = data.as_array().unwrap()[1]["val0"].as_u64().unwrap();
    let val_p1 = data.as_array().unwrap()[1]["val1"].as_u64().unwrap();
    let val_p2 = data.as_array().unwrap()[1]["val2"].as_u64().unwrap();
    let val_p3 = data.as_array().unwrap()[1]["val3"].as_u64().unwrap();
    let val_p4 = data.as_array().unwrap()[1]["val4"].as_u64().unwrap();
    let val_p5 = data.as_array().unwrap()[1]["val5"].as_u64().unwrap();
    let val_p6 = data.as_array().unwrap()[1]["val6"].as_u64().unwrap();
    let val_p7 = data.as_array().unwrap()[1]["val7"].as_u64().unwrap();
    let val_0 = data.as_array().unwrap()[0]["val0"].as_u64().unwrap();
    let val_1 = data.as_array().unwrap()[0]["val1"].as_u64().unwrap();
    let val_2 = data.as_array().unwrap()[0]["val2"].as_u64().unwrap();
    let val_3 = data.as_array().unwrap()[0]["val3"].as_u64().unwrap();
    let val_4 = data.as_array().unwrap()[0]["val4"].as_u64().unwrap();
    let val_5 = data.as_array().unwrap()[0]["val5"].as_u64().unwrap();
    let val_6 = data.as_array().unwrap()[0]["val6"].as_u64().unwrap();
    let val_7 = data.as_array().unwrap()[0]["val7"].as_u64().unwrap();

    // check mOp/mWr/lastAccess = 1 or 0
    // let mat = boolean_check_matrix_gen(mOp_in);
    // let mat2 = boolean_check_matrix_gen(mWr_in);
    // let mat3 = boolean_check_matrix_gen(lastAccess);

    let cs_1 = boolean_check_matrix_gen(mOp_in);
    // let mut mat_1 = File::create("./mat_1_1.txt").expect("error");
    // let output_mat_1 = format!("{:#?}", cs_1);
    // mat_1.write_all(output_mat_1.as_bytes());
    // let result = generate_qap::<LibsnarkReduction, Fr>(cs_1.clone());
    // println!("Result = {:#?}", result.unwrap());

    // let witness_map =
    //     LibsnarkReduction::witness_map::<Fr, GeneralEvaluationDomain<Fr>>(cs_1.clone());
    // println!("Witness map= {:#?}", witness_map.unwrap());

    // check Constraint: (1-lastAccess)*(addr'[0..3]-addr[0..3])*(addr'[4..7]-addr[4..7])=0
    let mat4 = addr_inc_check_matrix_gen(
        lastAccess, addr_p_0, addr_p_1, addr_p_2, addr_p_3, addr_0, addr_1, addr_2, addr_3,
    );

    // check Constraint: (1-mOp)*(mWr)=0
    let mat5 = mOp_mWr_check_matrix_gen(mOp_in, mWr_in);

    // check Constraint: (1-mOp'*mWr')(1-lastAccess)(val[0..7]'-val[0..7])=0
    let mat6 = update_value_check_matrix_gen(
        mOp_p_in, mWr_p_in, lastAccess, val_p0, val_p1, val_p2, val_p3, val_p4, val_p5, val_p6,
        val_p7, val_0, val_1, val_2, val_3, val_4, val_5, val_6, val_7,
    );
    // Constraint: (1-mOp'*mWr')lastAccess(val'[0..3])(val'[4..7])=0
    let mat7 = update_value_check_mul_matrix_gen(
        mOp_p_in, mWr_p_in, lastAccess, val_p0, val_p1, val_p2, val_p3, val_p4, val_p5, val_p6,
        val_p7,
    );
}
