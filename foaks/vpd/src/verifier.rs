use std::{char::MAX, thread::current};

use infrastructure::{
    constants::{LOG_SLICE_NUMBER, RS_CODE_RATE, SLICE_NUMBER},
    my_hash::{self, HashDigest},
};

use poly_commitment::LdtCommitment;
use prime_field::FieldElement;

use crate::fri::FRIContext;

pub fn verify_merkle(
    hash_digest: HashDigest,
    merkle_path: Vec<HashDigest>,
    len: usize,
    pow: i32,
    values: Vec<(FieldElement, FieldElement)>,
) -> bool {
    // We need to make sure the len is always smaller than the size of merklePath.
    assert!(merkle_path.len() >= len);

    let mut pow = pow;
    let current_hash: HashDigest = *merkle_path.last().unwrap();

    // don't mutate the current_hash, this is the output of the loop following
    let mut new_hash = HashDigest::new();

    for i in 0..(len - 1) {
        if (pow & i as i32).is_positive() {
            let data: [HashDigest; 2] = [merkle_path[i], current_hash];
            new_hash = my_hash::my_hash(data);
        } else {
            let data: [HashDigest; 2] = [current_hash, merkle_path[i]];
            new_hash = my_hash::my_hash(data);
        }
        pow /= 2;
    }

    let mut value_hash = HashDigest::new();

    for value in values {
        let data_element: [FieldElement; 2] = [value.0, value.1];
        // can't coerce type FieldElement to HashDigest
        // data_element[1] = value_hash; // ?
        // value_hash = my_hash::my_hash(data_element);
    }

    // QUESTION: should check if the function equals is custom one?
    // HashDigest::eq()
    hash_digest == new_hash && merkle_path.last() == Some(&value_hash)
}

// LdtCommitment

const MAX_FRI_DEPTH: usize = 0;

// Defined in fri.h
pub struct FriCommitPhaseData {
    pub merkle: [HashDigest; MAX_FRI_DEPTH],
    pub rs_codeword: [FieldElement; MAX_FRI_DEPTH],
    pub rs_codeword_mapping: Vec<[usize; MAX_FRI_DEPTH]>,
}

impl FRIContext {
    /// Request two values w^{pow0} and w^{pow1}, with merkle tree proof, where w is the root of unity and w^{pow0} and w^{pow1} are quad residue. Repeat ldt_repeat_num times, storing all result in vector.
    pub fn request_init_value_with_merkle(
        &mut self,
        pow_0: usize,
        pow_1: usize,
        // new_size: &i64,
        oracle_indicator: usize,
    ) -> (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>) {
        // we swap pow_0 and pow_1 when pow_0 > pow_1
        let (pow_0, pow_1) = if pow_0 > pow_1 {
            (pow_1, pow_0)
        } else {
            (pow_0, pow_1)
        };

        assert!(pow_0 + (1 << self.log_current_witness_size_per_slice) / 2 == pow_1);

        let mut value: Vec<(FieldElement, FieldElement)> = vec![];
        let log_leaf_size = LOG_SLICE_NUMBER + 1;

        let mut new_size = 0;
        for i in 0..SLICE_NUMBER {
            let element_1 = self.witness_rs_codeword_interleaved[oracle_indicator]
                [pow_0 << log_leaf_size | i << 1 | 0];

            let element_2 = self.witness_rs_codeword_interleaved[oracle_indicator]
                [pow_0 << log_leaf_size | i << 1 | 1];

            value.push((element_1, element_2));

            if !self.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 0] {
                self.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 0] = true;
            }
            if !self.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 1] {
                self.visited_witness[oracle_indicator][pow_0 << log_leaf_size | i << 1 | 1] = true;
            }
            new_size += std::mem::size_of::<FieldElement>();
        }

        let depth = self.log_current_witness_size_per_slice - 1;
        let mut com_hhash: Vec<HashDigest> = Vec::with_capacity(depth);

        // minus 1 since each leaf have 2 values (qual resi)
        let mut pos = pow_0 + (1 << (self.log_current_witness_size_per_slice - 1));

        let mut test_hash = self.witness_merkle[oracle_indicator][pos];
        com_hhash[depth] = test_hash;

        for i in 0..depth {
            if !self.visited_init[oracle_indicator][pos ^ 1] {
                new_size += std::mem::size_of::<HashDigest>();
            }
            self.visited_init[oracle_indicator][pos] = true;
            self.visited_init[oracle_indicator][pos ^ 1] = true;

            let data = if (pos & 1) == 1 {
                [self.witness_merkle[oracle_indicator][pos ^ 1], test_hash]
            } else {
                [test_hash, self.witness_merkle[oracle_indicator][pos ^ 1]]
            };
            test_hash = my_hash::my_hash(data);

            com_hhash[i] = self.witness_merkle[oracle_indicator][pos ^ 1];
            pos /= 2;

            assert_eq!(test_hash, self.witness_merkle[oracle_indicator][pos]);
        }
        assert!(pos == 1);
        (value, com_hhash)
    }

    /// Request the merkle proof to lvl-th level oracle, at w^{pow}, will also return it's quad residue's proof.
    /// returned value is unordered, meaning that one of them is the requested value and the other one is it's qual residue.
    pub fn request_step_commit(
        lvl: i64,
        pow: i64,
        new_size: i64,
    ) -> (Vec<(FieldElement, FieldElement)>, Vec<HashDigest>) {
        (vec![], vec![])
    }

    /// Given fold parameter r, return the root of the merkle tree of next level.
    pub fn commit_phrase_step(r: FieldElement) {}

    /// Return the final rs code since it is only constant size
    pub fn commit_phase_final() -> FieldElement {
        FieldElement::default()
    }
}

