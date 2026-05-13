OPENQASM 3.0;
int result = 0;
for int i in [0:5] {
    for int j in [0:5] {
        if (i == 2 && j == 3) {
            result = i + j;
            return;
        }
    }
}