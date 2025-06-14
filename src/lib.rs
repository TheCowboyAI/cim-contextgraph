//! CIM-ContextGraph: Fundamental graph abstractions for the Composable Information Machine
//!
//! This crate provides:
//! - **ContextGraph**: The base graph theory abstraction that can represent ANY graph
//! - **ConceptGraph**: A compositional type that combines multiple ContextGraphs
//! - **Component System**: Nodes and edges can be any typed value with components attached
//! - **Recursive Composition**: Graphs containing graphs, enabling fractal structures
//!
//! # Core Philosophy
//!
//! Everything in CIM is a graph. Whether it's a simple value object, a complex aggregate,
//! or an entire bounded context - they're all represented as ContextGraphs with different
//! shapes and invariants.
//!
//! ConceptGraphs compose these ContextGraphs to create higher-level abstractions,
//! and these can be recursively composed to create superconcepts.
//!
//! # Example
//!
//! ```rust,ignore
//! use cim_contextgraph::{ContextGraph, Label};
//!
//! // Create a graph with string nodes and integer edge weights
//! let mut graph = ContextGraph::<String, i32>::new("MyGraph");
//!
//! // Add nodes (can be primitives)
//! let n1 = graph.add_node("Hello".to_string());
//! let n2 = graph.add_node("World".to_string());
//!
//! // Add edge with weight
//! let edge = graph.add_edge(n1, n2, 42).unwrap();
//!
//! // Attach components to nodes and edges
//! graph.get_node_mut(n1).unwrap()
//!     .add_component(Label("Greeting".to_string()));
//! ```

pub mod context_graph;
pub mod types;
pub mod invariants;
pub mod composition;

// TODO: These modules will be implemented next
// pub mod morphisms;

// Re-export core types
pub use context_graph::{ContextGraph, GraphInvariant};
pub use types::{
    Component, ComponentStorage,
    NodeEntry, EdgeEntry,
    NodeId, EdgeId, ContextGraphId, ConceptGraphId,
    Label, Metadata, GraphReference, Subgraph,
    GraphError, GraphResult,
};
pub use invariants::{Acyclic, Connected};
pub use composition::{compose, union, intersection, product};
