//! Example: Graph Composition with ContextGraph
//!
//! This example demonstrates how to use ContextGraph for various use cases
//! including workflows, knowledge graphs, and recursive composition.

use cim_contextgraph::{
    ContextGraph, NodeId, Label, Metadata, Component, Subgraph,
};
use std::collections::HashMap;

/// Custom component for workflow states
#[derive(Debug, Clone)]
struct WorkflowState {
    status: WorkflowStatus,
    started_at: u64,
    completed_at: Option<u64>,
}

#[derive(Debug, Clone)]
enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Component for WorkflowState {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "WorkflowState" }
}

fn main() {
    println!("=== CIM ContextGraph Composition Example ===\n");

    // 1. Create a workflow graph
    println!("1. Creating Workflow Graph...");
    let workflow = create_workflow_graph();

    // 2. Create a knowledge graph that contains the workflow
    println!("\n2. Creating Knowledge Graph with Embedded Workflow...");
    let knowledge_graph = create_knowledge_graph(workflow);

    // 3. Demonstrate graph algorithms
    println!("\n3. Running Graph Algorithms...");
    demonstrate_algorithms();

    // 4. Show recursive graph traversal
    println!("\n4. Demonstrating Recursive Graph Traversal...");
    demonstrate_recursive_graphs();

    println!("\n=== Example Complete ===");
}

/// Create a workflow graph representing an order processing pipeline
fn create_workflow_graph() -> ContextGraph<String, String> {
    let mut workflow = ContextGraph::<String, String>::new("Order Processing Workflow");

    // Add workflow steps as nodes
    let receive = workflow.add_node("Receive Order".to_string());
    let validate = workflow.add_node("Validate Payment".to_string());
    let inventory = workflow.add_node("Check Inventory".to_string());
    let pack = workflow.add_node("Pack Items".to_string());
    let ship = workflow.add_node("Ship Order".to_string());
    let notify = workflow.add_node("Notify Customer".to_string());

    // Add workflow state components
    workflow.get_node_mut(receive).unwrap()
        .add_component(WorkflowState {
            status: WorkflowStatus::Completed,
            started_at: 1000,
            completed_at: Some(1100),
        })
        .unwrap();

    workflow.get_node_mut(validate).unwrap()
        .add_component(Label("Critical Step".to_string()))
        .unwrap();

    // Connect workflow steps
    workflow.add_edge(receive, validate, "next".to_string()).unwrap();
    workflow.add_edge(validate, inventory, "if_valid".to_string()).unwrap();
    workflow.add_edge(inventory, pack, "if_available".to_string()).unwrap();
    workflow.add_edge(pack, ship, "when_ready".to_string()).unwrap();
    workflow.add_edge(ship, notify, "after_shipping".to_string()).unwrap();

    // Alternative paths
    workflow.add_edge(validate, notify, "if_invalid".to_string()).unwrap();
    workflow.add_edge(inventory, notify, "if_unavailable".to_string()).unwrap();

    println!("Created workflow with {workflow.graph.node_count(} steps"));

    workflow
}

/// Create a knowledge graph that contains the workflow as a subgraph
fn create_knowledge_graph(workflow: ContextGraph<String, String>) -> ContextGraph<String, String> {
    let mut knowledge = ContextGraph::<String, String>::new("Business Knowledge Graph");

    // Add high-level business concepts
    let orders = knowledge.add_node("Order Management".to_string());
    let customers = knowledge.add_node("Customer Service".to_string());
    let inventory = knowledge.add_node("Inventory System".to_string());
    let shipping = knowledge.add_node("Shipping Partners".to_string());

    // Add the workflow as a subgraph of Order Management
    knowledge.get_node_mut(orders).unwrap()
        .add_component(Subgraph {
            graph: Box::new(workflow)
        })
        .unwrap();

    // Add labels to categorize concepts
    knowledge.get_node_mut(orders).unwrap()
        .add_component(Label("Core Process".to_string()))
        .unwrap();

    knowledge.get_node_mut(customers).unwrap()
        .add_component(Label("Support Function".to_string()))
        .unwrap();

    // Connect business concepts
    knowledge.add_edge(orders, customers, "notifies".to_string()).unwrap();
    knowledge.add_edge(orders, inventory, "checks".to_string()).unwrap();
    knowledge.add_edge(orders, shipping, "uses".to_string()).unwrap();
    knowledge.add_edge(customers, orders, "inquires_about".to_string()).unwrap();

    println!("Created knowledge graph with {knowledge.graph.node_count(} concepts"));

    knowledge
}

