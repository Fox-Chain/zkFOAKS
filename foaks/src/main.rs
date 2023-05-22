use std::{env, time::Instant};

use linear_code::parameter::COLUMN_SIZE;
use linear_pc::LinearPC;
use prime_field::FieldElement;

#[derive(Debug)]
enum Error {
  ParseParamsError,
}

fn main() -> Result<(), Error> {
  let args: Vec<String> = env::args().collect();
  let lg_n = match args.iter().nth(1) {
    Some(number) => number.parse::<usize>().unwrap(),
    _ => return Err(Error::ParseParamsError),
  };
  let n = 1 << lg_n;
  let mut linear_pc = LinearPC::init();
  unsafe { linear_pc.lce_ctx.expander_init(n / COLUMN_SIZE, None) };
  let mut coefs = vec![FieldElement::zero(); n];

  for i in 0..n {
    coefs[i] = FieldElement::new_random()
  }
  let commit_t0 = Instant::now();
  let h = unsafe { linear_pc.commit(coefs, n) };
  let commit_time_diff = commit_t0.elapsed();
  let open_t0 = Instant::now();
  let result = linear_pc.open_and_verify(FieldElement::new_random(), n, h);
  let open_time_diff = open_t0.elapsed();
  println!("Commit time: {}", commit_time_diff.as_secs_f64());
  println!("Open time: {}", open_time_diff.as_secs_f64());
  println!("{}", if result.1 { "succ" } else { "fail" });
  Ok(())
}
