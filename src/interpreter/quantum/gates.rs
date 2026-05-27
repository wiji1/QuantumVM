use num_complex::Complex;

pub type C64 = Complex<f64>;
pub type Matrix2 = [[C64; 2]; 2];
pub type Matrix4 = [[C64; 4]; 4];
pub type Matrix8 = [[C64; 8]; 8];

fn c(re: f64, im: f64) -> C64 { C64::new(re, im) }

pub fn gate_cx() -> Matrix4 {
    [
        [c(1.0,0.0), c(0.0,0.0), c(0.0,0.0), c(0.0,0.0)],
        [c(0.0,0.0), c(1.0,0.0), c(0.0,0.0), c(0.0,0.0)],
        [c(0.0,0.0), c(0.0,0.0), c(0.0,0.0), c(1.0,0.0)],
        [c(0.0,0.0), c(0.0,0.0), c(1.0,0.0), c(0.0,0.0)]
    ]
}

pub fn gate_u(theta: f64, phi: f64, lambda: f64) -> Matrix2 {
    let cos = (theta / 2.0).cos();
    let sin = (theta / 2.0).sin();
    [
        [c(cos, 0.0),c(-sin * lambda.cos(), -sin * lambda.sin())],
        [
            c(sin * phi.cos(), sin * phi.sin()),
            c(cos * (phi + lambda).cos(), cos * (phi + lambda).sin())
        ]
    ]
}

pub fn identity_2() -> Matrix2 {
    [[c(1.0, 0.0), c(0.0, 0.0)], [c(0.0, 0.0), c(1.0, 0.0)]]
}

pub fn conjugate_transpose_2(m: &Matrix2) -> Matrix2 {
    [[m[0][0].conj(), m[1][0].conj()], [m[0][1].conj(), m[1][1].conj()]]
}

pub fn matrix_pow_2(m: &Matrix2, n: i64) -> Matrix2 {
    if n == 0 { return identity_2(); }
    if n < 0 { return matrix_pow_2(&conjugate_transpose_2(m), -n); }
    let mut result = identity_2();
    for i in 0..n {
        result = matrix_mul_2(&result, m);
    }
    result
}

pub fn matrix_mul_2(a: &Matrix2, b: &Matrix2) -> Matrix2 {
    let mut result = [[c(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

pub fn controlled_2(m: &Matrix2) -> Matrix4 {
    [
        [c(1.0,0.0), c(0.0,0.0), c(0.0,0.0), c(0.0,0.0)],
        [c(0.0,0.0), c(1.0,0.0), c(0.0,0.0), c(0.0,0.0)],
        [c(0.0,0.0), c(0.0,0.0), m[0][0], m[0][1]],
        [c(0.0,0.0), c(0.0,0.0), m[1][0], m[1][1]]
    ]
}

pub fn neg_controlled_2(m: &Matrix2) -> Matrix4 {
    [
        [m[0][0], m[0][1], C64::new(0.0,0.0), C64::new(0.0,0.0)],
        [m[1][0], m[1][1], C64::new(0.0,0.0), C64::new(0.0,0.0)],
        [C64::new(0.0,0.0), C64::new(0.0,0.0), C64::new(1.0,0.0), C64::new(0.0,0.0)],
        [C64::new(0.0,0.0), C64::new(0.0,0.0), C64::new(0.0,0.0), C64::new(1.0,0.0)]
    ]
}

pub fn identity_4() -> Matrix4 {
    let mut m = [[c(0.0, 0.0); 4]; 4];
    for i in 0..4 { m[i][i] = c(1.0, 0.0); }
    m
}

pub fn conjugate_transpose_4(m: &Matrix4) -> Matrix4 {
    let mut result = [[c(0.0, 0.0); 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = m[j][i].conj();
        }
    }
    result
}

pub fn matrix_pow_4(m: &Matrix4, n: i64) -> Matrix4 {
    if n == 0 { return identity_4(); }
    if n < 0 { return matrix_pow_4(&conjugate_transpose_4(m), -n); }
    let mut result = identity_4();
    for _ in 0..n {
        result = matrix_mul_4(&result, m);
    }
    result
}

pub fn matrix_mul_4(a: &Matrix4, b: &Matrix4) -> Matrix4 {
    let mut result = [[c(0.0, 0.0); 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

// Permute a 4x4 matrix to swap qubit order from |q1,q0⟩ to |q0,q1⟩
// This swaps rows/columns according to the permutation [0,2,1,3]
pub fn swap_qubit_order(m: &Matrix4) -> Matrix4 {
    let perm = [0, 2, 1, 3];  // Maps |00⟩,|01⟩,|10⟩,|11⟩ to |00⟩,|10⟩,|01⟩,|11⟩
    let mut result = [[c(0.0, 0.0); 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = m[perm[i]][perm[j]];
        }
    }
    result
}

pub fn controlled_4(m: &Matrix4) -> Matrix8 {
    let mut result = identity_8();
    for i in 0..4 {
        for j in 0..4 {
            result[i + 4][j + 4] = m[i][j];
            result[i + 4][j] = C64::new(0.0, 0.0);
        }
    }
    result
}

pub fn neg_controlled_4(m: &Matrix4) -> Matrix8 {
    let mut result = identity_8();
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = m[i][j];
        }
    }
    result
}

pub fn identity_8() -> Matrix8 {
    let mut m = [[C64::new(0.0, 0.0); 8]; 8];
    for i in 0..8 { m[i][i] = C64::new(1.0, 0.0); }
    m
}

pub fn conjugate_transpose_8(m: &Matrix8) -> Matrix8 {
    let mut result = [[C64::new(0.0, 0.0); 8]; 8];
    for i in 0..8 {
        for j in 0..8 {
            result[i][j] = m[j][i].conj();
        }
    }
    result
}

pub fn matrix_pow_8(m: &Matrix8, n: i64) -> Matrix8 {
    if n == 0 { return identity_8(); }
    if n < 0 { return matrix_pow_8(&conjugate_transpose_8(m), -n); }
    let mut result = identity_8();
    for _ in 0..n {
        result = matrix_mul_8(&result, m);
    }
    result
}

pub fn matrix_mul_8(a: &Matrix8, b: &Matrix8) -> Matrix8 {
    let mut result = [[C64::new(0.0, 0.0); 8]; 8];
    for i in 0..8 {
        for j in 0..8 {
            for k in 0..8 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}