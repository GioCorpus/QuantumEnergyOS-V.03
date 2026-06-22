use qeos_quantum::QuantumSimulator;

/// Creates a maximally entangled Bell pair |Φ+⟩ = (|00⟩ + |11⟩)/√2
///
/// [Production Ready]
pub fn create_bell_pair() -> (Vec<u8>, Vec<u8>) {
    let mut sim = QuantumSimulator::new(2);
    sim.apply_hadamard(0);
    sim.apply_cnot(0, 1);
    let m0 = sim.measure()[0];
    let m1 = sim.measure()[1];
    (vec![m0], vec![m1])
}

/// Implements the Deutsch-Jozsa algorithm
///
/// [Research Prototype]
pub struct DeutschJozsaSolver {
    n: usize,
}

impl DeutschJozsaSolver {
    pub fn new(n: usize) -> Self {
        Self { n }
    }

    pub fn solve(&self, oracle: impl Fn(usize) -> bool) -> (&'static str, &'static str) {
        let mut sim = QuantumSimulator::new(self.n + 1);
        sim.apply_hadamard(0);
        for i in 1..=self.n {
            sim.apply_hadamard(i);
        }
        for i in 0..(1 << self.n) {
            if oracle(i) {
                sim.apply_pauli_x(self.n);
            }
        }
        sim.apply_qft();
        for i in 0..self.n {
            sim.apply_hadamard(i);
        }
        let result = sim.measure();
        let is_constant = !result.iter().any(|&b| b == 1);
        if is_constant {
            ("constant", "balanced")
        } else {
            ("balanced", "constant")
        }
    }
}
