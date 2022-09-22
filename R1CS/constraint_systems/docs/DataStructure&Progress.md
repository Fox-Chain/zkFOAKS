# R1CS Data Structure and progress explained

In this doc, the data structure used in R1CS constraint system and matrices generation is illustrated. Besides, the simplified progress of R1CS progress is explained.

## Data Structure

1. Constraint System
   1. Number of public variable/instance
   2. Public variable’s value
   3. Number of private variable/witness
   4. Private variable’s value
   5. Number of linear combination
   6. Linear combination index in each A,B,C matrices
2. Matrices
   1. A,B,C matrices, each has: matrix element and it’s corresponding index
   2. Number of non zero element in A,B,C matrices

## R1CS progress

1. Construct a new constraint system, with constant ONE pre-assigned.
2. Flatten the circuit

   $x^3+x+5=35$

   ```rust
   sym_1 = x * x
   y = sym_1 * x
   sym_2 = y + x
   ~out = sym_2 + 5
   ```

3. Define public variable and private variable
   1. Public Variable: 1, 35
   2. Private Variable: x, sym_1, y, sym_2
4. Write constraint in the form of **A\*B=C,** where A, B, C is a linear combination for each constraint.

   For every linear combination, their index is also record accordingly.

   ```rust
   // A*B=C
   // x*x=sym_1
   generate_constraint(x,x,sym_1)
   // For first constraint, LCindex(A)=0, LCindex(B)=1, LCindex(C)=2,
   // For secibd constraint, LCindex(A)=3, LCindex(B)=4, LCindex(C)=5,...
   ```

5. Generate matrices based on constraint systems

   For example:

   ```rust
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
   ```

   We have:

   ```rust
   // matrix = [[(a,b),(c,d)],[],[],[],...[]]
    // number of 1D element = number of rows in matrix
    // 2D elements = (value, index)
   matrix_A= [[(1,2)],[(1,3)],[(1,2),(1,4)],[(5,0),(1,5)]]
   matrix_B=...
   matrix_C=...
   A_non_zero=6
   B_non_zero=4
   C_non_zero=4
   ```
