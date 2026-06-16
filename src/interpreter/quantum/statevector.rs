use crate::interpreter::quantum::gates::{Matrix2, Matrix4, Matrix8};
use num_complex::Complex;
use rayon::prelude::*;

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
        let total_prob = self.total_probability();

        //TODO: Determine if this is necessary
        if total_prob <= 1e-10 || !total_prob.is_finite() {
            eprintln!("WARNING: Invalid statevector normalization (total prob = {})", total_prob);
            eprintln!("Resetting to |0⟩ state");
            self.amplitudes[0] = C64::new(1.0, 0.0);
            for i in 1..self.amplitudes.len() {
                self.amplitudes[i] = C64::new(0.0, 0.0);
            }
            return;
        }

        let norm = total_prob.sqrt();
        for amp in &mut self.amplitudes {
            *amp /= norm;
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

    pub fn reset_qubit(&mut self, qubit: usize) {
        for state in 0..self.amplitudes.len() {
            if (state >> qubit) & 1 == 1 { self.amplitudes[state] = C64::new(0.0, 0.0); }
        }
        self.renormalize();
    }

    pub fn apply_single_qubit_gate(&mut self, gate: &Matrix2, qubit: usize) {
        let size = self.amplitudes.len();
        let half_size = size >> 1;
        let low_mask = (1 << qubit) - 1;
        let high_mask = !low_mask;

        let updates: Vec<_> = (0..half_size).into_par_iter().map(|i| {
            let low = i & low_mask;
            let high = (i & high_mask) << 1;
            let state = low | high;
            let partner = state | (1 << qubit);

            let a0 = self.amplitudes[state];
            let a1 = self.amplitudes[partner];

            let new_state = gate[0][0] * a0 + gate[0][1] * a1;
            let new_partner = gate[1][0] * a0 + gate[1][1] * a1;

            (state, partner, new_state, new_partner)
        }).collect();

        for (state, partner, new_state, new_partner) in updates {
            self.amplitudes[state] = new_state;
            self.amplitudes[partner] = new_partner;
        }
    }

    pub fn apply_two_qubit_gate(&mut self, gate: &Matrix4, qubit0: usize, qubit1: usize) {
        let size = self.amplitudes.len();
        let (q0_bit, q1_bit) = if qubit0 < qubit1 { (qubit0, qubit1) } else
        { (qubit1, qubit0) };

        let quarter_size = size >> 2;

        let updates: Vec<_> = (0..quarter_size).into_par_iter().map(|i| {
            let low = i & ((1 << q0_bit) - 1);
            let mid = (i >> q0_bit) & ((1 << (q1_bit - q0_bit - 1)) - 1);
            let high = i >> (q1_bit - 1);

            let s00 = low | (mid << (q0_bit + 1)) | (high << (q1_bit + 1));
            let s01 = s00 | (1 << q0_bit);
            let s10 = s00 | (1 << q1_bit);
            let s11 = s00 | (1 << q0_bit) | (1 << q1_bit);

            let a00 = self.amplitudes[s00];
            let a01 = self.amplitudes[s01];
            let a10 = self.amplitudes[s10];
            let a11 = self.amplitudes[s11];

            let new_00 = gate[0][0]*a00 + gate[0][1]*a01 + gate[0][2]*a10 + gate[0][3]*a11;
            let new_01 = gate[1][0]*a00 + gate[1][1]*a01 + gate[1][2]*a10 + gate[1][3]*a11;
            let new_10 = gate[2][0]*a00 + gate[2][1]*a01 + gate[2][2]*a10 + gate[2][3]*a11;
            let new_11 = gate[3][0]*a00 + gate[3][1]*a01 + gate[3][2]*a10 + gate[3][3]*a11;

            (s00, s01, s10, s11, new_00, new_01, new_10, new_11)
        }).collect();

        for (s00, s01, s10, s11, new_00, new_01, new_10, new_11) in updates {
            self.amplitudes[s00] = new_00;
            self.amplitudes[s01] = new_01;
            self.amplitudes[s10] = new_10;
            self.amplitudes[s11] = new_11;
        }
    }

    pub fn apply_three_qubit_gate(&mut self, gate: &Matrix8, q0: usize, q1: usize, q2: usize) {
        let size = self.amplitudes.len();
        let eighth_size = size >> 3;

        let mut qubits = [q0, q1, q2];
        qubits.sort_unstable();
        let [qa, qb, qc] = qubits;

        for i in 0..eighth_size {
            let low = i & ((1 << qa) - 1);
            let mid1 = (i >> qa) & ((1 << (qb - qa - 1)) - 1);
            let mid2 = (i >> (qb - 1)) & ((1 << (qc - qb - 1)) - 1);
            let high = i >> (qc - 2);

            let s000 = low | (mid1 << (qa + 1)) | (mid2 << (qb + 1)) | (high << (qc + 1));
            let s001 = s000 | (1 << q2);
            let s010 = s000 | (1 << q1);
            let s011 = s000 | (1 << q1) | (1 << q2);
            let s100 = s000 | (1 << q0);
            let s101 = s000 | (1 << q0) | (1 << q2);
            let s110 = s000 | (1 << q0) | (1 << q1);
            let s111 = s000 | (1 << q0) | (1 << q1) | (1 << q2);

            let indices = [s000, s001, s010, s011, s100, s101, s110, s111];
            let amps: Vec<C64> = indices.iter().map(|&i| self.amplitudes[i]).collect();

            for (row, &idx) in indices.iter().enumerate() {
                self.amplitudes[idx] = (0..8).map(|col| gate[row][col] * amps[col]).sum();
            }
        }
    }
}