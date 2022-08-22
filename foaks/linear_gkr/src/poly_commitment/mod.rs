use prime_field::FieldElement;

use infrastructure::constants::*;
use infrastructure::my_hash::HashDigest;
use infrastructure::rs_polynomial::{self, fast_fourier_transform, inverse_fast_fourier_transform};
use infrastructure::utility;

static mut TWIDDLE_FACTOR: Vec<FieldElement> = Vec::new();
static mut INV_TWIDDLE_FACTOR: Vec<FieldElement> = Vec::new();
static mut TWIDDLE_FACTOR_SIZE: i64 = 0;

static mut INNER_PROD_EVALS: Vec<FieldElement> = Vec::new();

static mut L_COEF: Vec<FieldElement> = Vec::new();
static mut L_EVAL: Vec<FieldElement> = Vec::new();
static mut Q_COEF: Vec<FieldElement> = Vec::new();
static mut Q_EVAL: Vec<FieldElement> = Vec::new();

static mut LQ_COEF: Vec<FieldElement> = Vec::new();
static mut LQ_EVAL: Vec<FieldElement> = Vec::new();
static mut H_COEF: Vec<FieldElement> = Vec::new();
static mut H_EVAL: Vec<FieldElement> = Vec::new();

static mut H_EVAL_ARR: Vec<FieldElement> = Vec::new();

static mut L_COEF_LEN: usize = 0;
static mut L_EVAL_LEN: usize = 0;
static mut Q_COEF_LEN: usize = 0;
static mut Q_EVAL_LEN: usize = 0;

static mut SLICE_SIZE: usize = 0;
static mut SLICE_COUNT: usize = 0;
static mut SLICE_REAL_ELE_CNT: usize = 0;
static mut PRE_PREPARE_EXECUTED: bool = false;

struct LdtCommitment {
    commitment_hash: HashDigest,
    randomness: FieldElement,
    final_rs_code: FieldElement,
    mx_depth: usize,
    repeat_no: usize,
}

struct PolyCommitProver {
    total_time: f64,
}

impl PolyCommitProver {
    /// Safety:
    /// TODO
    ///
    /// Cannot yet determine the safety of these global static accesses
    pub unsafe fn commit_private_array(
        &mut self,
        private_array: &[FieldElement],
        log_array_length: usize,
    ) -> HashDigest {
        self.total_time = 0.;

        PRE_PREPARE_EXECUTED = true;
        SLICE_COUNT = 1 << LOG_SLICE_NUMBER;
        SLICE_SIZE = 1 << (log_array_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
        SLICE_REAL_ELE_CNT = SLICE_SIZE >> RS_CODE_RATE;

        L_EVAL_LEN = SLICE_COUNT * SLICE_SIZE;
        L_EVAL.reserve(L_EVAL_LEN);

        let mut tmp = Vec::<FieldElement>::with_capacity(SLICE_REAL_ELE_CNT);

        rs_polynomial::init_scratch_pad(SLICE_SIZE * SLICE_COUNT);

        for i in 0..SLICE_COUNT {
            let mut all_zero = true;
            let zero = FieldElement::zero();

            for j in 0..SLICE_REAL_ELE_CNT {
                if private_array[i * SLICE_REAL_ELE_CNT + j] == zero {
                    continue;
                }
                all_zero = false;
                break;
            }

            if all_zero {
                for j in 0..SLICE_SIZE {
                    L_EVAL[i * SLICE_SIZE + j] = zero;
                }
            } else {
                inverse_fast_fourier_transform(
                    &private_array[i * SLICE_REAL_ELE_CNT..],
                    SLICE_REAL_ELE_CNT,
                    SLICE_REAL_ELE_CNT,
                    FieldElement::get_root_of_unity(utility::my_log(SLICE_REAL_ELE_CNT).unwrap())
                        .unwrap(),
                    &mut tmp[..],
                );

                fast_fourier_transform(
                    &tmp[..],
                    SLICE_REAL_ELE_CNT,
                    SLICE_SIZE,
                    FieldElement::get_root_of_unity(utility::my_log(SLICE_SIZE).unwrap()).unwrap(),
                    &mut L_EVAL[i * SLICE_SIZE..],
                    &mut rs_polynomial::TWIDDLE_FACTOR[..],
                )
            }
        }

        unimplemented!()
    }
}
