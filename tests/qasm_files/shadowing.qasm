OPENQASM 3.0;

// Test variable shadowing
int[32] x = 5;
{
    int[32] x = 10;  // Shadow outer x
}
// x should still be 5 here
