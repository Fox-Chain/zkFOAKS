use serde::Deserialize;

use std::error::Error;

use std::io::BufReader;
use std::path::Path;
mod r1cs;
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

use std::{fs::File, io::Write};

use ark_test_curves::bls12_381::Fr;
mod mem_gen;
use mem_gen::*;
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn main() {
    let file = File::open("./src/data/mem_test.json").expect("file should open read only");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("file should be proper JSON");
    let data = json.get("data").expect("file should have data key");
    // println!(
    //     "{:#?}",
    //     data.as_array().unwrap().len()
    // );
    let mem_table_len = data.as_array().unwrap().len();

    // check mOp/mWr/lastAccess = 1 or 0
    let mOp_in = data.as_array().unwrap()[0]["mOp"].as_u64().unwrap();
    let mat = boolean_check_matrix_gen(mOp_in);

    // check Constraint: (1-lastAccess)*(addr'-addr)=0
    let lastAccess = data.as_array().unwrap()[0]["lastAccess"].as_u64().unwrap();
    let addr_p = data.as_array().unwrap()[1]["address"].as_u64().unwrap();
    let addr = data.as_array().unwrap()[0]["address"].as_u64().unwrap();
    let mat2 = addr_inc_check_matrix_gen(lastAccess, addr_p, addr);

    // check Constraint: (1-mOp)*(mWr)=0
    let mWr_in = data.as_array().unwrap()[0]["mWr"].as_u64().unwrap();
    let mat3 = mOp_mWr_check_matrix_gen(mOp_in, mWr_in);

    // check Constraint: (1-mOp'*mWr')(1-lastAccess)(val[0..7]'-val[0..7])=0
    let mOp_p_in = data.as_array().unwrap()[1]["mOp"].as_u64().unwrap();
    let mWr_p_in = data.as_array().unwrap()[1]["mWr"].as_u64().unwrap();
    let val_p0 = data.as_array().unwrap()[1]["val0"].as_u64().unwrap();
    let val_p1 = data.as_array().unwrap()[1]["val1"].as_u64().unwrap();
    let val_p2 = data.as_array().unwrap()[1]["val2"].as_u64().unwrap();
    let val_p3 = data.as_array().unwrap()[1]["val3"].as_u64().unwrap();
    let val_0 = data.as_array().unwrap()[0]["val0"].as_u64().unwrap();
    let val_1 = data.as_array().unwrap()[0]["val1"].as_u64().unwrap();
    let val_2 = data.as_array().unwrap()[0]["val2"].as_u64().unwrap();
    let val_3 = data.as_array().unwrap()[0]["val3"].as_u64().unwrap();
    let mat4 = update_value_check_matrix_gen(
        mOp_p_in, mWr_p_in, val_p0, val_p1, val_p2, val_p3, val_0, val_1, val_2, val_3,
    );
}
