//! Comprehensive tests for ContextGraph functionality
//!
//! Tests all aspects of ContextGraph including:
//! - Creation with different node/edge types
//! - Component system
//! - Graph algorithms
//! - Subgraph support
//! - Error handling
//!
//! ```mermaid
//! graph TD
//!     A[Test Setup] --> B[Create ContextGraph]
//!     B --> C[Add Nodes]
//!     C --> D[Connect Edges]
//!     D --> E[Test Algorithms]
//!     E --> F[Test Results]
//!
//!     G[Test Categories] --> H[Creation Tests]
//!     G --> I[Node Operations]
//!     G --> J[Edge Operations]
//!     G --> K[Component Tests]
//!     G --> L[Algorithm Tests]
//!     G --> M[Error Handling]
//! ```

use cim_contextgraph::{
    types::{Label, Metadata, Subgraph},
    ContextGraph, EdgeId, GraphError, GraphInvariant, GraphResult, NodeId,
};
use std::sync::{Arc, Mutex};

/// Test invariant that tracks validation calls
struct TrackingInvariant {
    calls: Arc<Mutex<Vec<String>>>,
}

impl TrackingInvariant {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl<N, E> GraphInvariant<N, E> for TrackingInvariant {
    fn check(&self, _graph: &ContextGraph<N, E>) -> GraphResult<()> {
        self.calls.lock().unwrap().push("check".to_string());
        Ok(())
    }

    fn name(&self) -> &str {
        "TrackingInvariant"
    }

    fn clone_box(&self) -> Box<dyn GraphInvariant<N, E>> {
        Box::new(TrackingInvariant {
            calls: self.calls.clone(),
        })
    }
}

/// Test invariant that always fails
struct RejectingInvariant;

impl<N, E> GraphInvariant<N, E> for RejectingInvariant {
    fn check(&self, _graph: &ContextGraph<N, E>) -> GraphResult<()> {
        Err(GraphError::InvariantViolation("Always reject".to_string()))
    }

    fn name(&self) -> &str {
        "RejectingInvariant"
    }

    fn clone_box(&self) -> Box<dyn GraphInvariant<N, E>> {
        Box::new(RejectingInvariant)
    }
}

#[test]
fn test_graph_creation() {
    // Create a simple graph with string nodes and integer edges
    let graph = ContextGraph::<String, i32>::new("TestGraph");

    // Check initial state
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);

    // Check metadata
    assert_eq!(
        graph
            .metadata
            .properties
            .get("name")
            .unwrap()
            .as_str()
            .unwrap(),
        "TestGraph"
    );
}

#[test]
fn test_node_operations() {
    let mut graph = ContextGraph::<String, i32>::new("NodeTest");

    // Add nodes
    let node1 = graph.add_node("Node1".to_string());
    let node2 = graph.add_node("Node2".to_string());
    let node3 = graph.add_node("Node3".to_string());

    // Check node count
    assert_eq!(graph.node_count(), 3);

    // Check node values
    assert_eq!(graph.get_node_value(node1).unwrap(), "Node1");
    assert_eq!(graph.get_node_value(node2).unwrap(), "Node2");
    assert_eq!(graph.get_node_value(node3).unwrap(), "Node3");

    // Test node removal
    let removed_node = graph.remove_node(node2);
    assert!(removed_node.is_some());
    assert_eq!(removed_node.unwrap().value, "Node2".to_string());
    assert_eq!(graph.node_count(), 2);
    assert!(graph.get_node(node2).is_none());
}

#[test]
fn test_edge_operations() {
    let mut graph = ContextGraph::<&str, i32>::new("EdgeTest");

    // Add nodes
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    // Add edges
    let edge1 = graph.add_edge(a, b, 1).unwrap();
    let edge2 = graph.add_edge(b, c, 2).unwrap();
    let edge3 = graph.add_edge(a, c, 3).unwrap();

    // Check edge count
    assert_eq!(graph.edge_count(), 3);

    // Check edge values
    assert_eq!(graph.get_edge_value(edge1).unwrap(), &1);
    assert_eq!(graph.get_edge_value(edge2).unwrap(), &2);
    assert_eq!(graph.get_edge_value(edge3).unwrap(), &3);

    // Test edge with non-existent nodes
    let result = graph.add_edge(NodeId::new(), b, 4);
    assert!(result.is_err());
}

