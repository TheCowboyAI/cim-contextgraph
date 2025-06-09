//! Graph morphisms and category theory operations

use crate::context_graph::ContextGraph;
use crate::types::GraphResult;

/// Trait for functors over graphs
pub trait Functor<N1, E1, N2, E2> {
    fn fmap(&self, graph: &ContextGraph<N1, E1>) -> GraphResult<ContextGraph<N2, E2>>;
}

/// Trait for monads over graphs
pub trait Monad<N, E> {
    fn pure(value: N) -> ContextGraph<N, E>;
    fn bind<F, N2, E2>(&self, f: F) -> GraphResult<ContextGraph<N2, E2>>
    where
        F: Fn(&N) -> ContextGraph<N2, E2>;
}

/// Trait for graph morphisms
pub trait GraphMorphism<N1, E1, N2, E2> {
    fn apply(&self, graph: &ContextGraph<N1, E1>) -> GraphResult<ContextGraph<N2, E2>>;
}
