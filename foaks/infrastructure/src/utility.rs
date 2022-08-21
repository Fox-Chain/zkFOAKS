pub enum Log2Error {
    NotPowerOfTwo,
}

pub fn mylog(x: i64) -> Result<i32, Log2Error> {
    for i in 0..64 {
        if 1i64 << i == x {
            return Ok(i);
        }
    }
    Err(Log2Error::NotPowerOfTwo)
}

pub fn min(x: i32, y: i32) -> i32 {
    if x > y {
        y
    } else {
        x
    }
}

pub fn max(x: i32, y: i32) -> i32 {
    if x > y {
        x
    } else {
        y
    }
}
