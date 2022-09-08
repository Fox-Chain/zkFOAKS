fn matrix_generation_example() -> crate::r1cs::Result<()> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    // helper definitions
    let three = Fr::from(3u8);
    let five = Fr::from(5u8);
    let nine = Fr::from(9u8);
    // There will be six variables in the system, in the order governed by adding
    // them to the constraint system (Note that the CS is initialised with
    // `Variable::One` in the first position implicitly).
    // Note also that the all public variables will always be placed before all witnesses
    //
    // Variable::One
    // Variable::Instance(35)
    // Variable::Witness(3) ( == x )
    // Variable::Witness(9) ( == sym_1 )
    // Variable::Witness(27) ( == y )
    // Variable::Witness(30) ( == sym_2 )

    // let one = Variable::One; // public input, implicitly defined
    let out = cs.new_input_variable(|| Ok(nine * three + three + five))?; // public input
    let x = cs.new_witness_variable(|| Ok(three))?; // explicit witness
    let sym_1 = cs.new_witness_variable(|| Ok(nine))?; // intermediate witness variable
    let y = cs.new_witness_variable(|| Ok(nine * three))?; // intermediate witness variable
    let sym_2 = cs.new_witness_variable(|| Ok(nine * three + three))?; // intermediate witness variable

    cs.enforce_constraint(lc!() + x, lc!() + x, lc!() + sym_1)?;
    cs.enforce_constraint(lc!() + sym_1, lc!() + x, lc!() + y)?;
    cs.enforce_constraint(lc!() + y + x, lc!() + Variable::One, lc!() + sym_2)?;
    cs.enforce_constraint(
        lc!() + sym_2 + (five, Variable::One),
        lc!() + Variable::One,
        lc!() + out,
    )?;

    cs.finalize();
    assert!(cs.is_satisfied().is_ok());
    let matrices = cs.to_matrices().unwrap();
    let mut file_matrices = File::create("./matrices.txt").expect("error");
    let output_matrices = format!("{:#?}", matrices);
    file_matrices.write_all(output_matrices.as_bytes());
    // There are four gates(constraints), each generating a row.
    // Resulting matrices:
    // (Note how 2nd & 3rd columns are swapped compared to the online example.
    // This results from an implementation detail of placing all Variable::Instances(_) first.
    //
    // A
    // [0, 0, 1, 0, 0, 0]
    // [0, 0, 0, 1, 0, 0]
    // [0, 0, 1, 0, 1, 0]
    // [5, 0, 0, 0, 0, 1]
    // B
    // [0, 0, 1, 0, 0, 0]
    // [0, 0, 1, 0, 0, 0]
    // [1, 0, 0, 0, 0, 0]
    // [1, 0, 0, 0, 0, 0]
    // C
    // [0, 0, 0, 1, 0, 0]
    // [0, 0, 0, 0, 1, 0]
    // [0, 0, 0, 0, 0, 1]
    // [0, 1, 0, 0, 0, 0]
    assert_eq!(matrices.a[0], vec![(Fr::one(), 2)]);
    assert_eq!(matrices.b[0], vec![(Fr::one(), 2)]);
    assert_eq!(matrices.c[0], vec![(Fr::one(), 3)]);

    assert_eq!(matrices.a[1], vec![(Fr::one(), 3)]);
    assert_eq!(matrices.b[1], vec![(Fr::one(), 2)]);
    assert_eq!(matrices.c[1], vec![(Fr::one(), 4)]);

    assert_eq!(matrices.a[2], vec![(Fr::one(), 2), (Fr::one(), 4)]);
    assert_eq!(matrices.b[2], vec![(Fr::one(), 0)]);
    assert_eq!(matrices.c[2], vec![(Fr::one(), 5)]);

    assert_eq!(matrices.a[3], vec![(five, 0), (Fr::one(), 5)]);
    assert_eq!(matrices.b[3], vec![(Fr::one(), 0)]);
    assert_eq!(matrices.c[3], vec![(Fr::one(), 1)]);
    Ok(())
}
