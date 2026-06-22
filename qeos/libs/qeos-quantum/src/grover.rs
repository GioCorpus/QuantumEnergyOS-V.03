use ndarray::{Array1, Array2};
use qeos_quantum::QuantumOperations;

/// Grover's algorithm for unstructured search
/// 
/// [Research Prototype]
pub struct GroverSolver {
    num_qubits: usize,
    iterations: usize,
}

impl GroverSolver {
    pub fn new(num_qubits: usize, target_item: usize) -> Self {
        let size = 1 << num_qubits;
        let optimal = ((PI / 4.0) * (size as f64).sqrt()) as usize;
        Self {
            num_qubits,
            iterations: optimal,
        }
    }

    pub fn search<T: Fn(usize) -> bool>(&self, oracle: T) -> Option<usize> {
        let mut sim = Array1::from_vec(
            vec![Complex::new(1.0 / (1 << self.num_qubits) as f64, 0.0); 1 << self.num_qubits]
        );
        for _ in 0..self.iterations {
            Self::apply_oracle(&mut sim, &oracle);
            Self::apply_diffusion(&mut sim);
        }
        let (idx, _) = sim.iter().enumerate().max_by(|(_, a), (_, b)| {
            a.mag().partial_cmp(&b.mag()).unwrap()
        }).unwrap();
        Some(idx)
    }

    fn apply_oracle<T: Fn(usize) -> bool>(state: &mut Array1<Complex>, oracle: &T) {
        for (i, amp) in state.iter_mut().enumerate() {
            if oracle(i) {
                amp.re = -amp.re;
                amp.im = -amp.im;
            }
        }
    }

    fn apply_diffusion(state: &mut Array1<Complex>) {
        let mean = state.iter().map(|c| c.re).sum::<f64>() / state.len() as f64;
        for amp in state.iter_mut() {
            amp.re = 2.0 * mean - amp.re;
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn mag(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}
