use std::{env, time::Instant};
use std::fs::read_to_string;

use linear_code::parameter::COLUMN_SIZE;
use linear_pc::LinearPC;
use prime_field::FieldElement;

#[derive(Debug)]
enum Error {
  ParseParamsError,
}
#[allow(unused)]
#[feature(iter_next_chunk)]
fn main() -> Result<(), Error> {
  let args: Vec<String> = env::args().collect();
  let lg_n = match args.iter().nth(1) {
    Some(number) => number.parse::<usize>().unwrap(),
    _ => return Err(Error::ParseParamsError),
  };
  let n = 1 << lg_n;
  let mut linear_pc = LinearPC::init();
  unsafe { linear_pc.lce_ctx.expander_init(n / COLUMN_SIZE, None) };
  let mut coefs = read_array_field_element("coefs.txt");
  //let mut coefs = vec![FieldElement::zero(); n];

  // for i in 0..n {
  //   coefs[i] = FieldElement::new_random()
  // }
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

fn read_array_field_element(path: &str) -> Vec<FieldElement> {
  let result_content = read_to_string(path).unwrap();
  let result_lines = result_content.lines();

  result_lines.into_iter().map(|r| {
    let mut elements = r.split_whitespace();

    FieldElement::new(
      elements.next().unwrap().parse::<u64>().unwrap(),
      elements.next().unwrap().parse::<u64>().unwrap()
    )
  }).collect()
}
