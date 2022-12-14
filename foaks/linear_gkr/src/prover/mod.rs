use crate::circuit_fast_track::LayeredCircuit;

use prime_field::FieldElement;

pub fn from_string(s: &str) -> FieldElement {
    let mut ret = FieldElement::from_real(0);

    for byte in s.bytes() {
        let digit = byte - b'0';
        ret = ret * FieldElement::from_real(10) + FieldElement::from_real(digit.into());
    }

    ret
}

static mut INV_2: FieldElement = FieldElement::zero();

pub fn get_circuit(from_verifier: &LayeredCircuit) {
    let C = from_verifier;
    INV_2 = prime_field::INITIALIZED;
}

pub struct ZKProver {}
