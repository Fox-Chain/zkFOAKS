//import linear_code_encode modules
use linear_code::linear_code_encode::*;
use prime_field::FieldElement::*;

//module to measure execution time.
use std::time::Instant;

//declaration of variables
fn main() { encode_test(); }

fn encode_test() {
  prime_field::init();
  let lgN: i32 = 20;
  let N: usize = 1 << lgN;
  let lgRate: i32 = 5;
  let rs_rate: usize = 1 << lgRate;

  //initialize expansion
  expander_init(N);
  //initialize an area of ​​memory used later in the code
  init_scratch_pad(N * rs_rate * 2);
  //Vector declaration and assignment
  let random_value = prime_field::random();
  let N = 10;
  let rs_rate = 2;

  let arr: Vec<_> = (0..N).map(|_| random_value).collect();
  let dst: Vec<_> = (0..2 * N)
    .map(|_| prime_field::field_element::FieldElement::default())
    .collect();
  let rscoef: Vec<_> = (0..N)
    .map(|_| prime_field::field_element::FieldElement::default())
    .collect();
  let rsdst: Vec<_> = (0..rs_rate * N)
    .map(|_| prime_field::field_element::FieldElement::default())
    .collect();

  //Random data generation
  for i in 0..N {
    arr[i] = prime_field::random();
  }

  let t0 = Instant::now();
  let final_size = encode(&arr, &mut dst, N);
  let t1 = Instant::now();
  let duration = t1.duration_since(t0).as_secs_f64();

  // inverse fourier transform, measures time of each execution (high resolution)
  let t0 = Instant::now();
  inverse_fast_fourier_transform(&arr, N, N, prime_field::get_root_of_unity(lgN), &mut rscoef);
  fast_fourier_transform(
    &rscoef,
    N,
    rs_rate * N,
    prime_field::get_root_of_unity(lgN + lgRate),
    &mut rsdst,
  );
  let t1 = Instant::now();
  let rs_duration = t1.duration_since(t0).as_secs_f64();

  println!("Encode time {:.6}, final size {}", duration, final_size);
  println!("RS Encode time {:.6}", rs_duration);
}
