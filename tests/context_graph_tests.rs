//! Tests for ContextGraph - The Universal Base Graph Abstraction
//!
//! These tests demonstrate how to use ContextGraph for various scenarios,
//! from simple primitive graphs to complex domain models.

use cim_contextgraph::{ContextGraph, NodeId, EdgeId, Label, Metadata, Subgraph, Component};
use serde::{Serialize, Deserialize};
use std::any::Any;

/// Test basic graph creation with primitive types
#[test]
fn test_primitive_graph_creation() {
    // Create a graph with string nodes and float edge weights
    let mut graph = ContextGraph::<String, f64>::new("Social Network");

    // Add nodes - can be any type including primitives
    let alice = graph.add_node("Alice".to_string());
    let bob = graph.add_node("Bob".to_string());
    let charlie = graph.add_node("Charlie".to_string());

    // Add edges with weights representing relationship strength
    let edge1 = graph.add_edge(alice, bob, 0.8).unwrap();
    let edge2 = graph.add_edge(bob, charlie, 0.6).unwrap();
    let edge3 = graph.add_edge(alice, charlie, 0.3).unwrap();

    // Verify structure
    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 3);

    // Access node values
    assert_eq!(graph.get_node_value(alice).unwrap(), "Alice");
    assert_eq!(graph.get_edge_value(edge1).unwrap(), &0.8);
}

/// Test component system - adding metadata without changing core types
#[test]
fn test_component_system() {
    // Graph with integer nodes and edges
    let mut graph = ContextGraph::<i32, i32>::new("Number Graph");

    let n1 = graph.add_node(100);
    let n2 = graph.add_node(200);
    let edge = graph.add_edge(n1, n2, 50).unwrap();

    // Add components to nodes
    graph.get_node_mut(n1).unwrap()
        .add_component(Label("Start Node".to_string()))
        .unwrap();

    graph.get_node_mut(n2).unwrap()
        .add_component(Label("End Node".to_string()))
        .unwrap();

    // Add metadata component
    let mut metadata = Metadata::default();
    metadata.description = Some("Connection between start and end".to_string());
    metadata.tags = vec!["important".to_string(), "primary".to_string()];

    graph.get_edge_mut(edge).unwrap()
        .add_component(metadata)
        .unwrap();

    // Query components
    let label = graph.get_node(n1).unwrap()
        .get_component::<Label>()
        .unwrap();
    assert_eq!(label.0, "Start Node");

    // Query nodes by component type
    let labeled_nodes = graph.query_nodes_with_component::<Label>();
    assert_eq!(labeled_nodes.len(), 2);
}

