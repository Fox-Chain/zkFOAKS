//use core::cmp::PartialOrd;

#[derive(Debug)]

pub enum Log2Error {
    NotPowerOfTwo,
}

pub fn my_log(x: usize) -> Result<usize, Log2Error> {
    for i in 0..64 {
        if 1usize << i == x {
            return Ok(i);
        }
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
