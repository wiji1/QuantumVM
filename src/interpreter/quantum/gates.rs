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
