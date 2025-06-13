//! CID-based DAG for Event Store and Object Store
//!
//! This provides a specialized graph type for content-addressed storage where:
//! - Nodes are identified by CIDs (Content Identifiers)
//! - Edges represent causal relationships (previous CID -> next CID)
//! - The graph is acyclic (enforced by content addressing)
//! - Used for both Event Store chains and Object Store references

use crate::types::*;
use daggy::{Dag, NodeIndex, EdgeIndex};
use cid::Cid;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

/// A CID-indexed DAG for Event Store and Object Store
#[derive(Debug, Clone)]
pub struct CidDag<T> {
    /// The underlying daggy DAG
    dag: Dag<CidNode<T>, CidEdge>,

    /// CID to NodeIndex mapping for fast lookups
    cid_index: HashMap<Cid, NodeIndex>,

    /// Reverse mapping
    node_to_cid: HashMap<NodeIndex, Cid>,

    /// Root CIDs (nodes with no parents)
    roots: Vec<Cid>,

    /// Leaf CIDs (nodes with no children)
    leaves: Vec<Cid>,
}

/// Node in a CID DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CidNode<T> {
    pub cid: Cid,
    pub content: T,
    pub timestamp: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge in a CID DAG representing causal relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CidEdge {
    pub edge_type: CidEdgeType,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CidEdgeType {
    /// Direct causal relationship (previous -> next)
    Causal,
    /// Reference to content (e.g., Event references Object)
    Reference,
    /// Merge of multiple chains
    Merge,
    /// Fork point
    Fork,
}

/// Specialized node types for Event Store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNode {
    pub event_id: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub sequence: u64,
    pub payload_cid: Option<Cid>, // Reference to Object Store
}

/// Specialized node types for Object Store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectNode {
    pub object_type: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub chunks: Vec<Cid>, // For large objects split into chunks
}

