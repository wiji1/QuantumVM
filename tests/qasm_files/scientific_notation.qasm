OPENQASM 3.0;


output int test;

output float[64] x;
test = 1;

// Test scientific notation
x = 1.5e-10;
float[64] y = 3.0E+8;
