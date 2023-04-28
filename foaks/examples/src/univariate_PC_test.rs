use std::{env, process};

use linear_code::{linear_code_encode::LinearCodeEncodeContext, parameter::COLUMN_SIZE};

enum Error {
  ParseParamsError,
}

fn main() -> Result<(), Error> {
  let args: Vec<String> = env::args().collect();
  let lg_n = match args.iter().next() {
    Some(number) => number.parse::<usize>().unwrap(),
    _ => return Err(Error::ParseParamsError),
  };
  let n = 1 << lg_n;
  let mut lce_ctx = LinearCodeEncodeContext::init();
  unsafe { lce_ctx.expander_init(n, Some(COLUMN_SIZE)) };
  Ok(())
}
