use std::{time, usize};

use infrastructure::{
    constants::{LOG_SLICE_NUMBER, MAX_BIT_LENGTH, MAX_FRI_DEPTH, RS_CODE_RATE, SLICE_NUMBER},
    my_hash::HashDigest,
};
use poly_commitment::PolyCommitContext;
use prime_field::FieldElement;

#[derive(Default)]
pub struct CommitPhaseData {
    merkle: [Vec<HashDigest>; MAX_FRI_DEPTH],
    merkle_size: [usize; MAX_FRI_DEPTH],
    rs_codeword: [Vec<FieldElement>; MAX_FRI_DEPTH],
    poly_coef: [Vec<FieldElement>; MAX_FRI_DEPTH],
    rs_codeword_mapping: [Vec<FieldElement>; MAX_FRI_DEPTH],
}

impl CommitPhaseData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn delete_self(&mut self) {
        std::mem::take(self);
    }
}

#[derive(Debug)]
struct FieldElement64([Vec<FieldElement>; SLICE_NUMBER]);

impl Default for FieldElement64 {
    fn default() -> Self {
        const EMPTY_VEC: Vec<FieldElement> = Vec::new();
        FieldElement64([EMPTY_VEC; SLICE_NUMBER])
    }
}

// impl FieldElement64 {
//     pub fn inner(&self) -> [Vec<FieldElement>; SLICE_NUMBER] {
//         self.0
//     }
// }

#[derive(Debug)]
struct Mapping64([Vec<usize>; SLICE_NUMBER]);

impl Default for Mapping64 {
    fn default() -> Self {
        const EMPTY_VEC: Vec<usize> = Vec::new();
        Mapping64([EMPTY_VEC; SLICE_NUMBER])
    }
}

// impl Mapping64 {
//     pub fn inner(self) -> [Vec<usize>; SLICE_NUMBER] {
//         self.0
//     }
// }

#[derive(Default)]
pub struct FRIContext {
    log_current_witness_size_per_slice: usize,
    witness_bit_length_per_slice: i64,
    current_step_no: usize,
    cpd: CommitPhaseData,
    fri_timer: f64,
    witness_merkle: [Vec<HashDigest>; 2],
    witness_rs_codeword_before_arrange: [FieldElement64; 2],
    witness_rs_codeword_interleaved: [Vec<FieldElement>; 2],
    witness_rs_mapping: [Mapping64; 2],
    l_group: Vec<FieldElement>,
    visited: [Vec<bool>; MAX_BIT_LENGTH],
    visited_init: [Vec<bool>; 2],
    visited_witness: [Vec<bool>; 2],
    virtual_oracle_witness: Vec<FieldElement>,
    virtual_oracle_witness_mapping: Vec<usize>,

    r_extended: Vec<FieldElement>,
    leaf_hash: [Vec<HashDigest>; 2],
}

pub fn request_init_commit(
    FRIContext {
        log_current_witness_size_per_slice,
        witness_bit_length_per_slice,
        current_step_no,
        cpd,
        fri_timer,
        witness_merkle,
        witness_rs_codeword_before_arrange,
        witness_rs_codeword_interleaved,
        witness_rs_mapping,
        l_group,
        visited,
        visited_init,
        visited_witness,
        virtual_oracle_witness,
        virtual_oracle_witness_mapping,
        r_extended,
        leaf_hash,
    }: &mut FRIContext,
    PolyCommitContext {
        slice_size,
        slice_count,
        l_eval,
        h_eval_arr,
        ..
    }: PolyCommitContext,
    bit_len: usize,
    oracle_indicator: usize,
) -> HashDigest {
    assert_eq!(
        slice_size * slice_count,
        (1 << (bit_len + RS_CODE_RATE - LOG_SLICE_NUMBER)) * (1 << LOG_SLICE_NUMBER)
    );

    *fri_timer = 0.;

    std::mem::take(&mut witness_merkle[oracle_indicator]);
    std::mem::take(&mut witness_rs_codeword_interleaved[oracle_indicator]);

    *current_step_no = 0;

    assert_eq!(1 << LOG_SLICE_NUMBER, SLICE_NUMBER);

    *log_current_witness_size_per_slice = bit_len + RS_CODE_RATE - LOG_SLICE_NUMBER;
    *witness_bit_length_per_slice = (bit_len - LOG_SLICE_NUMBER).try_into().unwrap();

    let now = time::Instant::now();

    let sliced_input_length_per_block = 1 << *witness_bit_length_per_slice;
    assert!(*witness_bit_length_per_slice >= 0);

    let mut root_of_unity =
        FieldElement::get_root_of_unity(*log_current_witness_size_per_slice).unwrap();
    if oracle_indicator == 0 {
        l_group.reserve(1 << *log_current_witness_size_per_slice);
        l_group[0] = FieldElement::from_real(1);

        for i in 1..(1 << *log_current_witness_size_per_slice) {
            l_group[i] = l_group[i - 1] * root_of_unity;
        }
        assert_eq!(
            l_group[(1 << *log_current_witness_size_per_slice) - 1] * root_of_unity,
            FieldElement::from_real(1)
        );
    }

    witness_rs_codeword_interleaved[oracle_indicator].reserve(1 << (bit_len + RS_CODE_RATE));

    let log_leaf_size = LOG_SLICE_NUMBER + 1;
    for i in 0..SLICE_NUMBER {
        assert_eq!(
            <usize as std::convert::TryInto<i64>>::try_into(*log_current_witness_size_per_slice)
                .unwrap()
                - RS_CODE_RATE as i64,
            *witness_bit_length_per_slice
        );
        root_of_unity =
            FieldElement::get_root_of_unity((*witness_bit_length_per_slice).try_into().unwrap())
                .unwrap();

        if oracle_indicator == 0 {
            witness_rs_codeword_before_arrange[0].0[i] = l_eval[i * slice_size..].to_vec();
        } else {
            witness_rs_codeword_before_arrange[1].0[i] = h_eval_arr[i * slice_size..].to_vec();
        }

        root_of_unity =
            FieldElement::get_root_of_unity(*log_current_witness_size_per_slice).unwrap();

        witness_rs_mapping[oracle_indicator].0[i].reserve(1 << *log_current_witness_size_per_slice);

        let a = FieldElement::zero();
        for j in 0..(1 << (*log_current_witness_size_per_slice - 1)) {
            assert!((j << log_leaf_size | (i << 1) | 1) < (1 << (bit_len + RS_CODE_RATE)));
            assert!((j << log_leaf_size | (i << 1) | 1) < slice_size * slice_count);

            witness_rs_mapping[oracle_indicator].0[i][j] = j << log_leaf_size | (i << 1) | 0;
            witness_rs_mapping[oracle_indicator].0[i]
                [j + (1 << *log_current_witness_size_per_slice) / 2] =
                j << log_leaf_size | (i << 1) | 0;
        }
    }

    leaf_hash[oracle_indicator].reserve(1 << (*log_current_witness_size_per_slice - 1));
    for i in 0..(1 << (*log_current_witness_size_per_slice - 1)) {
        let tmp_hash = HashDigest::new();
        let data = [HashDigest::new(), HashDigest::new()];

        for j in 0..(1 << log_leaf_size) {}
    }

    unimplemented!()
}
