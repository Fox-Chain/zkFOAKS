use linear_gkr::circuit_fast_track::LayeredCircuit;
use linear_gkr::{prover::ZkProver, verifier::ZkVerifier};
use prime_field::FieldElementContext;

use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();

    //Below unsafe function set packed 64-bit integers, it is mandatory?
    unsafe {
        FieldElementContext::init();
    }
    let mut zk_v = ZkVerifier::new();
    let mut zk_p = ZkProver::new();

    //let ptr_zk_p = &mut zk_p as *mut ZkProver;
    //let ptr_zk_v_arit_cir = &mut zk_v.aritmetic_circuit as *mut LayeredCircuit;

    zk_p.total_time(0.0);

    //zk_v.get_prover(ptr_zk_p);
    zk_v.get_prover(&mut zk_p);

    zk_v.read_circuit(&args[2], &args[3]);
    //zk_p.get_circuit(ptr_zk_v_arit_cir);
    zk_p.get_circuit(&mut zk_v.aritmetic_circuit);

    unsafe {
        let result = zk_v.verify_orion(&args[4]);
        println!("Pass verification? : {}", result);
    }
}
