#![warn(missing_docs)]

//! Quartz5D — Layer 1: Physical Layer
//!
//! Defines physical constants, state vectors, and dimensional models.
//! This layer does **not** introduce a 5th physical dimension; it models
//! standard 3D space + time + abstract system properties (coherence, entropy).
//!
//! # Classification
//!
//! [Research Prototype] — physical constants are approximations.

/// Dimensional axes modeled by Quartz5D.
///
/// Note: `Time`, `Entropy`, and `Coherence` are software abstractions,
/// not spacetime dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDimension {
    X,
    Y,
    Z,
    Time,
    Entropy,
    Coherence,
}

impl core::fmt::Display for PhysicalDimension {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::X => write!(f, "x"),
            Self::Y => write!(f, "y"),
            Self::Z => write!(f, "z"),
            Self::Time => write!(f, "time"),
            Self::Entropy => write!(f, "entropy"),
            Self::Coherence => write!(f, "coherence"),
        }
    }
}

/// Physical state of a point in the model.
#[derive(Debug, Clone, Copy, Default)]
pub struct PhysicalState {
    pub position: [f64; 3],
    pub momentum: [f64; 3],
    pub coherence: f64,
    pub entropy: f64,
}

impl PhysicalState {
    /// Creates a zero-initialized state.
    pub const fn zero() -> Self {
        Self {
            position: [0.0; 3],
            momentum: [0.0; 3],
            coherence: 0.0,
            entropy: 0.0,
        }
    }
}

impl core::fmt::Display for PhysicalState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PhysicalState(pos=[{:.2}, {:.2}, {:.2}], momentum=[{:.2}, {:.2}, {:.2}], coherence={:.4}, entropy={:.4})",
            self.position[0],
            self.position[1],
            self.position[2],
            self.momentum[0],
            self.momentum[1],
            self.momentum[2],
            self.coherence,
            self.entropy
        )
    }
}

/// Physical constants and model parameters.
///
/// All constants are in SI units unless otherwise noted.
#[derive(Debug, Clone, Copy)]
pub struct PhysicalModel {
    pub hbar: f64,
    pub c: f64,
    pub epsilon_0: f64,
    pub temperature_k: f64,
}

impl PhysicalModel {
    /// Standard physical constants (CODATA 2018 approximations).
    pub const fn new() -> Self {
        Self {
            hbar: 1.054_571_817e-34,
            c: 299_792_458.0,
            epsilon_0: 8.854_187_8128e-12,
            temperature_k: 0.0,
        }
    }

    /// Computes the de Broglie wavelength for a given momentum [kg·m/s].
    pub fn de_broglie_wavelength(&self, momentum: f64) -> f64 {
        self.hbar / momentum
    }
}

impl Default for PhysicalModel {
    fn default() -> Self {
        Self::new()
    }
}
