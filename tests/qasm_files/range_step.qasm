OPENQASM 3.0;

// Test range with step
int[32] sum = 0;
for int i in [0:10:2] {  // 0, 2, 4, 6, 8
    sum = sum + i;
}
