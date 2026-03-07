OPENQASM 3.0;
qubit[2] q;
bit[2] c;

h q[0];        // superposition
cx q[0], q[1]; // entangle
c = measure q; // measure both