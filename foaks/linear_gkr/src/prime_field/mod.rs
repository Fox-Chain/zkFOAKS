pub mod error;
pub mod ops;

use std::sync::atomic::AtomicBool;

use ethnum::{i256, AsI256};
use serde::Serialize;

use self::error::PrimeFieldError;

pub const MOD: u64 = 2305843009213693951;

pub static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

pub const MASK: u32 = 4294967295; // 2^32 - 1
pub const PRIME: u64 = 2305843009213693951; // 2^61 - 1

pub fn my_mod(x: u64) -> u64 {
    (x >> 61) + (x & MOD)
}

pub fn my_mult(x: u64, y: u64) -> u64 {
    // return a value between [0, 2PRIME) = x * y mod PRIME
    // return ((hi << 3) | (lo >> 61)) + (lo & PRIME)
    let (lo, hi) = x.widening_mul(y);
    ((hi << 3) | (lo >> 61)) + (lo & PRIME)
}

mod intrinsics {
    pub mod i256 {
        use ethnum::{i256, u256};

        pub fn srl(x: &i256, c: u32) -> i256 {
            u256::as_i256(x.as_u256() >> c)
        }

        pub fn sll(x: &i256, c: u32) -> i256 {
            u256::as_i256(x.as_u256() << c)
        }
    }
}

pub fn packed_my_mult(x: i256, y: i256) -> i256 {
    let x_shift = intrinsics::i256::srl(&x, 32);
    let y_shift = intrinsics::i256::srl(&y, 32);

    let ac = x_shift * y_shift;
    let ad = x_shift * y;
    let bc = x * y_shift;
    let bd = x * y;

    let ad_bc = ad + bc;
    let bd_srl32 = intrinsics::i256::srl(&bd, 32);
    let ad_bc_srl32 = intrinsics::i256::srl(&(ad_bc + bd_srl32), 32);
    let ad_bc_sll32 = intrinsics::i256::sll(&ad_bc, 32);

    let hi = ac + ad_bc_srl32;
    let lo = bc + ad_bc_sll32;

    // ((hi << 3) | (lo >> 61)) + (lo & PRIME)
    (intrinsics::i256::sll(&hi, 3) | intrinsics::i256::srl(&lo, 61)) + (lo & PRIME.as_i256())
}

pub fn packed_my_mod(x: i256) -> i256 {
    // (x >> 61) + (x & mod)
    intrinsics::i256::srl(&x, 61) + (x & MOD.as_i256())
}

#[derive(Serialize, Default, PartialEq, Eq)]
pub struct FieldElement {
    pub real: u64,
    pub img: u64,
}

impl FieldElement {
    fn to_owned_bytes(&self) -> Result<Vec<u8>, PrimeFieldError> {
        Ok(bincode::serialize(self)?)
    }

    pub fn bit_stream(&self) -> Result<Vec<u8>, PrimeFieldError> {
        self.to_owned_bytes()
    }

    pub fn from_real(real: u64) -> Self {
        let real = real % MOD;
        Self { img: 0, real }
    }

    pub fn from_img(img: u64) -> Self {
        let img = img % MOD;
        Self { img, real: 0 }
    }

    pub fn new(real: u64, img: u64) -> Self {
        Self { img, real }
    }

    pub fn sum_parts(&self) -> u64 {
        self.real + self.img
    }
}

fn verify_lt_mod_once(mut a: u64) -> u64 {
    if a >= MOD {
        a -= MOD;
    }
    a
}

fn verify_lt_mod_many(mut a: u64) -> u64 {
    while a >= MOD {
        a -= MOD;
    }
    a
}
