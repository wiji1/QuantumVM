OPENQASM 3.0;

// Test missing return statement (should fail at type check)
def getValue() -> int[32] {
    int[32] x = 5;
    // Missing return statement
}

int[32] y = getValue();
