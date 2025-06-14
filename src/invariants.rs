//! Graph invariants that can be enforced on ContextGraphs

use crate::context_graph::{ContextGraph, GraphInvariant};
use crate::types::GraphResult;

/// Invariant that ensures the graph is acyclic
#[derive(Clone)]
pub struct Acyclic;

impl<N, E> GraphInvariant<N, E> for Acyclic {
    fn check(&self, _graph: &ContextGraph<N, E>) -> GraphResult<()> {
        // TODO: Implement cycle detection
        Ok(())
    }

    fn name(&self) -> &str {
        "Acyclic"
    }

    fn clone_box(&self) -> Box<dyn GraphInvariant<N, E>> {
        Box::new(self.clone())
    }


}

/// Invariant that ensures the graph is connected
#[derive(Clone)]
pub struct Connected;

impl<N, E> GraphInvariant<N, E> for Connected {
    fn check(&self, _graph: &ContextGraph<N, E>) -> GraphResult<()> {
        // TODO: Implement connectivity check
        Ok(())
    }

    fn name(&self) -> &str {
        "Connected"
    }

    fn clone_box(&self) -> Box<dyn GraphInvariant<N, E>> {
        Box::new(self.clone())
    }


}
