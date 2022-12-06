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
    values: Vec<FieldElement>,
) -> bool {
    // We need to make sure the len is always smaller than the size of merklePath.
    assert!(merkle_path.len() > len);

    let current_hash: HashDigest = *merkle_path.last().unwrap();

    let mut data: [HashDigest; 2] = [HashDigest::new(), HashDigest::new()];
    let value_hash: [HashDigest; 2] = [HashDigest::new(), HashDigest::new()];

    let mut new_hash = HashDigest::new();

    for i in 0..len {
        if (pow & i as i32).is_positive() {
            data[0] = merkle_path[i];
            data[1] = current_hash;
        } else {
            data[0] = current_hash;
            data[1] = merkle_path[i];
        }

        let pow = pow / 2;
        new_hash = my_hash::my_hash(data);

        // TODO: field element needs to be done
    }

    for i in 0..len {
        let element = values[i];
    }

    hash_digest == new_hash // && merkle_path.last() == Some(value_hash)
}

// LdtCommitment

const MAX_FRI_DEPTH: usize = 0;

// Defined in fri.h
pub struct FriCommitPhaseData {
    pub merkle: [HashDigest; MAX_FRI_DEPTH],
    pub rs_codeword: [FieldElement; MAX_FRI_DEPTH],
    pub rs_codeword_mapping: Vec<[usize; MAX_FRI_DEPTH]>,
}

pub struct VpdVerifier {
    cpd: FriCommitPhaseData,
    step: usize,
}

impl VpdVerifier {
    pub fn init() {}

    pub fn commit_phase_step(mut self, r: FieldElement) -> HashDigest {
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

// poly_commit::ldt_commitment poly_commit::poly_commit_prover::commit_phase
pub fn commit_phase(log_length: usize) -> LdtCommitment {
    // LOG_SLICE_NUMBER;

    let mut codeword_size = 1 << (log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

    let mut randomness: Vec<FieldElement> =
        Vec::with_capacity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

    let mut ret: Vec<HashDigest> = Vec::with_capacity(log_length + RS_CODE_RATE - LOG_SLICE_NUMBER);

    let mut ptr = 0;
    while codeword_size > (1 << RS_CODE_RATE) {
        randomness[ptr] = prime_field::FieldElement::new_random();
        ret[ptr] = commit_step(randomness[ptr]);
        codeword_size /= 2;
        ptr += 1;
    }

    LdtCommitment {
        commitment_hash: ret,
        // final_rs_code: vpdVerifier::finish(),
        randomness,
        mx_depth: ptr,
    }
}

pub fn commit_step(r: FieldElement) -> HashDigest {
    HashDigest::new()
}
