/// Quantum Fourier Transform implementation
/// 
/// [Research Prototype]
pub struct QuantumFourierTransform;

impl QuantumFourierTransform {
    pub fn apply(sim: &mut impl QuantumOperations, num_qubits: usize) {
        for target in 0..num_qubits {
            sim.apply_hadamard(target);
            for control in (target + 1)..num_qubits {
                let angle = PI / (1 << (control - target + 1));
                sim.apply_cphase(control, target, angle);
            }
        }
        for i in 0..(num_qubits / 2) {
            sim.swap(i, num_qubits - 1 - i);
        }
    }
}

pub trait QuantumOperations {
    fn apply_hadamard(&mut self, target: usize);
    fn apply_cphase(&mut self, control: usize, target: usize, angle: f64);
    fn swap(&mut self, a: usize, b: usize);
}
