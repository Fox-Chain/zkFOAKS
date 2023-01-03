# Interface for Prover & Temp Interface for Verifier

## File Format

In the first line, input a integer `d` represent the number of layers.
Input layer has layer number `0` output layer has layer number `d-1`

For next `d` lines, each line specify a layer.

In this first line, it specifies the input layer, the first number is `n`, specifies the number of gates in this layer, next `4n` numbers represents `n` gates. For each gate, we use 4 integers to describe: `ty gate_id value0 value1`, indicates the type of the gate, the id of the gate and the input value of the gate.

For `i`-th line, it specifies the layer `i - 1`, the first number is `n`, specifies the number of gates in this layer. `n` must be a power of `2`.
The rest of this line contains `4n` integers, represent `n` gates. For each gate, we use 4 integers to describe: `ty g u v`, indicates the type of the gate, and the connection of the gate, `g` is the gate number, `u` is the left input of the gate, `v` is the right input of the gate.

We have `11` different types of gates for now.

`ty=0` is addition gate, `ty=1` is multiplication gate, `ty=2` is dummy gate, `ty=3` is input gate, `ty=4` is direct relay gate, `ty=5` is summation gate， `ty=6` is not gate, `ty=7` is minus gate, `ty=8` is XOR gate, `ty=9` is NAAB gate ($\not x \land y$), `ty=10` is relay gate.

## Special gate explain
### Direct relay gate
Do not use it in the circuit description, it's a gate that we use it to simplify computation. The gate just directly copy the value from the node in previous layer which has the same label as the direct relay gate.

### Summation gate
It's a gate that output the summation of previous layer. A simple use case is matrix multiplication.

## Example
```
3 \\ three layers
4 3 0 1 1 3 1 1 1 3 2 1 1 3 3 1 1
2 0 0 0 1 1 1 2 3 \\ first gate is addition, and second is a multiplication
1 1 0 0 1 \\ this is the output layer, it's a multiplication gate
```

## Vigro/linear_gkr code explain (Peter's notes)

> **_NOTE:_**  This are just my notes base on my understanding.

### Matrix multiplication test explain

```bash
git clone git@github.com:sunblaze-ucb/Virgo.git
```

```bash
cd tests/matmul
python build.py
python run.py
```

or

```bash
cd tests/matmul
g++ gen.cpp -o gen -O3
./gen 16 mat_16_circuit.txt mat_16_meta.txt
./gen 32 mat_32_circuit.txt mat_32_meta.txt
./gen 64 mat_64_circuit.txt mat_64_meta.txt
./gen 128 mat_128_circuit.txt mat_128_meta.txt
./gen 256 mat_256_circuit.txt mat_256_meta.txt
```

To understand the matrix generation check the gen.cpp in this folder

### Use this line to run main.rs
```bash
cd src/
cargo run main.rs mat_16_circuit.txt mat_16_meta.txt LOG/mat_16.txt
```