/// Demonstrate various graph algorithms
fn demonstrate_algorithms() {
    // Create a dependency graph
    let mut deps = ContextGraph::<&str, &str>::new("Module Dependencies");

    let core = deps.add_node("core");
    let utils = deps.add_node("utils");
    let api = deps.add_node("api");
    let ui = deps.add_node("ui");
    let tests = deps.add_node("tests");

    // Define dependencies
    deps.add_edge(api, core, "depends_on").unwrap();
    deps.add_edge(api, utils, "depends_on").unwrap();
    deps.add_edge(ui, api, "depends_on").unwrap();
    deps.add_edge(ui, utils, "depends_on").unwrap();
    deps.add_edge(tests, ui, "depends_on").unwrap();
    deps.add_edge(tests, api, "depends_on").unwrap();

    // Check for cycles
    println!("Dependency graph is cyclic: {deps.is_cyclic(}"));

    // Get build order using topological sort
    match deps.topological_sort() {
        Ok(order) => {
            println!("Build order:");
            for (i, module_id) in order.iter().enumerate() {
                if let Some(node) = deps.get_node(*module_id) {
                    println!("  {i + 1}. {node.value}");
                }
            }
        }
        Err(_) => println!("Cannot determine build order - cycle detected!"),
    }

    // Find strongly connected components
    let sccs = deps.strongly_connected_components();
    println!("Found {sccs.len(} independent components"));

    // Find paths between nodes
    let paths = deps.all_simple_paths(core, tests, 10);
    println!("Found {paths.len(} paths from 'core' to 'tests'"));
}

/// Demonstrate recursive graph structures
fn demonstrate_recursive_graphs() {
    // Create a system architecture graph
    let mut system = ContextGraph::<String, String>::new("System Architecture");

    // Add main components
    let frontend = system.add_node("Frontend".to_string());
    let backend = system.add_node("Backend".to_string());
    let database = system.add_node("Database".to_string());

    // Create a subgraph for frontend components
    let mut frontend_graph = ContextGraph::<String, String>::new("Frontend Components");
    let ui = frontend_graph.add_node("UI Components".to_string());
    let state = frontend_graph.add_node("State Management".to_string());
    let router = frontend_graph.add_node("Router".to_string());

    frontend_graph.add_edge(ui, state, "updates".to_string()).unwrap();
    frontend_graph.add_edge(router, ui, "renders".to_string()).unwrap();

    // Add frontend subgraph to main graph
    system.get_node_mut(frontend).unwrap()
        .add_component(Subgraph {
            graph: Box::new(frontend_graph)
        })
        .unwrap();

    // Connect main components
    system.add_edge(frontend, backend, "api_calls".to_string()).unwrap();
    system.add_edge(backend, database, "queries".to_string()).unwrap();

    // Count nodes recursively
    let subgraph_nodes = system.get_subgraph_nodes();
    println!("System has {subgraph_nodes.len(} nodes with subgraphs"));

    // Visit recursively
    println!("Recursive graph structure:");
    system.visit_recursive(|graph, depth| {
        let indent = "  ".repeat(depth);
        println!("{indent}Graph: {graph.metadata.properties.get("name"} (nodes: {})")
                     .and_then(|v| v.as_str())
                     .unwrap_or("unnamed"),
                 graph.graph.node_count());
    });
}
