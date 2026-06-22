OPENQASM 3.0;

// Test range with negative step
output int[32] sum;
sum = 0;

for int[8] i in [10:-2:0] {
    sum += i;
}
// sum should be 10 + 8 + 6 + 4 + 2 = 30
