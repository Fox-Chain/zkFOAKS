# Memory Operations' R1CS matrices configuration

In this doc, the constraint system of memory operations, matrices property and performance are explained.

## Constraint System and Matrices Property

In memory operation, traces from geth are collected and configured into a table. Checking the correctness and soundness of these traces ensure that the memory operations are done correctly and sequentially.

In the memory operation check, there are four main types of constraint system.

### Boolean check

Selector or Operation indicator in the table needed to be either one or zero.
**Constraint System: (x)(x-1)=0, x∈{lastAccess, mOp, mWr}**
Variables: One, Output=0, x
Matrix size: `1x3`
Matrices:

```
A [0, 0, 1]
B [-1, 0, 1]
C [0, 1, 0]
```

Memory Usage: TODO

### Increment check

If it is last access of a memory slot, address of next line in the table should be incremental.
**Constraint System:(1-lastAccess)(addr'-addr)=0**
Variables: One, Output=0, lastAccess, addr', addr
Matrix size: `1x5`
Matrices:

```
A [1, 0, -1, 0, 0]
B [0, 0, 0, 1, -1]
C [0, 1, 0, 0, 0]
```

Memory Usage: TODO

### Memory operation check

Checking whether mOp or mWr is one or zero.
**Constraint System: (1-mOp)(mWr)=0**
Variables: One, Output=0, mOp, mWr
Matrix size: `1x4`
Matrices:

```
A [1, 0, -1, 0]
B [0, 0, 0, 1]
C [0, 1, 0, 0]
```

Memory Usage: TODO

### Value increment check

Memory value consist of 8 registers with 4 bytes each, total 32 bytes(256 bits). If memory slot changes, value will also change. For better performance and lesser memory usage, we check the value of 4 registers on one constraint system.
**Constraint System: (1-mOp'\*mWr')(1-lastAccess)(val[0..7]'-val[0..7])=0**
Variables:

1. One
2. Output = 0
3. mid_1 = mOp\*mWr
4. mid_2 = (1-mid_1)\*(1-lastAccess)
5. mOpr
6. mWr
7. lastAccess
8. val'[0..3]
9. val[0..3]

Matrix size: `3x8`
Matrices:

```
A [0, 0, 0, 0, 1, 0, 0, 0]
  [1, 0, -1, 0, 1, 0, 0, 0, 0]
  [0, 0, 0, 1, 0, 0, 0, 0, 0]
B [0, 0, 0, 0, 0, 1, 0, 0, 0]
  [1, 0, 0, 0, 0, 0, -1, 0, 0]
  [0, 0, 0, 0, 0, 0, 0, 1, -1]
C [0, 0, 1, 0, 0, 0, 0, 0, 0]
  [0, 0, 0, 1, 0, 0, 0, 0, 0]
  [0, 1, 0, 0, 0, 0, 0, 0, 0]
```

Memory Usage: TODO
