OPENQASM 3.0;

// Test division by zero (should fail at runtime)
int[32] x = 10;
int[32] y = 0;
int[32] z = x / y;
