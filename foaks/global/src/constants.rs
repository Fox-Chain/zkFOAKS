pub const MAX_FRI_DEPTH: usize = 30;
//pub const LDT_REPEAT_NUM: usize = 33;

pub const LOG_SLICE_NUMBER: usize = 6;
pub const SLICE_NUMBER: usize = 1 << LOG_SLICE_NUMBER;
pub const RS_CODE_RATE: usize = 5;
pub const MAX_BIT_LENGTH: usize = 30;
pub const SIZE: usize = 1000000;

//pub const PACKED_SIZE: usize = 4;

pub const MAX_ORDER_FFT: usize = 28;

use prime_field::FieldElement;
pub const FE_ZERO: FieldElement = FieldElement::zero();
pub const FE_REAL_ONE: FieldElement = FieldElement::real_one();
