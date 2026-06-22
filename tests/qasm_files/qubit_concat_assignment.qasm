OPENQASM 3.0;

// Test qubit concatenation with assignment (not declaration)
qubit q1;
qubit q2;
h q1;
x q2;
qubit[2] q;
q = q1 ++ q2;
