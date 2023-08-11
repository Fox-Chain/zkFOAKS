# zkFOAKS

Repository for Orion of foaks , includes implementations of R1CS and Virgo.

### General Polynomial Commitment
We support both univariant and multivariant polynomial commitment schemes. Currently, up to 20.

### Succinct proof size
The proof size of our system is $O(\log^2 N)$

### State-of-the-art performance
We offer the fastest prover that can prove $2^{27}$ coefficients within $100$s in single thread mode.

### Expander testing
We offer our expander testing protocol for people to set up their own expander.

# Examples and tests
## Univariant Polynomial Commitment
### Build
```
cargo run 14 text.txt
```

## Multivariant Polynomial Commitment
### Build
```
cargo run 14 text.txt multi
```