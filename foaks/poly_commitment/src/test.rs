#[cfg(test)]
mod test {
    use crate::vpd::verifier::verify_merkle;
    use infrastructure::my_hash::{HashDigest, my_hash, my_hash_blake};
    use prime_field::FieldElement;
    use std::env;
    use std::fs::read_to_string;

    fn generate_hash_digest(h0: &str, h1: &str) -> HashDigest {
        HashDigest {
            h0: u128::from_str_radix(h0, 16).unwrap(),
            h1: u128::from_str_radix(h1, 16).unwrap(),
        }
    }

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
    fn verify_merkle_test() {
        let hash_digest = generate_hash_digest(
            "5dd1fdf47f03d24f100d896a8561dba5",
            "68f7438ca99737561044a4fdac19f9f7",
        );
        let merkle_path = vec![
            generate_hash_digest(
                "8f6a92ecadaf4d3527cc9fd7fd7e7b6b",
                "bd36cbb14173b495396fc823caf5221b",
            ),
            generate_hash_digest(
                "54790f164bf92ff597e22de9b43d7cbe",
                "6816da2ba3277db9710646523198be15",
            ),
            generate_hash_digest(
                "02a08c9e3d5675355d358225f8f0c5a0",
                "460a641d79608e67c0f2f887c32ba68f",
            ),
            generate_hash_digest(
                "9ebf71f69c4bcce8d72b3d35af3c3882",
                "88ee8499825ef530f5e62569f59bb525",
            ),
            generate_hash_digest(
                "6dd05bc00b9d56e84d517c6b55fc1cf7",
                "6660db3c1c916e2cf202e04e70d34ee3",
            ),
            generate_hash_digest(
                "c77e964bcf2553fdab9eec4d257590d3",
                "b186d0f7cd344d822030f9a20c8cc4fc",
            ),
            generate_hash_digest(
                "2dee9d01a08368592e929c61f70f1d00",
                "8ad15424147427787cfb2edd52ed26a7",
            ),
            generate_hash_digest(
                "19169f81ced2c32fe24a9f1fdf821c42",
                "bdec40413a8121a5585e49426a8190d7",
            ),
        ];

        let len = 8;
        let pow = 70;

        let mut values = vec![];
        let content =
            read_to_string(env::current_dir().unwrap().join("src/value_for_merkle.txt")).unwrap();
        for line in content.lines() {
            values.push(generate_tuple_field_element(line));
        }

        assert_eq!(
            true,
            verify_merkle(hash_digest, &merkle_path, len, pow, &values)
        );
    }

    #[test]
    fn hash_digest() {
        let input = [
            generate_hash_digest(
                "5dd1fdf47f03d24f100d896a8561dba5",
                "68f7438ca99737561044a4fdac19f9f7",
            ),
            generate_hash_digest(
                "5dd1fdf47f03d24f100d896a8561dba5",
                "68f7438ca99737561044a4fdac19f9f7",
            )
        ];

        println!("C++ hash:   {:?}", generate_hash_digest("f5ef8ca9d101cd82e38f39c60b1a5006", "06c40ef077ba2f8b70ac0aa9384d5c3e"));
        println!("Ring hash:  {:?}", my_hash(input));
        println!("Blake hash: {:?}", my_hash_blake(input));
        /*
        assert_eq!(
            my_hash(input),
            generate_hash_digest("f5ef8ca9d101cd82e38f39c60b1a5006", "06c40ef077ba2f8b70ac0aa9384d5c3e")
        );

        assert_eq!(
            my_hash_blake(input),
            generate_hash_digest("f5ef8ca9d101cd82e38f39c60b1a5006", "06c40ef077ba2f8b70ac0aa9384d5c3e")
        );*/
    }
}
