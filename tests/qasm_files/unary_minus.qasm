OPENQASM 3.0;

// Test unary minus operator
output int[32] result1;
output float[64] result2;

int[32] a = 5;
result1 = -a;      // Should be -5
result2 = -3.14;   // Should be -3.14
