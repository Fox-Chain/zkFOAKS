# Fox's R1CS

R1CS(Rank-1 Constraint System) used in Fox.

## Build guide

The library compiles on the `stable` toolchain of the Rust compiler. To install the latest version of Rust, first install `rustup` by following the instructions [here](https://rustup.rs/), or via your platform's package manager. Once `rustup` is installed, install the Rust toolchain by invoking:

```bash
rustup install stable
```

After that, use `cargo`, the standard Rust build tool, to build the library:

```bash
git clone https://github.com/Fox-Chain/r1cs.git
cargo build --release
```

This library comes with unit tests for each of the provided crates. Run the tests with:

```bash
cargo test
```
