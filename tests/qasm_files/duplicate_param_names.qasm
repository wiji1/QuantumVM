OPENQASM 3.0;

// Test duplicate parameter names in function (should fail at type check)
def func(int[32] x, int[32] x) -> int[32] {
    return x;
}
