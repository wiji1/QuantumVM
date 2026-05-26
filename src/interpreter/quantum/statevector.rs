use crate::interpreter::quantum::gates::{Matrix2, Matrix4, Matrix8};
use num_complex::Complex;

pub type C64 = Complex<f64>;

#[derive(Clone, Debug)]
pub struct StateVector {
    pub amplitudes: Vec<C64>,
    pub num_qubits: usize,
}

impl StateVector {
    pub fn new(num_qubits: usize) -> Self {
        let size = (2usize).pow(num_qubits as u32);
        let mut amplitudes = vec![C64::new(0.0, 0.0); size];
        amplitudes[0] = C64::new(1.0, 0.0);
        StateVector { amplitudes, num_qubits }
    }

    pub fn num_states(&self) -> usize {
        self.amplitudes.len()
    }

    pub fn probability(&self, index: usize) -> f64 {
        self.amplitudes[index].norm_sqr()
    }

    pub fn total_probability(&self) -> f64 {
        self.amplitudes.iter().map(|a| a.norm_sqr()).sum()
    }

    pub fn renormalize(&mut self) {
        let total = self.total_probability().sqrt();
        for amp in &mut self.amplitudes {
            *amp /= total;
        }
    }

    pub fn measure_qubit(&mut self, qubit: usize) -> bool {
        let prob_one: f64 = self.amplitudes.iter().enumerate()
            .filter(|(state, _)| (state >> qubit) & 1 == 1)
            .map(|(_, amp)| amp.norm_sqr())
            .sum();

        let outcome = rand::random::<f64>() < prob_one;

        for state in 0..self.amplitudes.len() {
            let bit = (state >> qubit) & 1;
            if (bit == 1) != outcome {
                self.amplitudes[state] = C64::new(0.0, 0.0);
            }
        }

        self.renormalize();

        outcome
    }

    pub fn apply_single_qubit_gate(&mut self, gate: &Matrix2, qubit: usize) {
        let size = self.amplitudes.len();

        for state in 0..size {
            if (state >> qubit) & 1 == 1 { continue; }

            let partner = state | (1 << qubit);

            let a0 = self.amplitudes[state];
            let a1 = self.amplitudes[partner];

            self.amplitudes[state] = gate[0][0] * a0 + gate[0][1] * a1;
            self.amplitudes[partner] = gate[1][0] * a0 + gate[1][1] * a1;
        }
    }

    pub fn apply_two_qubit_gate(&mut self, gate: &Matrix4, control: usize, target: usize) {
        let size = self.amplitudes.len();

        for state in 0..size {
            if (state >> control) & 1 == 1 { continue; }
            if (state >> target) & 1 == 1 { continue; }

            let s00 = state;
            let s01 = state | (1 << target);
            let s10 = state | (1 << control);
            let s11 = state | (1 << control) | (1 << target);

            let a00 = self.amplitudes[s00];
            let a01 = self.amplitudes[s01];
            let a10 = self.amplitudes[s10];
            let a11 = self.amplitudes[s11];

            self.amplitudes[s00] = gate[0][0]*a00 + gate[0][1]*a01 + gate[0][2]*a10 + gate[0][3]*a11;
            self.amplitudes[s01] = gate[1][0]*a00 + gate[1][1]*a01 + gate[1][2]*a10 + gate[1][3]*a11;
            self.amplitudes[s10] = gate[2][0]*a00 + gate[2][1]*a01 + gate[2][2]*a10 + gate[2][3]*a11;
            self.amplitudes[s11] = gate[3][0]*a00 + gate[3][1]*a01 + gate[3][2]*a10 + gate[3][3]*a11;
        }
    }

    pub fn apply_three_qubit_gate(&mut self, gate: &Matrix8, q0: usize, q1: usize, q2: usize) {
        let size = self.amplitudes.len();

        for state in 0..size {
            if (state >> q0) & 1 == 1 { continue; }
            if (state >> q1) & 1 == 1 { continue; }
            if (state >> q2) & 1 == 1 { continue; }

            let s000 = state;
            let s001 = state | (1 << q2);
            let s010 = state | (1 << q1);
            let s011 = state | (1 << q1) | (1 << q2);
            let s100 = state | (1 << q0);
            let s101 = state | (1 << q0) | (1 << q2);
            let s110 = state | (1 << q0) | (1 << q1);
            let s111 = state | (1 << q0) | (1 << q1) | (1 << q2);

            let indices = [s000, s001, s010, s011, s100, s101, s110, s111];
            let amps: Vec<C64> = indices.iter().map(|&i| self.amplitudes[i]).collect();

            for (row, &idx) in indices.iter().enumerate() {
                self.amplitudes[idx] = (0..8).map(|col| gate[row][col] * amps[col]).sum();
            }
        }
    }
}