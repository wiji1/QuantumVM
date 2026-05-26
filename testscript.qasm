OPENQASM 3.0;
qubit[2] q;
output bit[2] c;
U(pi/2, 0, pi) q[0];
CX q[0], q[1];
c = measure q;