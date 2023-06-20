#[cfg(test)]
mod test {
    use std::env;
    use std::fs::read_to_string;

    use infrastructure::merkle_tree::hash_double_field_element_merkle_damgard;
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
    fn verify_damgard() {
        // Extracted from i linear_pc/src/lib line 65
        let mut prev_hash = HashDigest::new_from_c(0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000);
        let mut x = FieldElement::new(1290311065, 751792346);
        let mut y = FieldElement::new(213961255, 3403933);

        unsafe {
            assert_eq!(
                HashDigest::new_from_c(0xfa7897b655bb128a, 0x58c3d9a3999de2f3, 0xa819bd64f1312f99, 0x79bfe75a8ffdb45b),
                hash_double_field_element_merkle_damgard(x, y, prev_hash)
            );
        }

        prev_hash = HashDigest::new_from_c(0xb57a052575651e28, 0x2771f4abfb7cfb46, 0x40c1484e5310ffa3, 0x3d6fe15a5a9486dd);
        x = FieldElement::new(751266410 ,2029902733);
        y = FieldElement::new(912098043, 2031678345);

        unsafe {
            assert_eq!(
                HashDigest::new_from_c(0x68b872dc68bf5200, 0x21af84922511cfff, 0x65d9dd125e0ca0f4, 0x4165f31efeaf4cd5),
                hash_double_field_element_merkle_damgard(x, y, prev_hash)
            );
        }
    }

    #[test]
    fn test_hash() {
        let src = [
            HashDigest::new_from_c(0x0,0x0,0x0,0x0),
            HashDigest::new_from_c(0x000000004ce89599, 0x000000002ccf70da, 0x000000000cc0ca27, 0x000000000033f09d)
        ];

        unsafe {
            assert_eq!(
                my_hash(src),
                HashDigest::new_from_c(0xfa7897b655bb128a, 0x58c3d9a3999de2f3, 0xa819bd64f1312f99, 0x79bfe75a8ffdb45b)
            )
        }
    }
}
