//! Graph invariants that can be enforced on ContextGraphs

use crate::context_graph::{ContextGraph, GraphInvariant};
use crate::types::{GraphResult, GraphError, Node, Edge};

/// Invariant that ensures the graph is acyclic
pub struct Acyclic;

impl GraphInvariant<Node, Edge> for Acyclic {
    fn check(&self, _graph: &ContextGraph<Node, Edge>) -> GraphResult<()> {
        // TODO: Implement cycle detection
        Ok(())
    }

    fn name(&self) -> &str {
        "Acyclic"
    }
}

/// Invariant that ensures the graph is connected
pub struct Connected;

impl GraphInvariant<Node, Edge> for Connected {
    fn check(&self, _graph: &ContextGraph<Node, Edge>) -> GraphResult<()> {
        // TODO: Implement connectivity check
        Ok(())
    }

    fn name(&self) -> &str {
        "Connected"
    }
}
