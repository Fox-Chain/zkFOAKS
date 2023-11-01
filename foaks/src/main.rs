use linear_gkr::verifier::generate_randomness;
use linear_pc::LinearPC;
use prime_field::FieldElement;
use std::{env, time::Instant};

pub mod utils;
use utils::{parse_number, MainError};

fn main() -> Result<(), MainError> {
  let args: Vec<String> = env::args().collect();

  let lg_n = match args.get(1) {
    Some(number) => parse_number(number)?,
    None => return Err(MainError::NoNumberProvided),
  };

  let n = 1 << lg_n;

  let mut linear_pc = LinearPC::init(n); // Declara como mutable

  let coefs = generate_randomness(n);

  let commit_t0 = Instant::now();
  let h = linear_pc.commit(&coefs);
  let commit_time_diff = commit_t0.elapsed();

  let open_t0 = Instant::now();
  let multi = args
    .get(3)
    .map(|arg| arg.contains("multi"))
    .unwrap_or(false);

  let result = if multi {
    let r = generate_randomness(lg_n);
    linear_pc.open_and_verify_multi(&r, n, h)
  } else {
    linear_pc.open_and_verify(FieldElement::new_random(), n, h)
  };

  let open_time_diff = open_t0.elapsed();

  println!("Commit time: {}", commit_time_diff.as_secs_f64());
  println!("Open time: {}", open_time_diff.as_secs_f64());
  println!("{}", if result.1 { "succ" } else { "fail" });

  Ok(())
}
