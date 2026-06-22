OPENQASM 3.0;

// Test nested loops
output int[32] sum;
sum = 0;

for int[8] i in [0:3] {
    for int[8] j in [0:2] {
        sum += 1;
    }
}
// sum should be 3 * 3 = 9