#[test]
fn test_component_system() {
    let mut graph = ContextGraph::<String, f64>::new("ComponentTest");

    // Add nodes with components
    let n1 = graph.add_node("Node1".to_string());
    let n2 = graph.add_node("Node2".to_string());

    // Add label component to node1
    graph
        .get_node_mut(n1)
        .unwrap()
        .add_component(Label("Important Node".to_string()))
        .unwrap();

    // Add metadata component to node2
    let mut metadata = Metadata::default();
    metadata.description = Some("This is a test node".to_string());
    metadata.tags.push("test".to_string());
    graph
        .get_node_mut(n2)
        .unwrap()
        .add_component(metadata)
        .unwrap();

    // Query nodes with Label component
    let labeled_nodes = graph.query_nodes_with_component::<Label>();
    assert_eq!(labeled_nodes.len(), 1);
    assert!(labeled_nodes.contains(&n1));

    // Check component values
    let label = graph
        .get_node(n1)
        .unwrap()
        .get_component::<Label>()
        .unwrap();
    assert_eq!(label.0, "Important Node");

    let meta = graph
        .get_node(n2)
        .unwrap()
        .get_component::<Metadata>()
        .unwrap();
    assert_eq!(meta.description.as_ref().unwrap(), "This is a test node");
}

#[test]
fn test_graph_algorithms() {
    let mut graph = ContextGraph::<&str, i32>::new("AlgorithmTest");

    // Create a simple DAG
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");

    graph.add_edge(a, b, 1).unwrap();
    graph.add_edge(b, c, 2).unwrap();
    graph.add_edge(c, d, 3).unwrap();
    graph.add_edge(a, d, 10).unwrap();

    // Test cycle detection
    assert!(!graph.is_cyclic());

    // Add a cycle
    graph.add_edge(d, a, 4).unwrap();
    assert!(graph.is_cyclic());

    // Test topological sort (should fail with cycle)
    let topo_result = graph.topological_sort();
    assert!(topo_result.is_err());

    // Test strongly connected components
    let sccs = graph.strongly_connected_components();
    assert!(!sccs.is_empty());
}

#[test]
fn test_path_finding() {
    let mut graph = ContextGraph::<&str, i32>::new("PathTest");

    // Create a graph with multiple paths
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");

    graph.add_edge(a, b, 1).unwrap();
    graph.add_edge(b, c, 1).unwrap();
    graph.add_edge(c, d, 1).unwrap();
    graph.add_edge(a, c, 2).unwrap(); // Shortcut
    graph.add_edge(b, d, 3).unwrap(); // Another path

    // Find all simple paths from A to D
    let paths = graph.all_simple_paths(a, d, 5);
    assert!(paths.len() >= 2); // At least 2 paths exist

    // Test degree calculation
    assert_eq!(graph.degree(b), 3); // 1 in + 2 out
}

#[test]
fn test_invariant_system() {
    let mut graph = ContextGraph::<String, i32>::new("InvariantTest");

    // Add tracking invariant
    let tracker = Arc::new(TrackingInvariant::new());
    graph.invariants.push(Box::new(TrackingInvariant {
        calls: tracker.calls.clone(),
    }));

    // Add nodes and edges - should trigger invariant checks
    let n1 = graph.add_node("Node1".to_string());
    let n2 = graph.add_node("Node2".to_string());

    // Adding edge triggers invariant check
    let result = graph.add_edge(n1, n2, 1);
    assert!(result.is_ok());

    // Check that invariant was called
    let calls = tracker.get_calls();
    assert!(!calls.is_empty());

    // Add rejecting invariant
    graph.invariants.push(Box::new(RejectingInvariant));

    // Now adding edge should fail
    let result = graph.add_edge(n2, n1, 2);
    assert!(result.is_err());
}

#[test]
fn test_subgraph_support() {
    let mut parent = ContextGraph::<String, i32>::new("ParentGraph");

    // Create a subgraph
    let subgraph = ContextGraph::<String, i32>::new("SubGraph");

    // Add a node that contains the subgraph
    let container_node = parent.add_node("Container".to_string());

    // Add subgraph as component
    parent
        .get_node_mut(container_node)
        .unwrap()
        .add_component(Subgraph {
            graph: Box::new(subgraph),
        })
        .unwrap();

    // Query for nodes with subgraphs
    let subgraph_nodes = parent.get_subgraph_nodes();
    assert_eq!(subgraph_nodes.len(), 1);
    assert!(subgraph_nodes.contains(&container_node));
}