// No longer needed, we got FRIContext
pub struct VpdVerifier {
    cpd: FriCommitPhaseData,
    step: usize,
}

// SHould I put inside VPD?
impl VpdVerifier {
    pub fn init() {}

    pub fn commit_phrase_step(mut self, r: FieldElement) -> HashDigest {
        let log_current_witness_size_per_slice = 0; // TODO: ?
        let next_witness_size: usize = (1 << log_current_witness_size_per_slice) / 2;

        if self.cpd.rs_codeword[self.step] == FieldElement::default() {
            self.cpd.rs_codeword[self.step] =
            // QUESTION: from_img or from_real
                FieldElement::from_img(next_witness_size as u64 * /* SLICE_COUNT */ 0);
        }

        // let mut previous_witness = FieldElement::default();

        let (previous_witness, previous_witness_mapping) = match self.step {
            // TODO: virtual oracle
            0 => (FieldElement::default(), []),
            _ => (
                self.cpd.rs_codeword[self.step - 1],
                self.cpd.rs_codeword_mapping[self.step - 1],
            ),
        };

        for i in 0..next_witness_size {
            let qual_res_0 = i;
            let qual_res_1 = ((1 << (log_current_witness_size_per_slice - 1)) + i) / 2;

            let pos = usize::min(qual_res_0, qual_res_1);

            // TODO: L_group

            for j in 0..SLICE_NUMBER {
                let real_pos = previous_witness_mapping[pos << (LOG_SLICE_NUMBER | j)];
                // let x = self.cpd.rs_codeword[self.step][i << LOG_SLICE_NUMBER | j];
            }
        }

        HashDigest::new()
    }

    pub fn finish(&self) -> FieldElement {
        assert!(self.step != 0);
        self.cpd.rs_codeword[self.step - 1]
    }
}

// Return the hhash array of commitments, randomness and final small polynomial (represented by rscode)
// pub fn commit_phase(log_length: usize) -> LdtCommitment {
//     // LOG_SLICE_NUMBER;

//     let mut codeword_size = 1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

//     let mut randomness: Vec<FieldElement> =
//         Vec::with_capacity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

//     let mut ret: Vec<HashDigest> = Vec::with_capacity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

//     let mut ptr = 0;
//     while codeword_size > (1 << RS_CODE_RATE) {
//         randomness[ptr] = prime_field::FieldElement::new_random();
//         ret[ptr] = commit_step(randomness[ptr]);
//         codeword_size /= 2;
//         ptr += 1;
//     }

//     LdtCommitment {
//         commitment_hash: ret,
//         // final_rs_code: vpdVerifier::finish(),
//         randomness,
//         mx_depth: ptr,
//     }
// }
