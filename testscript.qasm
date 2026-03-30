OPENQASM 3.0;
 qubit[4] q;
 int n = 2;
 for int i in {0, n, n+1} {
     h q[i];
 }