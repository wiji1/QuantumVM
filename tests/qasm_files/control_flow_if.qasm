OPENQASM 3.0;

// Test if statement
int[32] x = 5;
int[32] result;
if (x > 0) {
    result = 1;
} else {
    result = -1;
}
