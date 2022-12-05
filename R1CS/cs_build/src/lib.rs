// //! Core interface for working with various relations that are useful in
// //! zkSNARKs. At the moment, we only implement APIs for working with Rank-1
// //! Constraint Systems (R1CS).

//#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    unused,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    missing_docs
)]
#![deny(unsafe_code)]

#[macro_use]
extern crate ark_std;

pub mod mat_gen;
pub mod r1cs;

#[cfg(test)]
mod tests {
    use crate::lc;
    use crate::mat_gen::mem_gen::*;
    use crate::mat_gen::stack_gen::push_num_check;
    use crate::r1cs::*;
    use ark_ff::BigInteger256;
    use ark_test_curves::bls12_381::Fr;
    use std::{fs::File, io::Write};
    #[test]
    fn mstore_check_val() {
        let file_tx =
            File::open("./src/data/tx_mem_table.json").expect("file should open read only");
        let json_tx: serde_json::Value =
            serde_json::from_reader(file_tx).expect("file should be proper JSON");
        let data = json_tx.get("data").expect("file should have data key");
        let mWr = data.as_array().unwrap()[0]["m_wr"].as_u64().unwrap();
        let mWr8 = data.as_array().unwrap()[0]["m_wr8"].as_u64().unwrap();
        let val_src_0 = data.as_array().unwrap()[0]["val_src_0"].as_u64().unwrap();
        let val_src_1 = data.as_array().unwrap()[0]["val_src_1"].as_u64().unwrap();
        let val_src_2 = data.as_array().unwrap()[0]["val_src_2"].as_u64().unwrap();
        let val_src_3 = data.as_array().unwrap()[0]["val_src_3"].as_u64().unwrap();
        let val_dst_0 = data.as_array().unwrap()[0]["val_dst_0"].as_u64().unwrap();
        let val_dst_1 = data.as_array().unwrap()[0]["val_dst_1"].as_u64().unwrap();
        let val_dst_2 = data.as_array().unwrap()[0]["val_dst_2"].as_u64().unwrap();
        let val_dst_3 = data.as_array().unwrap()[0]["val_dst_3"].as_u64().unwrap();

        let matrices = val_check_matrix_gen(
            mWr, mWr8, val_src_0, val_src_1, val_src_2, val_src_3, val_dst_0, val_dst_1, val_dst_2,
            val_dst_3,
        );
        assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 2)]);
        assert_eq!(
            matrices.a[1],
            vec![(Fr::from(1u64), 0), (Fr::from(1u64), 4)]
        );
        assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 3)]);
        assert_eq!(matrices.b[1], vec![(Fr::from(1u64), 5), (Fr::from(-1), 6)]);
        assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 4)]);
        assert_eq!(matrices.c[1], vec![(Fr::from(1u64), 1)]);
    }
    #[test]
    fn mwr_mwr8_check() {
        let file_tx =
            File::open("./src/data/tx_mem_table.json").expect("file should open read only");
        let json_tx: serde_json::Value =
            serde_json::from_reader(file_tx).expect("file should be proper JSON");
        let data = json_tx.get("data").expect("file should have data key");
        let mWr = data.as_array().unwrap()[0]["m_wr"].as_u64().unwrap();
        let mWr8 = data.as_array().unwrap()[0]["m_wr8"].as_u64().unwrap();
        let matrices = mWr_mWr8_check_matrix_gen(mWr, mWr8);
        assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 0), (Fr::from(-1), 2)]);
        assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 3)]);
        assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    }

    fn bool_check() {
        let file_tx =
            File::open("./src/data/tx_mem_table.json").expect("file should open read only");
        let json_tx: serde_json::Value =
            serde_json::from_reader(file_tx).expect("file should be proper JSON");
        let data = json_tx.get("data").expect("file should have data key");
        let mWr = data.as_array().unwrap()[0]["m_wr"].as_u64().unwrap();
        let matrices = boolean_check(mWr);
        assert_eq!(matrices.a[0], vec![(Fr::from(1u64), 2)]);
        assert_eq!(matrices.b[0], vec![(Fr::from(-1), 0), (Fr::from(1u64), 2)]);
        assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    }

    #[test]
    fn push_num_test() {
        let file_tx =
            File::open("./src/data/tx_stack_table.json").expect("file should open read only");
        let json_tx: serde_json::Value =
            serde_json::from_reader(file_tx).expect("file should be proper JSON");
        let data = json_tx.get("data").expect("file should have data key");
        let is_push_in = data.as_array().unwrap()[0]["is_push"].as_u64().unwrap();
        let push_num_in = data.as_array().unwrap()[0]["push_num"].as_u64().unwrap();
        let matrices = push_num_check(is_push_in, push_num_in);
        assert_eq!(matrices.a[0], vec![(Fr::from(-1), 0), (Fr::from(1u64), 2)]);
        assert_eq!(matrices.b[0], vec![(Fr::from(1u64), 3)]);
        assert_eq!(matrices.c[0], vec![(Fr::from(1u64), 1)]);
    }
}
