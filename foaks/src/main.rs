use linear_gkr::verifier::generate_randomness;
use linear_pc::LinearPC;
use prime_field::FieldElement;
use std::{env, time::Instant};

const REQUIRED_THRESHOLD: usize = 14;

#[derive(Debug)]
enum Error {
  ParseParamsError,
}
fn main() -> Result<(), Error> {
  let args: Vec<String> = env::args().collect();
  let lg_n = match args.get(1) {
    Some(number) => match number.parse::<usize>() {
      Ok(n) => {
        if n < REQUIRED_THRESHOLD {
          eprintln!(
            "Error: The number is below the required threshold of {}.",
            REQUIRED_THRESHOLD
          );
          std::process::exit(1);
        } else {
          n
        }
      }
      Err(_) => {
        eprintln!("Error: Failed to parse the number.");
        std::process::exit(1);
      }
    },
    None => {
      eprintln!("Error: No number provided.");
      std::process::exit(1);
    }
  };

  let n = 1 << lg_n;
  let mut linear_pc = LinearPC::init(n);

  let coefs = generate_randomness(n);

  let commit_t0 = Instant::now();
  let h = linear_pc.commit(&coefs);
  let commit_time_diff = commit_t0.elapsed();
  let open_t0 = Instant::now();

  let multi = env::args().nth(3).unwrap_or_default().contains("multi");
  let result = match multi {
    true => {
      let r = generate_randomness(lg_n);
      linear_pc.open_and_verify_multi(&r, n, h)
    }
    false => linear_pc.open_and_verify(FieldElement::new_random(), n, h),
  };
  let open_time_diff = open_t0.elapsed();
  println!("Commit time: {}", commit_time_diff.as_secs_f64());
  println!("Open time: {}", open_time_diff.as_secs_f64());
  println!("{}", if result.1 { "succ" } else { "fail" });
  Ok(())
}