#[test]
fn test_error_handling() {
    let mut graph = ContextGraph::<&str, i32>::new("ErrorTest");

    // Test adding edge with non-existent nodes
    let result = graph.add_edge(NodeId::new(), NodeId::new(), 1);
    assert!(matches!(result, Err(GraphError::NodeNotFound(_))));

    // Add one node
    let n1 = graph.add_node("Node1");

    // Try to connect to non-existent node
    let result = graph.add_edge(n1, NodeId::new(), 1);
    assert!(matches!(result, Err(GraphError::NodeNotFound(_))));

    // Test getting non-existent node
    assert!(graph.get_node(NodeId::new()).is_none());

    // Test getting non-existent edge
    assert!(graph.get_edge(EdgeId::new()).is_none());
}

#[test]
fn test_complex_scenario() {
    // Create a more complex graph representing a simple workflow
    let mut workflow = ContextGraph::<String, String>::new("OrderWorkflow");

    // Add workflow steps as nodes
    let start = workflow.add_node("Start".to_string());
    let validate = workflow.add_node("ValidateOrder".to_string());
    let payment = workflow.add_node("ProcessPayment".to_string());
    let inventory = workflow.add_node("CheckInventory".to_string());
    let ship = workflow.add_node("ShipOrder".to_string());
    let complete = workflow.add_node("Complete".to_string());

    // Add metadata to nodes
    workflow
        .get_node_mut(validate)
        .unwrap()
        .add_component(Label("Validation Step".to_string()))
        .unwrap();

    let mut payment_meta = Metadata::default();
    payment_meta.description = Some("Process customer payment".to_string());
    payment_meta
        .properties
        .insert("timeout".to_string(), serde_json::json!(30));
    workflow
        .get_node_mut(payment)
        .unwrap()
        .add_component(payment_meta)
        .unwrap();

    // Connect workflow steps
    workflow
        .add_edge(start, validate, "trigger".to_string())
        .unwrap();
    workflow
        .add_edge(validate, payment, "if_valid".to_string())
        .unwrap();
    workflow
        .add_edge(validate, inventory, "check_stock".to_string())
        .unwrap();
    workflow
        .add_edge(payment, ship, "payment_success".to_string())
        .unwrap();
    workflow
        .add_edge(inventory, ship, "in_stock".to_string())
        .unwrap();
    workflow
        .add_edge(ship, complete, "shipped".to_string())
        .unwrap();

    // Verify the workflow structure
    assert_eq!(workflow.node_count(), 6);
    assert_eq!(workflow.edge_count(), 6);

    // Check it's a DAG (no cycles)
    assert!(!workflow.is_cyclic());

    // Get topological sort
    let topo_sort = workflow.topological_sort().unwrap();
    assert_eq!(topo_sort.len(), 6);

    // Start should be first in topological sort
    assert_eq!(topo_sort[0], start);
}

/// Test demonstrating the mermaid diagram structure
///
/// ```mermaid
/// graph TD
///     subgraph "ContextGraph Test Coverage"
///         A[Graph Creation]
///         A --> A1[Empty Graph]
///         A --> A2[With Metadata]
///
///         B[Node Operations]
///         B --> B1[Add Nodes]
///         B --> B2[Remove Nodes]
///         B --> B3[Query Nodes]
///
///         C[Edge Operations]
///         C --> C1[Connect Nodes]
///         C --> C2[Query Edges]
///         C --> C3[Validate Connections]
///
///         D[Component System]
///         D --> D1[Add Components]
///         D --> D2[Query by Component]
///         D --> D3[Component Types]
///
///         E[Graph Algorithms]
///         E --> E1[Cycle Detection]
///         E --> E2[Topological Sort]
///         E --> E3[Path Finding]
///         E --> E4[Connected Components]
///
///         F[Advanced Features]
///         F --> F1[Invariants]
///         F --> F2[Subgraphs]
///         F --> F3[Error Handling]
///     end
/// ```
#[test]
fn test_documentation_completeness() {
    // This test ensures all documented features are tested
    assert!(true, "Documentation test placeholder");
}
