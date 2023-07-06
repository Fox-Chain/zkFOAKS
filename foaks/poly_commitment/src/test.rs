#[cfg(test)]
mod test {
    use std::env;
    use std::fs::read_to_string;

    use infrastructure::merkle_tree::{hash_double_field_element_merkle_damgard, hash_single_field_element};
    use infrastructure::my_hash::{HashDigest, my_hash, my_hash_blake};
    use prime_field::FieldElement;

    use crate::vpd::verifier::verify_merkle;

    fn generate_tuple_field_element(input: &str) -> (FieldElement, FieldElement) {
        let mut content = input.split_whitespace();

        (
            FieldElement {
                real: content.next().unwrap().parse::<u64>().unwrap(),
                img: content.next().unwrap().parse::<u64>().unwrap(),
            },
            FieldElement {
                real: content.next().unwrap().parse::<u64>().unwrap(),
                img: content.next().unwrap().parse::<u64>().unwrap(),
            },
        )
    }

    #[test]
    fn test_hash() {
        let mut data = [HashDigest::default(), HashDigest::default()];
        let fields = [
            FieldElement { real: 1804289383, img: 846930886 },
            FieldElement { real: 1957747793, img: 424238335 },
        ];

        //HashDigest::memcpy_from_field_elements(fields).print_c();

        unsafe {
            hash_single_field_element(fields[0]).print_c();
        }
    }
}
