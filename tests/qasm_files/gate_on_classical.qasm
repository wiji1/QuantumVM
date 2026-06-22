OPENQASM 3.0;

// Test applying quantum gate to classical variable (should fail at type check)
int[32] x = 5;
h x;
