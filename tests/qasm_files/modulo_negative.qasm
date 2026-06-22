OPENQASM 3.0;

// Test modulo with negative numbers
output int[32] result1;
output int[32] result2;

int[32] a = -7;
int[32] b = 3;

result1 = a % b;      // Behavior depends on implementation
result2 = (-10) % 4;
