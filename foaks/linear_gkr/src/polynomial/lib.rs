#[cfg(test)]
mod test {
    use prime_field::FieldElement;
    use crate::polynomial::QuadraticPoly;

    #[test]
    fn test_eval() {
        let a = FieldElement::new(1, 2);
        let b = FieldElement::new(2, 2);
        let c = FieldElement::new(3, 2);

        let x = FieldElement::new(2, 2);

        println!("MUL {:?}", a * b);

        let q_poly = QuadraticPoly::new(a, b, c);
        println!("{:?}", q_poly.eval(&x));
        //assert_eq!(FieldElement::new(), q_poly.eval(3));
    }
}