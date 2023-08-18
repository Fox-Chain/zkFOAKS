#[derive(Debug)]
pub enum Log2Error {
  NotPowerOfTwo,
}

pub fn my_log(x: usize) -> Result<usize, Log2Error> {
  if x != 0 && (x & (x - 1)) == 0 {
    let exponent = x.trailing_zeros() as usize;
    return Ok(exponent);
  }
  Err(Log2Error::NotPowerOfTwo)
}

pub fn min<T: PartialOrd>(x: T, y: T) -> T {
  if x > y {
    y
  } else {
    x
  }
}

pub fn max<T: PartialOrd>(x: T, y: T) -> T {
  if x > y {
    x
  } else {
    y
  }
}
