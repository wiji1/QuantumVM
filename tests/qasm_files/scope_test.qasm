OPENQASM 3.0;

// Test variable scope
int[32] outer = 1;
{
    int[32] inner = 2;
    outer = 10;  // Can access outer
}
// inner not accessible here
output int[32] result;
result = outer;  // Should be 10
