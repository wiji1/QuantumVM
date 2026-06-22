OPENQASM 3.0;

// Test function return type mismatch (should fail at type check)
def foo() -> int[32] {
    return 3.14;  // Returning float instead of int
}

int[32] x = foo();
