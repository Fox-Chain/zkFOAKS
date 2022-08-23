use std::ops::DerefMut;
use std::sync::atomic::{self, AtomicBool, AtomicI64, AtomicUsize};
use std::sync::RwLock;

use prime_field::FieldElement;

use infrastructure::constants::*;
use infrastructure::my_hash::HashDigest;
use infrastructure::rs_polynomial::{self, fast_fourier_transform, inverse_fast_fourier_transform};
use infrastructure::utility;

static TWIDDLE_FACTOR: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
static INV_TWIDDLE_FACTOR: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
static TWIDDLE_FACTOR_SIZE: AtomicI64 = AtomicI64::new(0);

static INNER_PROD_EVALS: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());

static L_COEF: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
static L_EVAL: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
static Q_COEF: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
static Q_EVAL: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());

static LQ_COEF: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
static LQ_EVAL: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
static H_COEF: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
static H_EVAL: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());

static H_EVAL_ARR: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());

static L_COEF_LEN: AtomicUsize = AtomicUsize::new(0);
static L_EVAL_LEN: AtomicUsize = AtomicUsize::new(0);
static Q_COEF_LEN: AtomicUsize = AtomicUsize::new(0);
static Q_EVAL_LEN: AtomicUsize = AtomicUsize::new(0);

static SLICE_SIZE: AtomicUsize = AtomicUsize::new(0);
static SLICE_COUNT: AtomicUsize = AtomicUsize::new(0);
static SLICE_REAL_ELE_CNT: AtomicUsize = AtomicUsize::new(0);
static PRE_PREPARE_EXECUTED: AtomicBool = AtomicBool::new(false);

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
    pub fn commit_private_array(
        &mut self,
        private_array: &[FieldElement],
        log_array_length: usize,
    ) -> HashDigest {
        use atomic::Ordering::SeqCst;

        self.total_time = 0.;

        PRE_PREPARE_EXECUTED.store(true, SeqCst);

        let slice_count = 1 << LOG_SLICE_NUMBER;
        SLICE_COUNT.store(slice_count, SeqCst);

        let slice_size = 1 << (log_array_length + RS_CODE_RATE - LOG_SLICE_NUMBER);
        SLICE_SIZE.store(slice_size, SeqCst);

        let slice_real_ele_cnt = slice_size >> RS_CODE_RATE;
        SLICE_REAL_ELE_CNT.store(slice_real_ele_cnt, SeqCst);

        let l_eval_len = slice_count * slice_size;
        L_EVAL_LEN.store(l_eval_len, SeqCst);

        let mut l_eval = L_EVAL.write().unwrap();
        l_eval.reserve(l_eval_len);

        let mut tmp = Vec::<FieldElement>::with_capacity(SLICE_REAL_ELE_CNT.load(SeqCst));

        let order = slice_size * slice_count;
        rs_polynomial::init_scratch_pad(order);

        for i in 0..slice_count {
            let mut all_zero = true;
            let zero = FieldElement::zero();

            for j in 0..slice_real_ele_cnt {
                if private_array[i * slice_real_ele_cnt + j] == zero {
                    continue;
                }
                all_zero = false;
                break;
            }

            if all_zero {
                for j in 0..slice_size {
                    l_eval[i * slice_size + j] = zero;
                }
            } else {
                inverse_fast_fourier_transform(
                    &private_array[i * slice_real_ele_cnt..],
                    slice_real_ele_cnt,
                    slice_real_ele_cnt,
                    FieldElement::get_root_of_unity(utility::my_log(slice_real_ele_cnt).unwrap())
                        .unwrap(),
                    &mut tmp[..],
                );

                fast_fourier_transform(
                    &tmp[..],
                    slice_real_ele_cnt,
                    slice_size,
                    FieldElement::get_root_of_unity(utility::my_log(slice_size).unwrap()).unwrap(),
                    &mut l_eval[i * slice_size..],
                    &mut rs_polynomial::TWIDDLE_FACTOR.write().unwrap().deref_mut()[..],
                )
            }
        }

        unimplemented!()
    }
}
