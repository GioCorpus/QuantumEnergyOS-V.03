use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumState {
    pub qubits: Vec<Qubit>,
    pub coherence_time_ms: f64,
    pub gate_fidelity: f64,
    pub topology: Topology,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qubit {
    pub id: u32,
    pub state: Complex,
    pub error_rate: f64,
    pub t1_time: f64,
    pub t2_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub gate_type: GateType,
    pub qubit_count: u32,
    pub connectivity: ConnectivityMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityMap {
    pub edges: Vec<(u32, u32)>,
    pub distance_matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateType {
    SurfaceCode,
    LatticeSurgery,
    Majorana,
    Transmon,
    TrappedIon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCircuit {
    pub id: uuid::Uuid,
    pub name: String,
    pub qubits: u32,
    pub depth: u32,
    pub gates: Vec<QuantumGate>,
    pub shots: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumGate {
    pub kind: String,
    pub targets: Vec<u32>,
    pub parameters: Vec<f64>,
    pub duration_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumResult {
    pub circuit_id: uuid::Uuid,
    pub success: bool,
    pub counts: std::collections::HashMap<String, u64>,
    pub execution_time_ms: u64,
    pub fidelity: Option<f64>,
    pub error: Option<String>,
}
