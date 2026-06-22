OPENQASM 3.0;

// Test assigning void function return (should fail at type check)
def foo() {
    int[32] x = 5;
}

int[32] y = foo();  // foo returns void
