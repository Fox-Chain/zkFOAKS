use crate::lc;
use crate::r1cs::*;
use ark_bls12_381::*;
use ark_test_curves::bls12_381::Fr;
pub type ConstantFr = ark_test_curves::bls12_381::Fr;

pub struct sampleModule {
    a: u8,
    b: u8,
    c: u8,
}

impl ConstraintSynthesizer<ConstantFr> for sampleModule {
    fn generate_constraints(self, cs: ConstraintSystemRef<ConstantFr>) -> Result<()> {
        let cs = ConstraintSystem::<ConstantFr>::new_ref();
        let a = cs
            .new_input_variable(|| Ok(ConstantFr::from(self.a)))
            .unwrap();
        println!("a={:#?}", a);
        let b = cs
            .new_witness_variable(|| Ok(ConstantFr::from(self.b)))
            .unwrap();
        println!("b={:#?}", b);
        let c = cs
            .new_witness_variable(|| Ok(ConstantFr::from(self.c)))
            .unwrap();
        println!("c={:#?}", c);
        cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)
            .unwrap();
        cs.inline_all_lcs();
        //let matrices = cs.to_matrices().unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_struct_correct() {
        let sample = sampleModule { a: 2, b: 3, c: 1 };
        let a = 2;
        assert_eq!(sample.b, 3);
        println!("here")
    }

    #[test]
    fn proof_generate_correct() {
        use ark_groth16::Groth16;
        use ark_snark::SNARK;
        use ark_test_curves::bls12_381::Fr;
        let mut rng = ark_std::test_rng();
        let sample_circuit = sampleModule {
            a: 3u8,
            b: 2u8,
            c: 4u8,
        };
        let cs = ConstraintSystem::new_ref();
        sample_circuit.generate_constraints(cs.clone()).unwrap();
        let (pk, vk) =
            Groth16::<bls12_381>::circuit_specific_setup(sample_circuit, &mut rng).unwrap();
        let public_input = [sample_circuit.a, sample_circuit.b];

        let proof = Groth16::prove(&pk, sample_circuit, &mut rng).unwrap();
        let valid_proof = Groth16::verify(&vk, &public_input, &proof).unwrap();
        assert!(valid_proof);
    }
}
