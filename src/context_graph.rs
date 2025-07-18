//! ContextGraph v2 - Wrapping PetGraph for best of both worlds
//!
//! This approach gives us:
//! - All PetGraph algorithms and optimizations
//! - Component system for extensibility
//! - Domain-specific features
//! - Recursive graph support

use crate::types::*;
use petgraph::graph::{EdgeIndex, Graph, NodeIndex};
use std::collections::HashMap;
use std::fmt::Debug;

/// Trait for graph invariants that must be maintained
pub trait GraphInvariant<N, E>: Send + Sync {
    fn check(&self, graph: &ContextGraph<N, E>) -> GraphResult<()>;
    fn name(&self) -> &str;
    fn clone_box(&self) -> Box<dyn GraphInvariant<N, E>>;
}

/// ContextGraph wraps PetGraph with our component system
pub struct ContextGraph<N, E> {
    pub id: ContextGraphId,

    // The actual PetGraph - we get all its algorithms!
    pub graph: Graph<NodeEntry<N>, EdgeEntry<E>>,

    // Additional mappings for our ID system
    node_id_map: HashMap<NodeId, NodeIndex>,
    edge_id_map: HashMap<EdgeId, EdgeIndex>,

    // Reverse mappings
    node_index_map: HashMap<NodeIndex, NodeId>,
    edge_index_map: HashMap<EdgeIndex, EdgeId>,

    pub metadata: Metadata,

    pub invariants: Vec<Box<dyn GraphInvariant<N, E>>>,
}

impl<N, E> Debug for ContextGraph<N, E>
where
    N: Debug,
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextGraph")
            .field("id", &self.id)
            .field("graph", &self.graph)
            .field("metadata", &self.metadata)
            .field("invariants_count", &self.invariants.len())
            .finish()
    }
}

impl<N, E> Clone for ContextGraph<N, E>
where
    N: Clone,
    E: Clone,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            graph: self.graph.clone(),
            node_id_map: self.node_id_map.clone(),
            edge_id_map: self.edge_id_map.clone(),
            node_index_map: self.node_index_map.clone(),
            edge_index_map: self.edge_index_map.clone(),
            metadata: self.metadata.clone(),
            invariants: self.invariants.iter().map(|inv| inv.clone_box()).collect(),
        }
    }
}

