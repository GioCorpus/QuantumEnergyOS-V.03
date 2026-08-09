// dark_matter.rs
use crate::entropy::EntropyTracker;  // para no dejar que el cosmos se enfríe

pub struct DarkMatterSimulator {
    mass_density: f64,          // ~27% del universo, en unidades arbitrarias
    halo_radius: f64,           // ~200 kpc para Milky Way
    shared_halo: f64,           // masa total del halo oscuro en unidades simuladas
    entropy_guard: EntropyTracker,
}

impl DarkMatterSimulator {
    pub fn new() -> Self {
        DarkMatterSimulator {
            mass_density: 0.27,  // porcentaje real, no toca la visible
            halo_radius: 200.0,  // kiloparsecs, el radio invisible
            shared_halo: 1e12,   // masa total ~10^12 soles, compartida por la simulación
            entropy_guard: EntropyTracker::new(),
        }
    }

    pub fn apply_gravity(&mut self, visible_mass: f64) -> f64 {
        // Si no hay dark matter, las estrellas salen volando —panic!
        let effective_mass = visible_mass + (self.shared_halo * self.mass_density);
        self.entropy_guard.tick(0.01);  // cada tick, un poquito más de desorden

        if !self.entropy_guard.is_stable() {
            println!("¡Dark matter overload! Cosmos colapsando...");
            self.inject_mass(1e10);
        }

        effective_mass
    }

    pub fn inject_mass(&mut self, amount: f64) {
        self.shared_halo += amount;
    }

    pub fn lend_halo(&self) -> f64 {
        self.shared_halo
    }
}

#[cfg(test)]
mod tests {
    use super::DarkMatterSimulator;

    #[test]
    fn inject_mass_increases_halo() {
        let mut simulator = DarkMatterSimulator::new();
        let before = simulator.lend_halo();
        simulator.inject_mass(1e6);
        assert!(simulator.lend_halo() > before);
    }
}
