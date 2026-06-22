OPENQASM 3.0;

// Test break and continue
int[32] sum = 0;
for int i in [0:10] {
    if (i == 5) {
        break;
    }
    if (i == 2) {
        continue;
    }
    sum = sum + i;
}