impl<N, E> ContextGraph<N, E>
where
    N: Clone + Debug,
    E: Clone + Debug,
{
    pub fn new(name: impl Into<String>) -> Self {
        let mut metadata = Metadata::default();
        metadata
            .properties
            .insert("name".to_string(), serde_json::json!(name.into()));

        Self {
            id: ContextGraphId::new(),
            graph: Graph::new(),
            node_id_map: HashMap::new(),
            edge_id_map: HashMap::new(),
            node_index_map: HashMap::new(),
            edge_index_map: HashMap::new(),
            metadata,
            invariants: Vec::new(),
        }
    }

    /// Add a node - wraps PetGraph's add_node
    pub fn add_node(&mut self, value: N) -> NodeId {
        let node_entry = NodeEntry::new(value);
        let node_id = node_entry.id;

        // Add to PetGraph
        let node_index = self.graph.add_node(node_entry);

        // Maintain our mappings
        self.node_id_map.insert(node_id, node_index);
        self.node_index_map.insert(node_index, node_id);

        node_id
    }

    /// Add an edge - wraps PetGraph's add_edge
    pub fn add_edge(&mut self, source: NodeId, target: NodeId, value: E) -> GraphResult<EdgeId> {
        // Get PetGraph indices
        let source_idx = self
            .node_id_map
            .get(&source)
            .ok_or(GraphError::NodeNotFound(source))?;
        let target_idx = self
            .node_id_map
            .get(&target)
            .ok_or(GraphError::NodeNotFound(target))?;

        let edge_entry = EdgeEntry::new(source, target, value);
        let edge_id = edge_entry.id;

        // Add to PetGraph
        let edge_index = self.graph.add_edge(*source_idx, *target_idx, edge_entry);

        // Maintain mappings
        self.edge_id_map.insert(edge_id, edge_index);
        self.edge_index_map.insert(edge_index, edge_id);

        // Check invariants
        self.check_invariants()?;

        Ok(edge_id)
    }

    /// Get node by our ID
    pub fn get_node(&self, id: NodeId) -> Option<&NodeEntry<N>> {
        self.node_id_map
            .get(&id)
            .and_then(|idx| self.graph.node_weight(*idx))
    }

    /// Get mutable node by our ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut NodeEntry<N>> {
        self.node_id_map
            .get(&id)
            .and_then(|idx| self.graph.node_weight_mut(*idx))
    }

    // Now we can expose PetGraph algorithms directly!

    /// Find shortest path using PetGraph's dijkstra
    pub fn shortest_path(&self, start: NodeId, end: NodeId) -> Option<Vec<NodeId>> {
        use petgraph::algo::dijkstra;

        let start_idx = self.node_id_map.get(&start)?;
        let end_idx = self.node_id_map.get(&end)?;

        let node_map = dijkstra(&self.graph, *start_idx, Some(*end_idx), |_| 1);

        // Convert back to our IDs
        if node_map.contains_key(end_idx) {
            // Reconstruct path...
            Some(vec![]) // Simplified
        } else {
            None
        }
    }

    /// Check if graph is cyclic using PetGraph
    pub fn is_cyclic(&self) -> bool {
        petgraph::algo::is_cyclic_directed(&self.graph)
    }

    /// Get strongly connected components
    pub fn strongly_connected_components(&self) -> Vec<Vec<NodeId>> {
        use petgraph::algo::kosaraju_scc;

        let sccs = kosaraju_scc(&self.graph);

        // Convert to our IDs
        sccs.into_iter()
            .map(|component| {
                component
                    .into_iter()
                    .filter_map(|idx| self.node_index_map.get(&idx).copied())
                    .collect()
            })
            .collect()
    }

    /// Topological sort
    pub fn topological_sort(&self) -> Result<Vec<NodeId>, GraphError> {
        use petgraph::algo::toposort;

        match toposort(&self.graph, None) {
            Ok(sorted) => Ok(sorted
                .into_iter()
                .filter_map(|idx| self.node_index_map.get(&idx).copied())
                .collect()),
            Err(_) => Err(GraphError::CycleDetected),
        }
    }

    // Component-based queries (our added value)

    /// Query nodes by component type
    pub fn query_nodes_with_component<T: Component + 'static>(&self) -> Vec<NodeId> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let node = &self.graph[idx];
                if node.components.has::<T>() {
                    self.node_index_map.get(&idx).copied()
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all subgraph nodes (for recursion)
    pub fn get_subgraph_nodes(&self) -> Vec<NodeId>
    where
        N: 'static + Send + Sync,
        E: 'static + Send + Sync,
    {
        self.query_nodes_with_component::<Subgraph<N, E>>()
    }

    /// Recursive visitor using PetGraph's DFS
    pub fn visit_recursive<F>(&self, mut visitor: F)
    where
        F: FnMut(&ContextGraph<N, E>, usize),
        N: 'static + Send + Sync,
        E: 'static + Send + Sync,
    {
        use petgraph::visit::Dfs;

        // Visit this graph
        visitor(self, 0);

        // Use DFS to find all nodes with subgraphs
        let mut dfs = Dfs::new(
            &self.graph,
            self.graph
                .node_indices()
                .next()
                .unwrap_or(NodeIndex::new(0)),
        );

        while let Some(nx) = dfs.next(&self.graph) {
            if let Some(node) = self.graph.node_weight(nx) {
                if let Some(subgraph) = node.get_component::<Subgraph<N, E>>() {
                    subgraph.graph.visit_recursive_impl(&mut visitor, 1);
                }
            }
        }
    }

    fn visit_recursive_impl<F>(&self, visitor: &mut F, depth: usize)
    where
        F: FnMut(&ContextGraph<N, E>, usize),
        N: 'static + Send + Sync,
        E: 'static + Send + Sync,
    {
        visitor(self, depth);

        for node_id in self.get_subgraph_nodes() {
            if let Some(node) = self.get_node(node_id) {
                if let Some(subgraph) = node.get_component::<Subgraph<N, E>>() {
                    subgraph.graph.visit_recursive_impl(visitor, depth + 1);
                }
            }
        }
    }

    // Invariant checking
    pub fn check_invariants(&self) -> GraphResult<()> {
        for invariant in &self.invariants {
            invariant.check(self)?;
        }
        Ok(())
    }

    /// Remove a node and clean up mappings (for testing)
    #[doc(hidden)]
    pub fn remove_node(&mut self, node_id: NodeId) -> Option<NodeEntry<N>> {
        if let Some(node_idx) = self.node_id_map.remove(&node_id) {
            // First, get the node data before removal
            let node_data = self.graph.node_weight(node_idx).cloned();

            // Remove the node from PetGraph
            self.graph.remove_node(node_idx);

            // Remove from reverse mapping
            self.node_index_map.remove(&node_idx);

            // PetGraph shifts indices when removing nodes, so we need to rebuild our mappings
            // Clear and rebuild all mappings
            self.node_id_map.clear();
            self.node_index_map.clear();
            self.edge_id_map.clear();
            self.edge_index_map.clear();

            // Rebuild node mappings
            for node_idx in self.graph.node_indices() {
                if let Some(node) = self.graph.node_weight(node_idx) {
                    self.node_id_map.insert(node.id, node_idx);
                    self.node_index_map.insert(node_idx, node.id);
                }
            }

            // Rebuild edge mappings
            for edge_idx in self.graph.edge_indices() {
                if let Some(edge) = self.graph.edge_weight(edge_idx) {
                    self.edge_id_map.insert(edge.id, edge_idx);
                    self.edge_index_map.insert(edge_idx, edge.id);
                }
            }

            node_data
        } else {
            None
        }
    }

    // Convenience methods for common operations

    /// Get the number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Get the value of a node
    pub fn get_node_value(&self, node_id: NodeId) -> Option<&N> {
        self.get_node(node_id).map(|entry| &entry.value)
    }

    /// Get the value of an edge
    pub fn get_edge_value(&self, edge_id: EdgeId) -> Option<&E> {
        self.edge_id_map
            .get(&edge_id)
            .and_then(|idx| self.graph.edge_weight(*idx))
            .map(|entry| &entry.value)
    }

    /// Get an edge by ID
    pub fn get_edge(&self, edge_id: EdgeId) -> Option<&EdgeEntry<E>> {
        self.edge_id_map
            .get(&edge_id)
            .and_then(|idx| self.graph.edge_weight(*idx))
    }

    /// Get a mutable edge by ID
    pub fn get_edge_mut(&mut self, edge_id: EdgeId) -> Option<&mut EdgeEntry<E>> {
        self.edge_id_map
            .get(&edge_id)
            .and_then(|idx| self.graph.edge_weight_mut(*idx))
    }

    /// Get the degree of a node (in + out edges)
    pub fn degree(&self, node_id: NodeId) -> usize {
        use petgraph::Direction;

        if let Some(node_idx) = self.node_id_map.get(&node_id) {
            self.graph
                .edges_directed(*node_idx, Direction::Incoming)
                .count()
                + self
                    .graph
                    .edges_directed(*node_idx, Direction::Outgoing)
                    .count()
        } else {
            0
        }
    }

    /// Get all nodes as an iterator
    pub fn get_all_nodes(&self) -> impl Iterator<Item = (NodeId, &NodeEntry<N>)> {
        self.graph.node_indices().filter_map(move |idx| {
            self.node_index_map
                .get(&idx)
                .and_then(|id| self.graph.node_weight(idx).map(|node| (*id, node)))
        })
    }

    /// Get all edges as an iterator
    pub fn get_all_edges(&self) -> impl Iterator<Item = (EdgeId, &EdgeEntry<E>)> {
        self.graph.edge_indices().filter_map(move |idx| {
            self.edge_index_map
                .get(&idx)
                .and_then(|id| self.graph.edge_weight(idx).map(|edge| (*id, edge)))
        })
    }

    /// Find all paths between two nodes (simple wrapper for find_paths)
    pub fn find_paths(&self, start: NodeId, end: NodeId) -> Vec<Vec<NodeId>> {
        self.all_simple_paths(start, end, 10) // Default max length of 10
    }

    /// Count total nodes including subgraphs
    pub fn total_node_count(&self) -> usize
    where
        N: 'static + Send + Sync,
        E: 'static + Send + Sync,
    {
        let mut count = self.node_count();

        for node_id in self.get_subgraph_nodes() {
            if let Some(node) = self.get_node(node_id) {
                if let Some(subgraph) = node.get_component::<Subgraph<N, E>>() {
                    count += subgraph.graph.total_node_count();
                }
            }
        }

        count
    }
}

