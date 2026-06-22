OPENQASM 3.0;

// Test power operator **
output int[32] result1;
output float[64] result2;
output int[32] result3;

result1 = 2 ** 10;     // Should be 1024
result2 = 2.0 ** 0.5;  // Should be ~1.414
result3 = 3 ** 3;      // Should be 27
