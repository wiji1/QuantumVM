OPENQASM 3.0;

// Test chained comparison
output bool result1;
output bool result2;

int[32] a = 5;
int[32] b = 10;
int[32] c = 15;

result1 = (a < b) && (b < c);  // Should be true
result2 = (a > b) || (b < c);  // Should be true
