OPENQASM 3.0;

// Test accessing variable outside its scope (should fail at type check)
{
    int[32] x = 5;
}
int[32] y = x;  // x not in scope
