OPENQASM 3.0;

// Simpler concatenation test
qubit q1;
qubit q2;
bit r1 = measure (q1 ++ q2)[0];
