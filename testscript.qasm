OPENQASM 3.0;
output int total = 0;
for uint i in [0:3] {
    for uint j in [0:3] {
        total += 1;
    }
}