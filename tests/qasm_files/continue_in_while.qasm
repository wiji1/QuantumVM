OPENQASM 3.0;

// Test continue in while loop
output int[32] sum;
sum = 0;
int[32] i = 0;

while (i < 10) {
    i += 1;
    if (i % 2 == 0) {
        continue;
    }
    sum += i;
}
// sum should be 1 + 3 + 5 + 7 + 9 = 25
