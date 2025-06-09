//! Example: Composing Different Graph Types in CIM
//!
//! This example demonstrates how ContextGraph, CidDag, and future graph types
//! work together to create a complete system.

use cim_contextgraph::{
    ContextGraph, CidDag, EventDag, ObjectDag,
    EventNode, ObjectNode, NodeId, EdgeId,
    Label, Metadata, Component, Subgraph,
};
use cid::Cid;
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
}

/// Create a test CID from content
fn create_cid(content: &str) -> Cid {
    use cid::multihash::{Code, MultihashDigest};
    let hash = Code::Sha2_256.digest(content.as_bytes());
    Cid::new_v1(0x55, hash)
}

fn main() {
    println!("=== CIM Graph Composition Example ===\n");

    // 1. Create a workflow graph using ContextGraph
    println!("1. Creating Workflow Graph...");
    let mut workflow = create_workflow_graph();

    // 2. Create an event store using EventDag
    println!("\n2. Creating Event Store...");
    let mut event_store = create_event_store();

    // 3. Create an object store using ObjectDag
    println!("\n3. Creating Object Store...");
    let mut object_store = create_object_store();

    // 4. Create a knowledge graph that references the workflow
    println!("\n4. Creating Knowledge Graph...");
    let knowledge_graph = create_knowledge_graph(workflow);

    // 5. Convert EventDag to ContextGraph for visualization
    println!("\n5. Converting Event Store for Visualization...");
    visualize_event_store(&event_store);

    // 6. Demonstrate graph algorithms
    println!("\n6. Running Graph Algorithms...");
    demonstrate_algorithms();

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

    // Connect workflow steps
    workflow.add_edge(receive, validate, "next".to_string()).unwrap();
    workflow.add_edge(validate, inventory, "if_valid".to_string()).unwrap();
    workflow.add_edge(inventory, pack, "if_available".to_string()).unwrap();
    workflow.add_edge(pack, ship, "when_ready".to_string()).unwrap();
    workflow.add_edge(ship, notify, "after_shipping".to_string()).unwrap();

    // Alternative paths
    workflow.add_edge(validate, notify, "if_invalid".to_string()).unwrap();
    workflow.add_edge(inventory, notify, "if_unavailable".to_string()).unwrap();

    println!("Created workflow with {} steps and {} transitions",
             workflow.nodes.len(), workflow.edges.len());

    workflow
}

/// Create an event store tracking order events
fn create_event_store() -> EventDag {
    let mut event_store = EventDag::new();

    // Create events for order processing
    let events = vec![
        ("order-created", "OrderCreated", 1, None),
        ("payment-validated", "PaymentValidated", 2, None),
        ("inventory-checked", "InventoryChecked", 3, None),
        ("items-packed", "ItemsPacked", 4, Some(create_cid("packing-slip"))),
        ("order-shipped", "OrderShipped", 5, Some(create_cid("shipping-label"))),
        ("customer-notified", "CustomerNotified", 6, None),
    ];

    let mut previous_cid = None;

    for (event_id, event_type, sequence, payload_cid) in events {
        let event = EventNode {
            event_id: event_id.to_string(),
            aggregate_id: "order-12345".to_string(),
            event_type: event_type.to_string(),
            sequence,
            payload_cid,
        };

        let event_cid = create_cid(&format!("{}-content", event_id));
        event_store.add_event(event_cid, previous_cid, event, sequence * 1000).unwrap();
        previous_cid = Some(event_cid);

        println!("Added event: {} (sequence: {})", event_type, sequence);
    }

    // Verify the chain
    if let Some(last_cid) = previous_cid {
        let chain = event_store.verify_chain(&last_cid, None).unwrap();
        println!("Event chain verified with {} events", chain.len());
    }

    event_store
}

/// Create an object store for documents
fn create_object_store() -> ObjectDag {
    let mut object_store = ObjectDag::new();

    // Store a packing slip document
    let packing_slip = ObjectNode {
        object_type: "application/pdf".to_string(),
        size: 1024 * 25, // 25KB
        mime_type: Some("application/pdf".to_string()),
        chunks: vec![],
    };

    let packing_cid = create_cid("packing-slip");
    object_store.add_object(packing_cid, packing_slip, 4000).unwrap();

    // Store a shipping label with chunks
    let chunk_cids: Vec<Cid> = (0..3)
        .map(|i| {
            let chunk = ObjectNode {
                object_type: "chunk".to_string(),
                size: 1024 * 100, // 100KB each
                mime_type: None,
                chunks: vec![],
            };
            let cid = create_cid(&format!("shipping-label-chunk-{}", i));
            object_store.add_object(cid, chunk, 5000 + i).unwrap();
            cid
        })
        .collect();

    let shipping_label = ObjectNode {
        object_type: "image/png".to_string(),
        size: 1024 * 300, // 300KB total
        mime_type: Some("image/png".to_string()),
        chunks: chunk_cids,
    };

    let shipping_cid = create_cid("shipping-label");
    object_store.add_object(shipping_cid, shipping_label, 5100).unwrap();

    println!("Stored {} objects in object store", 5); // 2 documents + 3 chunks

    object_store
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

    println!("Created knowledge graph with {} concepts", knowledge.nodes.len());

    // Count total nodes including workflow subgraph
    let total = knowledge.total_node_count();
    println!("Total nodes including subgraphs: {}", total);

    knowledge
}

/// Convert EventDag to ContextGraph for visualization
fn visualize_event_store(event_store: &EventDag) {
    let context_graph = event_store.to_context_graph();

    println!("Converted EventDag to ContextGraph:");
    println!("  - {} nodes (events)", context_graph.nodes.len());
    println!("  - {} edges (causal links)", context_graph.edges.len());

    // The context graph can now be visualized using standard graph visualization tools
    // Each node has a CidReference component for lookup
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
    println!("Dependency graph is cyclic: {}", deps.is_cyclic());

    // Get build order using topological sort
    match deps.topological_sort() {
        Ok(order) => {
            println!("Build order:");
            for (i, module) in order.iter().enumerate() {
                if let Some(node) = deps.get_node(*module) {
                    println!("  {}. {}", i + 1, node.value);
                }
            }
        }
        Err(_) => println!("Cannot determine build order - cycle detected!"),
    }

    // Find strongly connected components
    let sccs = deps.strongly_connected_components();
    println!("Found {} independent components", sccs.len());
}

/// Example output visualization
/// ```mermaid
/// graph TB
///     subgraph "Knowledge Graph"
///         OM[Order Management] -->|notifies| CS[Customer Service]
///         OM -->|checks| IS[Inventory System]
///         OM -->|uses| SP[Shipping Partners]
///         CS -->|inquires_about| OM
///
///         subgraph "Order Workflow"
///             RO[Receive Order] --> VP[Validate Payment]
///             VP -->|if_valid| CI[Check Inventory]
///             CI -->|if_available| PI[Pack Items]
///             PI --> SO[Ship Order]
///             SO --> NC[Notify Customer]
///         end
///     end
///
///     subgraph "Event Store"
///         E1[OrderCreated] -->|next| E2[PaymentValidated]
///         E2 -->|next| E3[InventoryChecked]
///         E3 -->|next| E4[ItemsPacked]
///         E4 -->|next| E5[OrderShipped]
///         E5 -->|next| E6[CustomerNotified]
///     end
/// ```