impl<T> CidDag<T>
where
    T: Clone + Debug + Serialize,
{
    /// Create a new empty CID DAG
    pub fn new() -> Self {
        Self {
            dag: Dag::new(),
            cid_index: HashMap::new(),
            node_to_cid: HashMap::new(),
            roots: Vec::new(),
            leaves: Vec::new(),
        }
    }

    /// Add a new node with its CID
    pub fn add_node(&mut self, cid: Cid, content: T, timestamp: u64) -> Result<(), GraphError> {
        // Ensure CID is unique
        if self.cid_index.contains_key(&cid) {
            return Err(GraphError::DuplicateCid(cid));
        }

        let node = CidNode {
            cid,
            content,
            timestamp,
            metadata: HashMap::new(),
        };

        let node_idx = self.dag.add_node(node);
        self.cid_index.insert(cid, node_idx);
        self.node_to_cid.insert(node_idx, cid);

        // New nodes start as both roots and leaves
        self.roots.push(cid);
        self.leaves.push(cid);

        Ok(())
    }

    /// Add a causal edge (previous -> next)
    pub fn add_causal_edge(&mut self, previous: Cid, next: Cid) -> Result<(), GraphError> {
        let prev_idx = self.cid_index.get(&previous)
            .ok_or(GraphError::CidNotFound(previous))?;
        let next_idx = self.cid_index.get(&next)
            .ok_or(GraphError::CidNotFound(next))?;

        let edge = CidEdge {
            edge_type: CidEdgeType::Causal,
            metadata: HashMap::new(),
        };

        // This will fail if it creates a cycle (DAG property)
        self.dag.add_edge(*prev_idx, *next_idx, edge)
            .map_err(|_| GraphError::CycleDetected)?;

        // Update roots and leaves
        self.roots.retain(|&cid| cid != next); // next is no longer a root
        self.leaves.retain(|&cid| cid != previous); // previous is no longer a leaf

        Ok(())
    }

    /// Add a reference edge (e.g., Event -> Object)
    pub fn add_reference(&mut self, from: Cid, to: Cid) -> Result<(), GraphError> {
        let from_idx = self.cid_index.get(&from)
            .ok_or(GraphError::CidNotFound(from))?;
        let to_idx = self.cid_index.get(&to)
            .ok_or(GraphError::CidNotFound(to))?;

        let edge = CidEdge {
            edge_type: CidEdgeType::Reference,
            metadata: HashMap::new(),
        };

        self.dag.add_edge(*from_idx, *to_idx, edge)
            .map_err(|_| GraphError::CycleDetected)?;

        Ok(())
    }

    /// Get a node by CID
    pub fn get_node(&self, cid: &Cid) -> Option<&CidNode<T>> {
        self.cid_index.get(cid)
            .and_then(|idx| self.dag.node_weight(*idx))
    }

    /// Get all ancestors of a CID (previous events in chain)
    pub fn ancestors(&self, cid: &Cid) -> Vec<Cid> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        if let Some(&node_idx) = self.cid_index.get(cid) {
            // Manually walk parents
            let mut to_visit = vec![node_idx];

            while let Some(current) = to_visit.pop() {
                if visited.insert(current) {
                    // Get parents of current node
                    for (_, parent_idx) in self.dag.parents(current).iter(&self.dag) {
                        to_visit.push(parent_idx);
                        if let Some(&parent_cid) = self.node_to_cid.get(&parent_idx) {
                            result.push(parent_cid);
                        }
                    }
                }
            }
        }

        result
    }

    /// Get all descendants of a CID
    pub fn descendants(&self, cid: &Cid) -> Vec<Cid> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        if let Some(&node_idx) = self.cid_index.get(cid) {
            // Manually walk children
            let mut to_visit = vec![node_idx];

            while let Some(current) = to_visit.pop() {
                if visited.insert(current) {
                    // Get children of current node
                    for (_, child_idx) in self.dag.children(current).iter(&self.dag) {
                        to_visit.push(child_idx);
                        if let Some(&child_cid) = self.node_to_cid.get(&child_idx) {
                            result.push(child_cid);
                        }
                    }
                }
            }
        }

        result
    }

    /// Verify the chain from a CID back to a root
    pub fn verify_chain(&self, from: &Cid, to_root: Option<&Cid>) -> Result<Vec<Cid>, GraphError> {
        let mut chain = vec![*from];
        let mut current = *from;

        while let Some(node_idx) = self.cid_index.get(&current) {
            let parents: Vec<_> = self.dag.parents(*node_idx)
                .iter(&self.dag)
                .filter_map(|(edge_idx, parent_idx)| {
                    // Only follow causal edges
                    let edge = self.dag.edge_weight(edge_idx)?;
                    match edge.edge_type {
                        CidEdgeType::Causal => self.node_to_cid.get(&parent_idx).copied(),
                        _ => None,
                    }
                })
                .collect();

            if parents.is_empty() {
                // Reached a root
                if let Some(expected_root) = to_root {
                    if current != *expected_root {
                        return Err(GraphError::ChainVerificationFailed);
                    }
                }
                break;
            }

            if parents.len() > 1 {
                // Multiple parents - this is a merge point
                // For now, just follow the first parent
                // Could implement more sophisticated merge handling
            }

            current = parents[0];
            chain.push(current);
        }

        chain.reverse(); // Return root-to-leaf order
        Ok(chain)
    }

    /// Get the latest CIDs (leaves)
    pub fn latest_cids(&self) -> &[Cid] {
        &self.leaves
    }

    /// Get root CIDs
    pub fn root_cids(&self) -> &[Cid] {
        &self.roots
    }

    /// Find common ancestor of two CIDs
    pub fn common_ancestor(&self, cid1: &Cid, cid2: &Cid) -> Option<Cid> {
        let ancestors1: HashSet<_> = self.ancestors(cid1).into_iter().collect();
        let ancestors2 = self.ancestors(cid2);

        // Find first common ancestor
        ancestors2.into_iter()
            .find(|cid| ancestors1.contains(cid))
    }

    /// Convert to a ContextGraph representation
    pub fn to_context_graph(&self) -> crate::context_graph::ContextGraph<CidNode<T>, CidEdge> {
        let mut graph = crate::context_graph::ContextGraph::new("CID DAG");

        // Map CIDs to NodeIds
        let mut cid_to_node_id = HashMap::new();

        // Add all nodes
        for (cid, node) in &self.cid_index {
            if let Some(cid_node) = self.dag.node_weight(*node) {
                let node_id = graph.add_node(cid_node.clone());
                cid_to_node_id.insert(cid, node_id);

                // Add CID as a component for easy lookup
                let _ = graph.get_node_mut(node_id).unwrap()
                    .add_component(CidReference(*cid));
            }
        }

        // Add all edges
        for edge_idx in self.dag.raw_edges().iter().enumerate().map(|(i, _)| EdgeIndex::new(i)) {
            if let Some((src_idx, dst_idx)) = self.dag.edge_endpoints(edge_idx) {
                if let (Some(&src_cid), Some(&dst_cid)) =
                    (self.node_to_cid.get(&src_idx), self.node_to_cid.get(&dst_idx)) {

                    if let (Some(&src_id), Some(&dst_id)) =
                        (cid_to_node_id.get(&src_cid), cid_to_node_id.get(&dst_cid)) {

                        if let Some(edge) = self.dag.edge_weight(edge_idx) {
                            graph.add_edge(src_id, dst_id, edge.clone()).ok();
                        }
                    }
                }
            }
        }

        graph
    }
}

