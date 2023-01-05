use linear_gkr::circuit_fast_track::LayeredCircuit;
use linear_gkr::{prover::zk_prover, verifier::zk_verifier};
use std::env;

fn main() {
    println!("Start Here !!!");
    let args: Vec<_> = env::args().collect();
    { // if args.len() > 1 {
         //   println!("The first argument is {}", args[1]);
         //    println!("The second argument is {}", args[2]);
         //  println!("The third argument is {}", args[3]);
         //   println!("The fourth argument is {}", args[4]);
         //}
    }

    //prime_field::init() // we don't need this line of code is it?
    let mut zk_v = zk_verifier::new();
    let mut zk_p = zk_prover::new2();

    type Prover = zk_prover;
    type LatCir = LayeredCircuit;

    let ptr_zk_p = &mut zk_p as *mut Prover;
    let ptr_zk_v_arit_cir = &mut zk_v.aritmetic_circuit as *mut LatCir;

    zk_p.init_total_time(0);

    zk_v.get_prover(ptr_zk_p);
    zk_v.read_circuit(&args[2], &args[3]);
    zk_p.get_circuit(ptr_zk_v_arit_cir);
    unsafe {
        println!("{:?}", (*zk_p.aritmetic_circuit.unwrap()).total_depth);
        let result = zk_v.verify_orion(&args[4]);
    }
}
