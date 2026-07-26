#![warn(missing_docs)]

//! Quartz5D — Layer 2: Topological Layer
//!
//! Models the graph structure of the system: nodes, edges, adjacency.
//!
//! # Classification
//!
//! [Research Prototype]

/// Unique vertex identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Vertex(pub u64);

impl Vertex {
    /// Creates a new vertex.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

impl core::fmt::Display for Vertex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "V{}", self.0)
    }
}

/// Directed or undirected edge between two vertices.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Edge(pub Vertex, pub Vertex);

impl Edge {
    /// Creates a new edge.
    pub const fn new(from: Vertex, to: Vertex) -> Self {
        Self(from, to)
    }
}

/// Topology graph.
///
/// Stores edges in a flat vector for cache-friendly traversal.
/// For sparse graphs with >10k edges, consider an adjacency-list representation.
#[derive(Debug, Clone, Default)]
pub struct TopologyGraph {
    edges: Vec<Edge>,
    node_count: usize,
}

impl TopologyGraph {
    /// Creates an empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, from: Vertex, to: Vertex) {
        self.edges.push(Edge::new(from, to));
        self.node_count = self
            .node_count
            .max(from.0.max(to.0) as usize + 1);
    }

    /// Returns the number of edges incident to `vertex`.
    pub fn neighbor_count(&self, vertex: Vertex) -> usize {
        self.edges
            .iter()
            .filter(|e| e.0 == vertex || e.1 == vertex)
            .count()
    }

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl core::fmt::Display for TopologyGraph {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "TopologyGraph(nodes={}, edges={})",
            self.node_count,
            self.edges.len()
        )
    }
}
