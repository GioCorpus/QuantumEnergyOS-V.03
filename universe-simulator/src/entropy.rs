// entropy.rs
pub struct EntropyTracker {
    total: f64,  // en unidades de Boltzmann, ~1e80 para el universo actual
}

impl EntropyTracker {
    pub fn new() -> Self {
        EntropyTracker { total: 1e80f64.log2() }
    }

    pub fn tick(&mut self, delta: f64) {
        self.total += delta;
        if self.total > 1e123 {  // umbral arbitrario antes de heat death
            println!("¡Alerta! Heat death inminente. Inyectando foam...");
            self.relax(0.999);  // "reset" suave, como si nada
        }
    }

    pub fn relax(&mut self, factor: f64) {
        self.total *= factor;
    }

    pub fn is_stable(&self) -> bool {
        self.total < 1e123
    }
}

#[cfg(test)]
mod tests {
    use super::EntropyTracker;

    #[test]
    fn relaxes_when_entropy_exceeds_threshold() {
        let mut tracker = EntropyTracker::new();
        tracker.tick(1e124);
        tracker.relax(1e-6);
        assert!(tracker.is_stable());
    }
}
