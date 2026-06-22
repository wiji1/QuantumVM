OPENQASM 3.0;

// Test while loop
int[32] i = 0;
int[32] sum = 0;
while (i < 5) {
    sum = sum + i;
    i = i + 1;
}
