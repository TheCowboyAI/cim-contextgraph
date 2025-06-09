//! ContextGraph - The fundamental graph abstraction that can represent ANY graph
//!
//! This is the base building block of the entire CIM system. Every domain concept,
//! from simple values to complex aggregates, is represented as a ContextGraph.
//!
//! Nodes and edges can be any typed value (including primitives) with components attached.

use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// The fundamental graph abstraction - can represent ANY graph structure
///
/// N: The node value type (can be any type, including primitives)
/// E: The edge value type (can be any type, including primitives)
#[derive(Debug, Clone)]
pub struct ContextGraph<N, E> {
    pub id: ContextGraphId,
    pub nodes: HashMap<NodeId, NodeEntry<N>>,
    pub edges: HashMap<EdgeId, EdgeEntry<E>>,
    pub metadata: Metadata,

    #[serde(skip)]
    pub invariants: Vec<Box<dyn GraphInvariant<N, E>>>,
}

/// Trait for graph invariants that must be maintained
pub trait GraphInvariant<N, E>: Send + Sync {
    fn check(&self, graph: &ContextGraph<N, E>) -> GraphResult<()>;
    fn name(&self) -> &str;
}

impl<N, E> ContextGraph<N, E>
where
    N: Clone + Debug,
    E: Clone + Debug,
{
    /// Create a new empty graph
    pub fn new(name: impl Into<String>) -> Self {
        let mut metadata = Metadata::default();
        metadata.properties.insert("name".to_string(), serde_json::json!(name.into()));

        Self {
            id: ContextGraphId::new(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            metadata,
            invariants: Vec::new(),
        }
    }

    /// Add a node with a value
    pub fn add_node(&mut self, value: N) -> NodeId {
        let node = NodeEntry::new(value);
        let id = node.id;
        self.nodes.insert(id, node);
        id
    }

    /// Add a node with a specific ID
    pub fn add_node_with_id(&mut self, id: NodeId, value: N) -> NodeId {
        let node = NodeEntry::with_id(id, value);
        self.nodes.insert(id, node);
        id
    }

    /// Get a node entry by ID
    pub fn get_node(&self, id: NodeId) -> Option<&NodeEntry<N>> {
        self.nodes.get(&id)
    }

    /// Get a mutable node entry by ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut NodeEntry<N>> {
        self.nodes.get_mut(&id)
    }

    /// Get just the node value by ID
    pub fn get_node_value(&self, id: NodeId) -> Option<&N> {
        self.nodes.get(&id).map(|entry| &entry.value)
    }

    /// Get a mutable node value by ID
    pub fn get_node_value_mut(&mut self, id: NodeId) -> Option<&mut N> {
        self.nodes.get_mut(&id).map(|entry| &mut entry.value)
    }

    /// Remove a node and all connected edges
    pub fn remove_node(&mut self, id: NodeId) -> Option<NodeEntry<N>> {
        // Remove all edges connected to this node
        self.edges.retain(|_, edge| {
            edge.source != id && edge.target != id
        });

        self.nodes.remove(&id)
    }

    /// Add an edge between two nodes with a value
    pub fn add_edge(&mut self, source: NodeId, target: NodeId, value: E) -> GraphResult<EdgeId> {
        // Check that both nodes exist
        if !self.nodes.contains_key(&source) {
            return Err(GraphError::NodeNotFound(source));
        }
        if !self.nodes.contains_key(&target) {
            return Err(GraphError::NodeNotFound(target));
        }

        let edge = EdgeEntry::new(source, target, value);
        let id = edge.id;
        self.edges.insert(id, edge);

        // Check invariants after modification
        self.check_invariants()?;

        Ok(id)
    }

    /// Get an edge entry by ID
    pub fn get_edge(&self, id: EdgeId) -> Option<&EdgeEntry<E>> {
        self.edges.get(&id)
    }

    /// Get a mutable edge entry by ID
    pub fn get_edge_mut(&mut self, id: EdgeId) -> Option<&mut EdgeEntry<E>> {
        self.edges.get_mut(&id)
    }

    /// Get just the edge value by ID
    pub fn get_edge_value(&self, id: EdgeId) -> Option<&E> {
        self.edges.get(&id).map(|entry| &entry.value)
    }

    /// Get a mutable edge value by ID
    pub fn get_edge_value_mut(&mut self, id: EdgeId) -> Option<&mut E> {
        self.edges.get_mut(&id).map(|entry| &mut entry.value)
    }

    /// Remove an edge
    pub fn remove_edge(&mut self, id: EdgeId) -> Option<EdgeEntry<E>> {
        self.edges.remove(&id)
    }

    /// Get all edges from a node
    pub fn edges_from(&self, node: NodeId) -> Vec<&EdgeEntry<E>> {
        self.edges
            .values()
            .filter(|edge| edge.source == node)
            .collect()
    }

    /// Get all edges to a node
    pub fn edges_to(&self, node: NodeId) -> Vec<&EdgeEntry<E>> {
        self.edges
            .values()
            .filter(|edge| edge.target == node)
            .collect()
    }

    /// Get the degree of a node (in + out)
    pub fn degree(&self, node: NodeId) -> usize {
        self.edges_from(node).len() + self.edges_to(node).len()
    }

    /// Add an invariant that must be maintained
    pub fn add_invariant(&mut self, invariant: Box<dyn GraphInvariant<N, E>>) {
        self.invariants.push(invariant);
    }

    /// Check all invariants
    pub fn check_invariants(&self) -> GraphResult<()> {
        for invariant in &self.invariants {
            invariant.check(self)?;
        }
        Ok(())
    }

    /// Query nodes by component type
    pub fn query_nodes_with_component<T: Component + 'static>(&self) -> Vec<(&NodeId, &NodeEntry<N>)> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.components.has::<T>())
            .collect()
    }

    /// Query edges by component type
    pub fn query_edges_with_component<T: Component + 'static>(&self) -> Vec<(&EdgeId, &EdgeEntry<E>)> {
        self.edges
            .iter()
            .filter(|(_, edge)| edge.components.has::<T>())
            .collect()
    }

    /// Get all nodes that contain subgraphs (for recursive traversal)
    pub fn get_subgraph_nodes(&self) -> Vec<(&NodeId, &NodeEntry<N>)> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.components.has::<Subgraph<N, E>>())
            .collect()
    }

    /// Count total nodes including nodes in subgraphs (recursive)
    pub fn total_node_count(&self) -> usize {
        let mut count = self.nodes.len();

        for (_, node) in self.get_subgraph_nodes() {
            if let Some(subgraph) = node.get_component::<Subgraph<N, E>>() {
                count += subgraph.graph.total_node_count();
            }
        }

        count
    }

    /// Find all paths between two nodes
    pub fn find_paths(&self, start: NodeId, end: NodeId) -> Vec<Vec<NodeId>> {
        let mut paths = Vec::new();
        let mut current_path = vec![start];
        let mut visited = HashMap::new();

        self.dfs_paths(start, end, &mut current_path, &mut visited, &mut paths);

        paths
    }

    fn dfs_paths(
        &self,
        current: NodeId,
        target: NodeId,
        path: &mut Vec<NodeId>,
        visited: &mut HashMap<NodeId, bool>,
        paths: &mut Vec<Vec<NodeId>>,
    ) {
        if current == target {
            paths.push(path.clone());
            return;
        }

        visited.insert(current, true);

        // Find all edges from current node
        for edge in self.edges_from(current) {
            if !visited.get(&edge.target).unwrap_or(&false) {
                path.push(edge.target);
                self.dfs_paths(edge.target, target, path, visited, paths);
                path.pop();
            }
        }

        visited.insert(current, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_nodes_and_edges() {
        // Graph with string nodes and f64 edge weights
        let mut graph = ContextGraph::<String, f64>::new("StringGraph");

        let node1 = graph.add_node("Hello".to_string());
        let node2 = graph.add_node("World".to_string());

        // Add edge with weight
        let edge = graph.add_edge(node1, node2, 1.5).unwrap();

        // Add components to nodes
        graph.get_node_mut(node1).unwrap()
            .add_component(Label("Greeting".to_string()));

        graph.get_node_mut(node2).unwrap()
            .add_component(Label("Target".to_string()));

        // Add component to edge
        graph.get_edge_mut(edge).unwrap()
            .add_component(Label("Connection".to_string()));

        // Check values
        assert_eq!(graph.get_node_value(node1).unwrap(), "Hello");
        assert_eq!(graph.get_edge_value(edge).unwrap(), &1.5);
    }

    #[test]
    fn test_integer_graph() {
        // Graph with integer nodes and edges
        let mut graph = ContextGraph::<i32, i32>::new("IntegerGraph");

        let n1 = graph.add_node(10);
        let n2 = graph.add_node(20);
        let n3 = graph.add_node(30);

        graph.add_edge(n1, n2, 1).unwrap();
        graph.add_edge(n2, n3, 2).unwrap();

        // Add metadata to nodes
        graph.get_node_mut(n1).unwrap()
            .add_component(Metadata {
                description: Some("First number".to_string()),
                tags: vec!["start".to_string()],
                properties: serde_json::Map::new(),
            });

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_bool_graph() {
        // Graph with boolean nodes and unit edges
        let mut graph = ContextGraph::<bool, ()>::new("BooleanGraph");

        let true_node = graph.add_node(true);
        let false_node = graph.add_node(false);

        // Edge with unit type
        graph.add_edge(true_node, false_node, ()).unwrap();

        assert_eq!(graph.get_node_value(true_node).unwrap(), &true);
        assert_eq!(graph.get_node_value(false_node).unwrap(), &false);
    }

    #[test]
    fn test_recursive_typed_graph() {
        // Outer graph with string nodes
        let mut outer = ContextGraph::<String, String>::new("OuterGraph");

        // Inner graph with integer nodes
        let mut inner = ContextGraph::<i32, i32>::new("InnerGraph");
        let i1 = inner.add_node(100);
        let i2 = inner.add_node(200);
        inner.add_edge(i1, i2, 50).unwrap();

        // Add string node that contains the integer graph
        let container = outer.add_node("Container".to_string());
        outer.get_node_mut(container).unwrap()
            .add_component(Subgraph { graph: Box::new(inner) });

        let regular = outer.add_node("Regular".to_string());
        outer.add_edge(container, regular, "contains".to_string()).unwrap();

        // Verify structure
        let subgraph_nodes = outer.get_subgraph_nodes();
        assert_eq!(subgraph_nodes.len(), 1);
        assert_eq!(outer.total_node_count(), 4); // 2 outer + 2 inner
    }

    #[test]
    fn test_component_queries() {
        let mut graph = ContextGraph::<&str, u32>::new("StaticStrGraph");

        let n1 = graph.add_node("first");
        let n2 = graph.add_node("second");
        let n3 = graph.add_node("third");

        // Add labels to some nodes
        graph.get_node_mut(n1).unwrap()
            .add_component(Label("Important".to_string()));
        graph.get_node_mut(n3).unwrap()
            .add_component(Label("Also Important".to_string()));

        // Query nodes with labels
        let labeled = graph.query_nodes_with_component::<Label>();
        assert_eq!(labeled.len(), 2);

        // Add edges
        let e1 = graph.add_edge(n1, n2, 10).unwrap();
        let e2 = graph.add_edge(n2, n3, 20).unwrap();

        // Add label to one edge
        graph.get_edge_mut(e1).unwrap()
            .add_component(Label("Primary".to_string()));

        // Query edges with labels
        let labeled_edges = graph.query_edges_with_component::<Label>();
        assert_eq!(labeled_edges.len(), 1);
    }
}
