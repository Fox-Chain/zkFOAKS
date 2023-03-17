pub mod circuit_fast_track;
pub mod config;
pub mod polynomial;
pub mod prover;
pub mod verifier;

#[cfg(test)]
mod test {
    use prime_field::FieldElement;

    #[test]
    fn test() {
        let mut a = vec![FieldElement::zero(); 3];
        for i in 0..a.len() {
            a[i] = FieldElement::real_one();
        }
        //a.push(FieldElement::real_one());
        println!("{:?}", a);
    }
}