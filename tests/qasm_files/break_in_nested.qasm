OPENQASM 3.0;

// Test break in nested loop
output int[32] count;
count = 0;

for int[8] i in [0:5] {
    for int[8] j in [0:5] {
        count += 1;
        if (j == 2) {
            break;  // Should only break inner loop
        }
    }
}
// count should be 6 * 3 = 18 (6 iterations of outer, 3 of inner each time)
