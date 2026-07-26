use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CognitiveNode {
    pub id: u64,
    pub activation: f32,
    pub bias: f32,
}

#[derive(Debug, Clone, Default)]
pub struct NeuralGraph {
    pub nodes: Vec<CognitiveNode>,
    pub edges: Vec<(u64, u64, f32)>,
}

#[derive(Debug, Clone, Copy)]
pub struct InferenceResult {
    pub confidence: f32,
    pub label: u32,
    pub latency_ns: u64,
}

impl NeuralGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: CognitiveNode) {
        self.nodes.push(node);
    }

    pub fn connect(&mut self, from: u64, to: u64, weight: f32) {
        self.edges.push((from, to, weight));
    }

    pub fn propagate(&self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            output.push(node.activation + input.get(node.id as usize).copied().unwrap_or(0.0));
        }
        output
    }
}

impl fmt::Display for NeuralGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NeuralGraph(nodes={}, edges={})",
            self.nodes.len(),
            self.edges.len()
        )
    }
}
