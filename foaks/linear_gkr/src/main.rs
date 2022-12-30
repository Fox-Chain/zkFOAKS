use linear_gkr::{prover::zk_prover, verifier::zk_verifier};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    println!("Start Here !!!");
    let args: Vec<_> = env::args().collect();
    // if args.len() > 1 {
    //   println!("The first argument is {}", args[1]);
    //    println!("The second argument is {}", args[2]);
    //  println!("The third argument is {}", args[3]);
    //   println!("The fourth argument is {}", args[4]);
    //}

    //prime_field::init() // we dont need this line of code is it?
    let mut zk_v = zk_verifier::new();
    let mut zk_p = zk_prover::new();

    zk_p.init_total_time(0);
    zk_v.get_prover(&zk_p);

    // let circuit_file = File::open(&args[2]).unwrap();
    // println!("{:?}", circuit_file);
    // let circuit_reader = BufReader::new(circuit_file);

    // let D = circuit_reader.lines().map(|l| l.unwrap()).next();
    // let d: i8 = D.unwrap().parse().unwrap();
    // println!("{:?}", d);
    // println!("{:?}", zk_v);
    // todo: zk_v.read_circuit(argv[2], argv[3]);
    zk_v.read_circuit(&args[2], &args[3]);
}
