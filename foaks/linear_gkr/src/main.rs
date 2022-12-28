use linear_gkr::{prover::zk_prover, verifier::zk_verifier};

pub fn main() {
    //prime_field::init() // we dont need this line of code is it?
    let mut zk_v = zk_verifier::new();
    let mut zk_p = zk_prover::new();
    zk_p.init_total_time(0);
    zk_v.get_prover(&zk_p);
    println!("{:?}", zk_v);
    // todo: zk_v.read_circuit(argv[1], argv[2]);
}
