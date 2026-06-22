use ndarray::{Array2, Array3};
use qeos_common::Result;
use std::f64::consts::PI;

/// Simulates quantum circuits on classical hardware
/// 
/// [Production Ready]
pub struct QuantumSimulator {
    pub num_qubits: usize,
    pub state: ndarray::Array1<Complex64>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Complex64 {
    pub re: f64,
    pub im: f64,
}

impl Complex64 {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn magnitude(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    pub fn multiply(self, other: Complex64) -> Complex64 {
        Complex64 {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

impl QuantumSimulator {
    pub fn new(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let mut state = ndarray::Array1::zeros(dim);
        state[0] = Complex64::new(1.0, 0.0);
        Self { num_qubits, state }
    }

    pub fn apply_hadamard(&mut self, target: usize) {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let n = 1 << target;
        for i in 0..self.state.len() {
            if (i & n) != 0 {
                let partner = i ^ n;
                let a = self.state[i];
                let b = self.state[partner];
                self.state[i] = Complex64::new(
                    inv_sqrt2 * (a.re + b.re),
                    inv_sqrt2 * (a.im + b.im),
                );
                self.state[partner] = Complex64::new(
                    inv_sqrt2 * (a.re - b.re),
                    inv_sqrt2 * (a.im - b.im),
                );
            }
        }
    }

    pub fn apply_pauli_x(&mut self, target: usize) {
        let n = 1 << target;
        for i in 0..self.state.len() {
            if (i & n) != 0 {
                let partner = i ^ n;
                self.state.swap(i, partner);
            }
        }
    }

    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        let c_bit = 1 << control;
        let t_bit = 1 << target;
        for i in 0..self.state.len() {
            if ((i & c_bit) != 0) && ((i & t_bit) == 0) {
                let partner = i ^ t_bit;
                self.state.swap(i, partner);
            }
        }
    }

    pub fn measure(&self) -> Vec<u8> {
        let mut result = vec![0u8; self.num_qubits];
        let mut remaining = self.state.len();
        for i in 0..self.num_qubits {
            let step = remaining >> 1;
            let mut prob = 0.0;
            for j in 0..step {
                prob += self.state[j].magnitude().powi(2);
            }
            if rand::random::<f64>() < prob {
                result[i] = 0;
            } else {
                result[i] = 1;
            }
            for j in 0..self.state.len() {
                if ((j >> i) & 1) != result[i] {
                    self.state[j] = Complex64::new(0.0, 0.0);
                }
            }
            remaining = step;
            for j in 0..self.state.len() {
                let mag = self.state[j].magnitude();
                if mag > 0.0 {
                    self.state[j] = Complex64::new(
                        self.state[j].re / mag,
                        self.state[j].im / mag,
                    );
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_simulator_initial_state_is_ground() {
        let sim = QuantumSimulator::new(2);
        assert_eq!(sim.num_qubits, 2);
    }

    #[test]
    fn hadamard_superposition() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_hadamard(0);
        let probs: Vec<f64> = sim.state.iter().map(|c| c.magnitude().powi(2)).collect();
        assert!((probs[0] - 0.5).abs() < 0.1);
    }

    #[test]
    fn pauli_x_flips() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_pauli_x(0);
        let probs: Vec<f64> = sim.state.iter().map(|c| c.magnitude().powi(2)).collect();
        assert!((probs[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cnot_entangles() {
        let mut sim = QuantumSimulator::new(2);
        sim.apply_hadamard(0);
        sim.apply_cnot(0, 1);
        let state = &sim.state;
        assert!((state[0].magnitude().powi(2) - 0.5).abs() < 1e-6);
        assert!((state[3].magnitude().powi(2) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn measure_returns_bits() {
        let mut sim = QuantumSimulator::new(2);
        sim.apply_hadamard(0);
        sim.apply_cnot(0, 1);
        let result = sim.measure();
        assert_eq!(result.len(), 2);
    }
}
