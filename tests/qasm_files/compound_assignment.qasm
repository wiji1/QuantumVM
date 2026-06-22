OPENQASM 3.0;

// Test compound assignment operators
output int[32] result1;
output int[32] result2;
output int[32] result3;
output int[32] result4;
output int[32] result5;

int[32] a = 10;
a += 5;
result1 = a;  // 15

a -= 3;
result2 = a;  // 12

a *= 2;
result3 = a;  // 24

a /= 4;
result4 = a;  // 6

a %= 4;
result5 = a;  // 2
