OPENQASM 3.0;
qubit[4] q;
output bit[4] c;
for int i in [0:4] {
    h q[i];
}
c = measure q;