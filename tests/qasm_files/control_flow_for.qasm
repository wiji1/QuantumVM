OPENQASM 3.0;

// Test for loop
int[32] sum = 0;
for int i in [0:5] {
    sum = sum + i;
}
