OPENQASM 3.0;

// Test nested gate definitions
gate bell q1, q2 {
    h q1;
    cx q1, q2;
}

gate prep_bell_measure q1, q2 {
    bell q1, q2;
}

qubit[2] q;
prep_bell_measure q[0], q[1];

output bit[2] result;
result = measure q;
