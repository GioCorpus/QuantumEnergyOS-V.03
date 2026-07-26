use core::fmt;

#[derive(Debug, Clone)]
pub enum ObjectiveFunction {
    Minimize(f64),
    Maximize(f64),
    Balance { left: f64, right: f64 },
}

#[derive(Debug, Clone, Copy)]
pub struct OptimizationResult {
    pub optimum: f64,
    pub iterations: usize,
    pub converged: bool,
}

#[derive(Debug, Clone)]
pub struct QuartzOptimizer {
    pub iterations: usize,
    pub learning_rate: f64,
}

impl QuartzOptimizer {
    pub const fn new(learning_rate: f64) -> Self {
        Self {
            iterations: 0,
            learning_rate,
        }
    }

    pub fn step(&mut self, current: f64, gradient: f64) -> f64 {
        let next = current - gradient * self.learning_rate;
        self.iterations += 1;
        next
    }

    pub fn optimize(
        &mut self,
        _objective: ObjectiveFunction,
        mut current: f64,
        gradient: fn(f64) -> f64,
        max_iter: usize,
        tolerance: f64,
    ) -> OptimizationResult {
        let mut converged = false;
        for i in 0..max_iter {
            let g = gradient(current);
            let next = self.step(current, g);
            let delta = (next - current).abs();
            current = next;
            if delta < tolerance {
                converged = true;
                self.iterations = i + 1;
                break;
            }
        }
        OptimizationResult {
            optimum: current,
            iterations: self.iterations,
            converged,
        }
    }
}

impl fmt::Display for OptimizationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OptimizationResult(optimum={:.6}, iterations={}, converged={})",
            self.optimum, self.iterations, self.converged
        )
    }
}