/// Component to mark nodes with their CID
#[derive(Debug, Clone)]
pub struct CidReference(pub Cid);

impl Component for CidReference {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "CidReference"
    }
}

/// Specialized CID DAG for Event Store
pub type EventDag = CidDag<EventNode>;

impl EventDag {
    /// Add an event to the chain
    pub fn add_event(
        &mut self,
        event_cid: Cid,
        previous_cid: Option<Cid>,
        event: EventNode,
        timestamp: u64,
    ) -> Result<(), GraphError> {
        // Store payload_cid before moving event
        let payload_cid = event.payload_cid;

        // Add the event node
        self.add_node(event_cid, event, timestamp)?;

        // Link to previous event if provided
        if let Some(prev) = previous_cid {
            self.add_causal_edge(prev, event_cid)?;
        }

        // If event references an object, add reference edge
        if let Some(payload_cid) = payload_cid {
            // This assumes the object is already in the DAG
            // In practice, might need to handle missing objects
            self.add_reference(event_cid, payload_cid).ok();
        }

        Ok(())
    }

    /// Get event chain for an aggregate
    pub fn get_aggregate_events(&self, aggregate_id: &str) -> Vec<(Cid, &EventNode)> {
        let mut events = Vec::new();

        for (cid, node_idx) in &self.cid_index {
            if let Some(node) = self.dag.node_weight(*node_idx) {
                if node.content.aggregate_id == aggregate_id {
                    events.push((*cid, &node.content));
                }
            }
        }

        // Sort by sequence number
        events.sort_by_key(|(_, event)| event.sequence);
        events
    }
}

/// Specialized CID DAG for Object Store
pub type ObjectDag = CidDag<ObjectNode>;

impl ObjectDag {
    /// Add an object with optional chunking
    pub fn add_object(
        &mut self,
        object_cid: Cid,
        object: ObjectNode,
        timestamp: u64,
    ) -> Result<(), GraphError> {
        // Add the main object node
        self.add_node(object_cid, object.clone(), timestamp)?;

        // Link to chunk CIDs if present
        for chunk_cid in &object.chunks {
            // Chunks should already be in the DAG
            self.add_reference(object_cid, *chunk_cid).ok();
        }

        Ok(())
    }

    /// Get all chunks for an object
    pub fn get_object_chunks(&self, object_cid: &Cid) -> Vec<Cid> {
        if let Some(node) = self.get_node(object_cid) {
            node.content.chunks.clone()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_chain() {
        let mut event_dag = EventDag::new();

        // Create some test CIDs
        let cid1 = Cid::default(); // Would be real CID in practice
        let cid2 = Cid::default(); // Different CID

        let event1 = EventNode {
            event_id: "evt-1".to_string(),
            aggregate_id: "agg-1".to_string(),
            event_type: "Created".to_string(),
            sequence: 1,
            payload_cid: None,
        };

        let event2 = EventNode {
            event_id: "evt-2".to_string(),
            aggregate_id: "agg-1".to_string(),
            event_type: "Updated".to_string(),
            sequence: 2,
            payload_cid: None,
        };

        // Add events
        event_dag.add_event(cid1, None, event1, 1000).unwrap();
        event_dag.add_event(cid2, Some(cid1), event2, 2000).unwrap();

        // Verify chain
        let chain = event_dag.verify_chain(&cid2, Some(&cid1)).unwrap();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], cid1);
        assert_eq!(chain[1], cid2);

        // Check ancestors
        let ancestors = event_dag.ancestors(&cid2);
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0], cid1);
    }
}
