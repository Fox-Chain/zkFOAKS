use linear_gkr::verifier::ZkVerifier;
use prime_field::FieldElementContext;

use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();

    //Below unsafe function set packed 64-bit integers, it is mandatory?
    unsafe {
        FieldElementContext::init();
    }
    let mut zk_verifier = ZkVerifier::new();

    let bit_length = zk_verifier.read_circuit(&args[2], &args[3]);

    let result = zk_verifier.verify(&args[4], bit_length.unwrap());
    //let result = zk_verifier.virgo_verify(&args[4], bit_length);
    println!("Pass verification? : {}", result);
}
