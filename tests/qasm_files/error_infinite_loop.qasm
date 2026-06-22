OPENQASM 3.0;

// Test potentially infinite loop (should run but might timeout)
int[32] x = 0;
while (true) {
    x += 1;
    if (x > 100) {
        break;
    }
}

output int[32] result;
result = x;
