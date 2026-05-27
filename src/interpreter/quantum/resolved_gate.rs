use crate::interpreter::quantum::gates::*;
use crate::interpreter::quantum::statevector::StateVector;
use crate::interpreter::quantum::gates::swap_qubit_order;

pub enum ResolvedGate {
    SingleQubit(Matrix2),
    TwoQubit(Matrix4),
    ThreeQubit(Matrix8),
}

impl ResolvedGate {
    pub fn apply_inv(self) -> ResolvedGate {
        match self {
            ResolvedGate::SingleQubit(m) => ResolvedGate::SingleQubit(conjugate_transpose_2(&m)),
            ResolvedGate::TwoQubit(m) => ResolvedGate::TwoQubit(conjugate_transpose_4(&m)),
            ResolvedGate::ThreeQubit(m) => ResolvedGate::ThreeQubit(conjugate_transpose_8(&m)),
        }
    }

    pub fn apply_pow(self, n: i64) -> ResolvedGate {
        match self {
            ResolvedGate::SingleQubit(m) => ResolvedGate::SingleQubit(matrix_pow_2(&m, n)),
            ResolvedGate::TwoQubit(m) => ResolvedGate::TwoQubit(matrix_pow_4(&m, n)),
            ResolvedGate::ThreeQubit(m) => ResolvedGate::ThreeQubit(matrix_pow_8(&m, n)),
        }
    }

    pub fn apply_ctrl(self) -> ResolvedGate {
        match self {
            ResolvedGate::SingleQubit(m) => ResolvedGate::TwoQubit(controlled_2(&m)),
            ResolvedGate::TwoQubit(m) => ResolvedGate::ThreeQubit(controlled_4(&m)),
            ResolvedGate::ThreeQubit(_) => panic!("ctrl on 3-qubit gate not supported"),
        }
    }

    pub fn apply_neg_ctrl(self) -> ResolvedGate {
        match self {
            ResolvedGate::SingleQubit(m) => ResolvedGate::TwoQubit(neg_controlled_2(&m)),
            ResolvedGate::TwoQubit(m) => ResolvedGate::ThreeQubit(neg_controlled_4(&m)),
            ResolvedGate::ThreeQubit(_) => panic!("negctrl on 3-qubit gate not supported"),
        }
    }

    pub fn apply_to_statevector(&self, sv: &mut StateVector, qubits: &[usize]) {
        match self {
            ResolvedGate::SingleQubit(m) => sv.apply_single_qubit_gate(m, qubits[0]),
            ResolvedGate::TwoQubit(m) => {
                let matrix = if qubits[0] < qubits[1] { swap_qubit_order(m) }
                else { *m };
                sv.apply_two_qubit_gate(&matrix, qubits[0], qubits[1])
            },
            ResolvedGate::ThreeQubit(m) => sv.apply_three_qubit_gate(m, qubits[0], qubits[1], qubits[2]),
        }
    }
}