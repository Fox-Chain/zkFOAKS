use std::mem;
use std::{time, usize};

use infrastructure::{
    constants::{LOG_SLICE_NUMBER, MAX_BIT_LENGTH, MAX_FRI_DEPTH, RS_CODE_RATE, SLICE_NUMBER},
    merkle_tree,
    my_hash::HashDigest,
};
use poly_commitment::poly_commitment::PolyCommitContext;
use prime_field::FieldElement;

#[derive(Default)]
pub struct CommitPhaseData {
    pub merkle: [Vec<HashDigest>; MAX_FRI_DEPTH],
    pub merkle_size: [usize; MAX_FRI_DEPTH],
    pub rs_codeword: [Vec<FieldElement>; MAX_FRI_DEPTH],
    pub poly_coef: [Vec<FieldElement>; MAX_FRI_DEPTH],
    pub rs_codeword_mapping: [Vec<usize>; MAX_FRI_DEPTH],
}

// namespace fri
impl CommitPhaseData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn delete_self(&mut self) {
        std::mem::take(self);
    }
}

#[derive(Debug)]
pub struct FieldElement64([Vec<FieldElement>; SLICE_NUMBER]);

impl Default for FieldElement64 {
    fn default() -> Self {
        const EMPTY_VEC: Vec<FieldElement> = Vec::new();
        FieldElement64([EMPTY_VEC; SLICE_NUMBER])
    }
}

#[derive(Debug)]
pub struct Mapping64([Vec<usize>; SLICE_NUMBER]);

impl Default for Mapping64 {
    fn default() -> Self {
        const EMPTY_VEC: Vec<usize> = Vec::new();
        Mapping64([EMPTY_VEC; SLICE_NUMBER])
    }
}

#[derive(Default)]
pub struct FRIContext {
    pub log_current_witness_size_per_slice: usize,
    pub witness_bit_length_per_slice: i64,
    pub current_step_no: usize,
    pub cpd: CommitPhaseData,
    pub fri_timer: f64,
    pub witness_merkle: [Vec<HashDigest>; 2],
    pub witness_rs_codeword_before_arrange: [FieldElement64; 2],
    pub witness_rs_codeword_interleaved: [Vec<FieldElement>; 2],
    pub witness_rs_mapping: [Mapping64; 2],
    pub l_group: Vec<FieldElement>,
    pub visited: [Vec<bool>; MAX_BIT_LENGTH],
    pub visited_init: [Vec<bool>; 2],
    pub visited_witness: [Vec<bool>; 2],
    pub virtual_oracle_witness: Vec<FieldElement>,
    pub virtual_oracle_witness_mapping: Vec<usize>,

    pub r_extended: Vec<FieldElement>,
    pub leaf_hash: [Vec<HashDigest>; 2],
}

/// Given private input, calculate the first oracle commitment
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
        mut l_eval,
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

    let _now = time::Instant::now();

    let _sliced_input_length_per_block = 1 << *witness_bit_length_per_slice;
    assert!(*witness_bit_length_per_slice >= 0);

    let mut root_of_unity =
        FieldElement::get_root_of_unity(*log_current_witness_size_per_slice).unwrap();
    if oracle_indicator == 0 {
        l_group.reserve(1 << *log_current_witness_size_per_slice);
        // println!("l_group: {:?}", root_of_unity);
        l_group.push(FieldElement::from_real(1));
        // l_group[0] = FieldElement::from_real(1);

        for i in 1..(1 << *log_current_witness_size_per_slice) {
            l_group.push(l_group[i - 1] * root_of_unity);
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

        // TODO
        if oracle_indicator == 0 {
            // println!("{:?}", i);

            // println!("{:?}", l_eval);
            l_eval.reserve(i * slice_size);
            witness_rs_codeword_before_arrange[0].0[i]
                .reserve(1 << *log_current_witness_size_per_slice);
            witness_rs_codeword_before_arrange[0].0[i] = l_eval[i * slice_size..].to_vec();
            // println!("{:?}", witness_rs_codeword_before_arrange[0].0);
        }
        // else {
        //     // commit_public_array error here
        //     witness_rs_codeword_before_arrange[1].0[i]
        //         .reserve(1 << *log_current_witness_size_per_slice);
        //     witness_rs_codeword_before_arrange[1].0[i] = h_eval_arr[i * slice_size..].to_vec();
        // }

        root_of_unity =
            FieldElement::get_root_of_unity(*log_current_witness_size_per_slice).unwrap();

        witness_rs_mapping[oracle_indicator].0[i].reserve(1 << *log_current_witness_size_per_slice);
        witness_rs_mapping[oracle_indicator].0[i].reserve(
            ((1 << *log_current_witness_size_per_slice) / 2)
                + (1 << (*log_current_witness_size_per_slice)),
        );
        let _a = FieldElement::zero();

        for j in 0..(1 << (*log_current_witness_size_per_slice - 1)) {
            assert!((j << log_leaf_size | (i << 1) | 1) < (1 << (bit_len + RS_CODE_RATE)));
            assert!((j << log_leaf_size | (i << 1) | 1) < slice_size * slice_count);
            // Fix bug here
            // println!("Here: {:?}", j);
            witness_rs_mapping[oracle_indicator].0[i].push(j << log_leaf_size | (i << 1) | 0);
            // witness_rs_mapping[oracle_indicator].0[i]
            //     [j + (1 << *log_current_witness_size_per_slice) / 2] =
            //     j << log_leaf_size | (i << 1) | 0;
        }
        // println!("{:?}", 1 << (*log_current_witness_size_per_slice - 1));
        // println!("{:?}", witness_rs_mapping);

        for j in 0..(1 << (*log_current_witness_size_per_slice - 1)) {
            witness_rs_mapping[oracle_indicator].0[i].push(j << log_leaf_size | (i << 1) | 0);
        }

        // println!("{:?}", witness_rs_mapping[0].0[i]);
    }

    leaf_hash[oracle_indicator].reserve(1 << (*log_current_witness_size_per_slice - 1));
    for _i in 0..(1 << (*log_current_witness_size_per_slice - 1)) {
        let _tmp_hash = HashDigest::new();
        let _data = [HashDigest::new(), HashDigest::new()];

        for _j in 0..(1 << log_leaf_size) {}
    }

    unsafe {
        merkle_tree::create_tree(
            leaf_hash[oracle_indicator].clone(),
            1 << (*log_current_witness_size_per_slice - 1),
            &mut witness_merkle[oracle_indicator],
            Some(mem::size_of::<HashDigest>()),
            Some(true),
        );
    };
    // TODO
    // witness_merkle_size[oracle_indicator] = 1 << (log_current_witness_size_per_slice - 1);
    // visited_init[oracle_indicator] = new bool[1 << (log_current_witness_size_per_slice)];
    // visited_witness[oracle_indicator] = new bool[1 << (bit_len + rs_code_rate)];
    // memset(visited_init[oracle_indicator], false, sizeof(bool) * (1 << (log_current_witness_size_per_slice)));
    // memset(visited_witness[oracle_indicator], false, sizeof(bool) * (1 << (bit_len + rs_code_rate)));
    // auto t1 = std::chrono::high_resolution_clock::now();
    // auto time_span = std::chrono::duration_cast<std::chrono::duration<double>>(t1 - t0);
    // double delta = time_span.count();
    // fri_timer = delta;
    //printf("Init %lf\n", delta);
    // println!("{:?}", witness_merkle[oracle_indicator]);
    return witness_merkle[oracle_indicator][1];
}