/// Custom domain components for business logic
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CustomerInfo {
    customer_id: String,
    tier: CustomerTier,
    lifetime_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CustomerTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl Component for CustomerInfo {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "CustomerInfo" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PurchaseInfo {
    order_id: String,
    amount: f64,
    date: String,
}

impl Component for PurchaseInfo {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "PurchaseInfo" }
}

/// Test domain-specific graph with custom components
#[test]
fn test_domain_graph_with_components() {
    // E-commerce customer graph
    let mut graph = ContextGraph::<String, String>::new("Customer Purchase Graph");

    // Add customer nodes
    let customer1 = graph.add_node("CUST-001".to_string());
    let customer2 = graph.add_node("CUST-002".to_string());

    // Add product nodes
    let product1 = graph.add_node("PROD-A".to_string());
    let product2 = graph.add_node("PROD-B".to_string());

    // Add customer info components
    graph.get_node_mut(customer1).unwrap()
        .add_component(CustomerInfo {
            customer_id: "CUST-001".to_string(),
            tier: CustomerTier::Gold,
            lifetime_value: 5000.0,
        })
        .unwrap();

    graph.get_node_mut(customer2).unwrap()
        .add_component(CustomerInfo {
            customer_id: "CUST-002".to_string(),
            tier: CustomerTier::Silver,
            lifetime_value: 2000.0,
        })
        .unwrap();

    // Add purchase edges
    let purchase1 = graph.add_edge(customer1, product1, "purchased".to_string()).unwrap();
    graph.get_edge_mut(purchase1).unwrap()
        .add_component(PurchaseInfo {
            order_id: "ORD-123".to_string(),
            amount: 299.99,
            date: "2024-01-15".to_string(),
        })
        .unwrap();

    // Query high-value customers
    let gold_customers: Vec<_> = graph.get_all_nodes()
        .filter_map(|(id, node)| {
            node.get_component::<CustomerInfo>()
                .filter(|info| matches!(info.tier, CustomerTier::Gold))
                .map(|_| id)
        })
        .collect();

    assert_eq!(gold_customers.len(), 1);
}

/// Test recursive graphs - graphs containing graphs
#[test]
fn test_recursive_graph_structure() {
    // Create a company org chart where departments contain teams
    let mut company = ContextGraph::<String, String>::new("TechCorp");

    // Add department nodes
    let engineering = company.add_node("Engineering".to_string());
    let sales = company.add_node("Sales".to_string());

    // Create engineering sub-graph
    let mut eng_teams = ContextGraph::<String, String>::new("Engineering Teams");
    let backend = eng_teams.add_node("Backend Team".to_string());
    let frontend = eng_teams.add_node("Frontend Team".to_string());
    let devops = eng_teams.add_node("DevOps Team".to_string());

    eng_teams.add_edge(backend, devops, "deploys to".to_string()).unwrap();
    eng_teams.add_edge(frontend, devops, "deploys to".to_string()).unwrap();

    // Attach sub-graph to engineering department
    company.get_node_mut(engineering).unwrap()
        .add_component(Subgraph {
            graph: Box::new(eng_teams)
        })
        .unwrap();

    // Create sales sub-graph
    let mut sales_teams = ContextGraph::<String, String>::new("Sales Teams");
    let enterprise = sales_teams.add_node("Enterprise Sales".to_string());
    let smb = sales_teams.add_node("SMB Sales".to_string());

    sales_teams.add_edge(enterprise, smb, "mentors".to_string()).unwrap();

    // Attach sub-graph to sales department
    company.get_node_mut(sales).unwrap()
        .add_component(Subgraph {
            graph: Box::new(sales_teams)
        })
        .unwrap();

    // Connect departments
    company.add_edge(engineering, sales, "supports".to_string()).unwrap();

    // Count total nodes including subgraphs
    let total_nodes = company.total_node_count();
    assert_eq!(total_nodes, 7); // 2 departments + 3 eng teams + 2 sales teams

    // Find all subgraph nodes
    let subgraph_nodes = company.get_subgraph_nodes();
    assert_eq!(subgraph_nodes.len(), 2); // Engineering and Sales have subgraphs
}

/// Test graph algorithms and traversal
#[test]
fn test_graph_algorithms() {
    // Create a workflow graph
    let mut workflow = ContextGraph::<&'static str, &'static str>::new("Order Processing");

    let start = workflow.add_node("Order Received");
    let validate = workflow.add_node("Validate Payment");
    let inventory = workflow.add_node("Check Inventory");
    let ship = workflow.add_node("Ship Order");
    let notify = workflow.add_node("Notify Customer");
    let complete = workflow.add_node("Order Complete");

    // Build workflow edges
    workflow.add_edge(start, validate, "next").unwrap();
    workflow.add_edge(validate, inventory, "if valid").unwrap();
    workflow.add_edge(inventory, ship, "if available").unwrap();
    workflow.add_edge(ship, notify, "after shipping").unwrap();
    workflow.add_edge(notify, complete, "finish").unwrap();

    // Alternative path
    workflow.add_edge(validate, notify, "if invalid").unwrap();
    workflow.add_edge(inventory, notify, "if unavailable").unwrap();

    // Find all paths from start to complete
    let paths = workflow.find_paths(start, complete);
    assert!(paths.len() >= 1); // At least one valid path

    // Check node degrees
    assert_eq!(workflow.degree(validate), 3); // 1 in, 2 out
    assert_eq!(workflow.degree(notify), 4); // 3 in, 1 out
}

/// Test type flexibility - graphs with different node and edge types
#[test]
fn test_type_flexibility() {
    // Boolean decision graph
    let mut decision_tree = ContextGraph::<bool, &'static str>::new("Decision Tree");

    let root = decision_tree.add_node(true);
    let left = decision_tree.add_node(false);
    let right = decision_tree.add_node(true);

    decision_tree.add_edge(root, left, "no").unwrap();
    decision_tree.add_edge(root, right, "yes").unwrap();

    // Mixed type graph using enums
    #[derive(Debug, Clone)]
    enum NodeType {
        Number(i32),
        Text(String),
        Flag(bool),
    }

    #[derive(Debug, Clone)]
    enum EdgeType {
        Numeric(f64),
        Labeled(String),
    }

    let mut mixed = ContextGraph::<NodeType, EdgeType>::new("Mixed Types");

    let n1 = mixed.add_node(NodeType::Number(42));
    let n2 = mixed.add_node(NodeType::Text("Hello".to_string()));
    let n3 = mixed.add_node(NodeType::Flag(true));

    mixed.add_edge(n1, n2, EdgeType::Numeric(3.14)).unwrap();
    mixed.add_edge(n2, n3, EdgeType::Labeled("connects to".to_string())).unwrap();

    assert_eq!(mixed.node_count(), 3);
    assert_eq!(mixed.edge_count(), 2);
}

/// Test error handling and invariants
#[test]
fn test_error_handling() {
    let mut graph = ContextGraph::<i32, i32>::new("Error Test");

    let n1 = graph.add_node(1);
    let n2 = graph.add_node(2);

    // Valid edge
    let edge = graph.add_edge(n1, n2, 10).unwrap();

    // Try to add edge with non-existent node
    let fake_node = NodeId::new();
    let result = graph.add_edge(n1, fake_node, 20);
    assert!(result.is_err());

    // Try to add duplicate component
    graph.get_node_mut(n1).unwrap()
        .add_component(Label("First".to_string()))
        .unwrap();

    let duplicate_result = graph.get_node_mut(n1).unwrap()
        .add_component(Label("Second".to_string()));
    assert!(duplicate_result.is_err());

    // Remove node and verify edges are cleaned up
    graph.remove_node(n2);
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0); // Edge was removed with node
}

/// Demonstrate mermaid graph visualization
/// ```mermaid
/// graph TD
///     A[Customer] -->|purchased| B[Product A]
///     A -->|purchased| C[Product B]
///     B -->|related to| C
///     D[Customer 2] -->|purchased| C
/// ```
#[test]
fn test_graph_visualization_structure() {
    let mut graph = ContextGraph::<String, String>::new("Purchase Graph");

    // This test documents the expected structure for visualization
    let customer1 = graph.add_node("Customer 1".to_string());
    let customer2 = graph.add_node("Customer 2".to_string());
    let product_a = graph.add_node("Product A".to_string());
    let product_b = graph.add_node("Product B".to_string());

    // Add visual components
    graph.get_node_mut(customer1).unwrap()
        .add_component(Label("VIP Customer".to_string()))
        .unwrap();

    // Create relationships
    graph.add_edge(customer1, product_a, "purchased".to_string()).unwrap();
    graph.add_edge(customer1, product_b, "purchased".to_string()).unwrap();
    graph.add_edge(product_a, product_b, "related to".to_string()).unwrap();
    graph.add_edge(customer2, product_b, "purchased".to_string()).unwrap();

    // This structure can be visualized using the mermaid diagram above
    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 4);
}
