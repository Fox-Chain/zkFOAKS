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
use mem_gen::{lastAccess_matrix_gen, matrix_gen};

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
    // let mut file_mat = File::create("./mat6.txt").expect("error");
    // let output_mat = format!("{:#?}", lastAccess_check);
    // file_mat.write_all(output_mat.as_bytes());
}