// Now we can implement more complex algorithms easily!

impl<N, E> ContextGraph<N, E>
where
    N: Clone + Debug,
    E: Clone + Debug,
{
    /// Find all simple paths between two nodes
    pub fn all_simple_paths(
        &self,
        start: NodeId,
        end: NodeId,
        max_length: usize,
    ) -> Vec<Vec<NodeId>> {
        use petgraph::algo::all_simple_paths;

        let start_idx = match self.node_id_map.get(&start) {
            Some(idx) => *idx,
            None => return vec![],
        };

        let end_idx = match self.node_id_map.get(&end) {
            Some(idx) => *idx,
            None => return vec![],
        };

        let paths: Vec<Vec<NodeIndex>> =
            all_simple_paths(&self.graph, start_idx, end_idx, 0, Some(max_length)).collect();

        // Convert to our IDs
        paths
            .into_iter()
            .map(|path| {
                path.into_iter()
                    .filter_map(|idx| self.node_index_map.get(&idx).copied())
                    .collect()
            })
            .collect()
    }

    /// Minimum spanning tree
    pub fn minimum_spanning_tree(&self) -> ContextGraph<N, E> {
        // Note: This would require EdgeEntry<E> to implement PartialOrd
        // For now, returning a new empty graph
        // TODO: Implement proper MST when edge weights can be compared

        // This would need proper implementation...
        Self::new("MST")
    }

    /// Page rank algorithm
    pub fn page_rank(&self, _damping_factor: f64, _max_iterations: usize) -> HashMap<NodeId, f64> {
        // We can implement PageRank using PetGraph's structure
        let mut ranks = HashMap::new();

        // Initialize ranks
        let n = self.graph.node_count() as f64;
        for node_id in self.node_index_map.values() {
            ranks.insert(*node_id, 1.0 / n);
        }

        // Iterate...
        // (simplified implementation)

        ranks
    }

    /// Get the PetGraph node index for a NodeId (for testing)
    #[doc(hidden)]
    pub fn get_node_index(&self, node_id: NodeId) -> Option<NodeIndex> {
        self.node_id_map.get(&node_id).copied()
    }

    /// Get the PetGraph edge index for an EdgeId (for testing)
    #[doc(hidden)]
    pub fn get_edge_index(&self, edge_id: EdgeId) -> Option<EdgeIndex> {
        self.edge_id_map.get(&edge_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_petgraph_algorithms() {
        let mut graph = ContextGraph::<&str, i32>::new("TestGraph");

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");

        graph.add_edge(a, b, 1).unwrap();
        graph.add_edge(b, c, 2).unwrap();
        graph.add_edge(c, d, 3).unwrap();
        graph.add_edge(a, d, 10).unwrap();

        // Test shortest path
        let path = graph.shortest_path(a, d);
        assert!(path.is_some());

        // Test cycle detection
        assert!(!graph.is_cyclic());

        // Add cycle
        graph.add_edge(d, a, 4).unwrap();
        assert!(graph.is_cyclic());

        // Test topological sort (should fail with cycle)
        assert!(graph.topological_sort().is_err());
    }

    #[test]
    fn test_component_queries_with_petgraph() {
        let mut graph = ContextGraph::<String, f64>::new("ComponentTest");

        let n1 = graph.add_node("Node1".to_string());
        let n2 = graph.add_node("Node2".to_string());
        let n3 = graph.add_node("Node3".to_string());

        // Add labels to some nodes
        graph
            .get_node_mut(n1)
            .unwrap()
            .add_component(Label("Important".to_string()));
        graph
            .get_node_mut(n3)
            .unwrap()
            .add_component(Label("Also Important".to_string()));

        // Query works with PetGraph backend
        let labeled = graph.query_nodes_with_component::<Label>();
        assert_eq!(labeled.len(), 2);
        assert!(labeled.contains(&n1));
        assert!(labeled.contains(&n3));
    }
}
