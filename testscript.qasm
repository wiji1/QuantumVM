OPENQASM 3.0;
int test = 0;

def adder() {
    test -= 1;
}

for int i in [0:10] {
    adder();
}