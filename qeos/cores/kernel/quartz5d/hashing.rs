use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
    QuantumSecure { qubits: usize },
}

#[derive(Debug, Clone, Copy)]
pub struct IntegrityProof {
    pub algorithm: HashAlgorithm,
    pub digest: [u8; 32],
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone)]
pub struct QuartzHasher {
    pub algorithm: HashAlgorithm,
}

impl QuartzHasher {
    pub const fn new(algorithm: HashAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn hash(&self, data: &[u8]) -> IntegrityProof {
        let mut digest = [0u8; 32];
        let len = data.len().min(32);
        digest[..len].copy_from_slice(&data[..len]);
        IntegrityProof {
            algorithm: self.algorithm,
            digest,
            timestamp_ns: 0,
        }
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sha256 => write!(f, "sha256"),
            Self::Blake3 => write!(f, "blake3"),
            Self::QuantumSecure { qubits } => write!(f, "quantum-secure({}q)", qubits),
        }
    }
}
