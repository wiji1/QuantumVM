OPENQASM 3.0;
include "stdgates.inc";

// Test that stdgates are included correctly
qubit q;
h q;
s q;
t q;
sdg q;  // S dagger
tdg q;  // T dagger
