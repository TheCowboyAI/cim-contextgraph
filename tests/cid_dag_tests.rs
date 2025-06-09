//! Tests for CidDag - Content-Addressed DAG for Event Store and Object Store
//!
//! These tests demonstrate how CidDag provides cryptographic integrity
//! for event sourcing and content storage scenarios.

use cim_contextgraph::{
    CidDag, EventDag, ObjectDag,
    EventNode, ObjectNode, CidNode, CidEdge, CidEdgeType,
    Component, GraphError
};
use cid::Cid;
use std::collections::HashMap;

/// Helper to create test CIDs (in real usage, these would be computed from content)
fn create_test_cid(content: &str) -> Cid {
    // In production, this would hash the content
    // For tests, we'll use a deterministic approach
    use cid::multihash::{Code, MultihashDigest};
    let hash = Code::Sha2_256.digest(content.as_bytes());
    Cid::new_v1(0x55, hash) // 0x55 is raw codec
}

/// Test basic event chain creation and verification
#[test]
fn test_event_chain_creation() {
    let mut event_dag = EventDag::new();

    // Create events for an order aggregate
    let event1 = EventNode {
        event_id: "evt-001".to_string(),
        aggregate_id: "order-123".to_string(),
        event_type: "OrderCreated".to_string(),
        sequence: 1,
        payload_cid: None,
    };

    let event2 = EventNode {
        event_id: "evt-002".to_string(),
        aggregate_id: "order-123".to_string(),
        event_type: "PaymentProcessed".to_string(),
        sequence: 2,
        payload_cid: None,
    };

    let event3 = EventNode {
        event_id: "evt-003".to_string(),
        aggregate_id: "order-123".to_string(),
        event_type: "OrderShipped".to_string(),
        sequence: 3,
        payload_cid: None,
    };

    // Create CIDs for events
    let cid1 = create_test_cid("evt-001");
    let cid2 = create_test_cid("evt-002");
    let cid3 = create_test_cid("evt-003");

    // Build the chain
    event_dag.add_event(cid1, None, event1, 1000).unwrap();
    event_dag.add_event(cid2, Some(cid1), event2, 2000).unwrap();
    event_dag.add_event(cid3, Some(cid2), event3, 3000).unwrap();

    // Verify the chain
    let chain = event_dag.verify_chain(&cid3, Some(&cid1)).unwrap();
    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0], cid1);
    assert_eq!(chain[1], cid2);
    assert_eq!(chain[2], cid3);

    // Get all events for the aggregate
    let aggregate_events = event_dag.get_aggregate_events("order-123");
    assert_eq!(aggregate_events.len(), 3);

    // Verify they're in sequence order
    assert_eq!(aggregate_events[0].1.sequence, 1);
    assert_eq!(aggregate_events[1].1.sequence, 2);
    assert_eq!(aggregate_events[2].1.sequence, 3);
}

/// Test event with object store references
#[test]
fn test_event_with_object_references() {
    let mut event_dag = EventDag::new();
    let mut object_dag = ObjectDag::new();

    // First, store a large object (e.g., invoice PDF)
    let invoice_object = ObjectNode {
        object_type: "application/pdf".to_string(),
        size: 1024 * 50, // 50KB
        mime_type: Some("application/pdf".to_string()),
        chunks: vec![], // No chunking for small file
    };

    let invoice_cid = create_test_cid("invoice-content");
    object_dag.add_object(invoice_cid, invoice_object, 1000).unwrap();

    // Create an event that references the object
    let event = EventNode {
        event_id: "evt-invoice".to_string(),
        aggregate_id: "order-456".to_string(),
        event_type: "InvoiceGenerated".to_string(),
        sequence: 1,
        payload_cid: Some(invoice_cid), // Reference to object store
    };

    let event_cid = create_test_cid("evt-invoice");
    event_dag.add_event(event_cid, None, event, 2000).unwrap();

    // Verify the reference was created
    let stored_event = event_dag.get_node(&event_cid).unwrap();
    assert_eq!(stored_event.content.payload_cid, Some(invoice_cid));
}

/// Test chunked object storage
#[test]
fn test_chunked_object_storage() {
    let mut object_dag = ObjectDag::new();

    // Store chunks first
    let chunk1_cid = create_test_cid("chunk-1");
    let chunk2_cid = create_test_cid("chunk-2");
    let chunk3_cid = create_test_cid("chunk-3");

    let chunk1 = ObjectNode {
        object_type: "chunk".to_string(),
        size: 1024 * 1024, // 1MB each
        mime_type: None,
        chunks: vec![],
    };

    object_dag.add_object(chunk1_cid, chunk1.clone(), 1000).unwrap();
    object_dag.add_object(chunk2_cid, chunk1.clone(), 1001).unwrap();
    object_dag.add_object(chunk3_cid, chunk1.clone(), 1002).unwrap();

    // Store main object referencing chunks
    let large_file = ObjectNode {
        object_type: "video/mp4".to_string(),
        size: 1024 * 1024 * 3, // 3MB total
        mime_type: Some("video/mp4".to_string()),
        chunks: vec![chunk1_cid, chunk2_cid, chunk3_cid],
    };

    let file_cid = create_test_cid("large-video");
    object_dag.add_object(file_cid, large_file, 2000).unwrap();

    // Retrieve chunks
    let chunks = object_dag.get_object_chunks(&file_cid);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], chunk1_cid);
    assert_eq!(chunks[1], chunk2_cid);
    assert_eq!(chunks[2], chunk3_cid);
}

/// Test DAG properties - no cycles allowed
#[test]
fn test_dag_cycle_prevention() {
    let mut dag = CidDag::<String>::new();

    let cid1 = create_test_cid("node-1");
    let cid2 = create_test_cid("node-2");
    let cid3 = create_test_cid("node-3");

    // Create a valid chain
    dag.add_node(cid1, "First".to_string(), 1000).unwrap();
    dag.add_node(cid2, "Second".to_string(), 2000).unwrap();
    dag.add_node(cid3, "Third".to_string(), 3000).unwrap();

    dag.add_causal_edge(cid1, cid2).unwrap();
    dag.add_causal_edge(cid2, cid3).unwrap();

    // Try to create a cycle
    let cycle_result = dag.add_causal_edge(cid3, cid1);
    assert!(matches!(cycle_result, Err(GraphError::CycleDetected)));
}

/// Test finding common ancestors (useful for merge operations)
#[test]
fn test_common_ancestor_finding() {
    let mut event_dag = EventDag::new();

    // Create a fork in the event chain
    //       root
    //      /    \
    //    e1      e2
    //    |       |
    //    e3      e4

    let root_cid = create_test_cid("root");
    let e1_cid = create_test_cid("e1");
    let e2_cid = create_test_cid("e2");
    let e3_cid = create_test_cid("e3");
    let e4_cid = create_test_cid("e4");

    // Add root event
    event_dag.add_event(root_cid, None, EventNode {
        event_id: "root".to_string(),
        aggregate_id: "agg-1".to_string(),
        event_type: "Created".to_string(),
        sequence: 1,
        payload_cid: None,
    }, 1000).unwrap();

    // Fork into two branches
    event_dag.add_event(e1_cid, Some(root_cid), EventNode {
        event_id: "e1".to_string(),
        aggregate_id: "agg-1".to_string(),
        event_type: "BranchA".to_string(),
        sequence: 2,
        payload_cid: None,
    }, 2000).unwrap();

    event_dag.add_event(e2_cid, Some(root_cid), EventNode {
        event_id: "e2".to_string(),
        aggregate_id: "agg-1".to_string(),
        event_type: "BranchB".to_string(),
        sequence: 2,
        payload_cid: None,
    }, 2001).unwrap();

    // Continue branches
    event_dag.add_event(e3_cid, Some(e1_cid), EventNode {
        event_id: "e3".to_string(),
        aggregate_id: "agg-1".to_string(),
        event_type: "ContinueA".to_string(),
        sequence: 3,
        payload_cid: None,
    }, 3000).unwrap();

    event_dag.add_event(e4_cid, Some(e2_cid), EventNode {
        event_id: "e4".to_string(),
        aggregate_id: "agg-1".to_string(),
        event_type: "ContinueB".to_string(),
        sequence: 3,
        payload_cid: None,
    }, 3001).unwrap();

    // Find common ancestor of the two branch tips
    let common = event_dag.common_ancestor(&e3_cid, &e4_cid);
    assert_eq!(common, Some(root_cid));
}

/// Test converting CidDag to ContextGraph for visualization
#[test]
fn test_dag_to_context_graph_conversion() {
    let mut event_dag = EventDag::new();

    // Create a simple chain
    let cid1 = create_test_cid("evt-1");
    let cid2 = create_test_cid("evt-2");

    event_dag.add_event(cid1, None, EventNode {
        event_id: "evt-1".to_string(),
        aggregate_id: "agg-1".to_string(),
        event_type: "Started".to_string(),
        sequence: 1,
        payload_cid: None,
    }, 1000).unwrap();

    event_dag.add_event(cid2, Some(cid1), EventNode {
        event_id: "evt-2".to_string(),
        aggregate_id: "agg-1".to_string(),
        event_type: "Completed".to_string(),
        sequence: 2,
        payload_cid: None,
    }, 2000).unwrap();

    // Convert to ContextGraph for visualization
    let context_graph = event_dag.to_context_graph();

    // Verify structure is preserved
    assert_eq!(context_graph.nodes.len(), 2);
    assert_eq!(context_graph.edges.len(), 1);

    // Verify CID components are attached
    let nodes_with_cid = context_graph.query_nodes_with_component::<crate::CidReference>();
    assert_eq!(nodes_with_cid.len(), 2);
}

/// Demonstrate event sourcing pattern with mermaid visualization
/// ```mermaid
/// graph LR
///     E1[OrderCreated<br/>CID: abc123] -->|previous| E2[PaymentProcessed<br/>CID: def456]
///     E2 -->|previous| E3[OrderShipped<br/>CID: ghi789]
///     E2 -->|references| O1[Invoice PDF<br/>CID: xyz999]
///
///     style E1 fill:#f9f,stroke:#333,stroke-width:2px
///     style E2 fill:#f9f,stroke:#333,stroke-width:2px
///     style E3 fill:#f9f,stroke:#333,stroke-width:2px
///     style O1 fill:#9ff,stroke:#333,stroke-width:2px
/// ```
#[test]
fn test_event_sourcing_pattern() {
    let mut event_dag = EventDag::new();

    // This test demonstrates the typical event sourcing pattern
    // where events form a chain with optional references to objects

    let order_created = EventNode {
        event_id: "evt-001".to_string(),
        aggregate_id: "order-789".to_string(),
        event_type: "OrderCreated".to_string(),
        sequence: 1,
        payload_cid: None,
    };

    let invoice_cid = create_test_cid("invoice-pdf");
    let payment_processed = EventNode {
        event_id: "evt-002".to_string(),
        aggregate_id: "order-789".to_string(),
        event_type: "PaymentProcessed".to_string(),
        sequence: 2,
        payload_cid: Some(invoice_cid), // References invoice in object store
    };

    let order_shipped = EventNode {
        event_id: "evt-003".to_string(),
        aggregate_id: "order-789".to_string(),
        event_type: "OrderShipped".to_string(),
        sequence: 3,
        payload_cid: None,
    };

    // Build the event chain
    let cid1 = create_test_cid("evt-001-content");
    let cid2 = create_test_cid("evt-002-content");
    let cid3 = create_test_cid("evt-003-content");

    event_dag.add_event(cid1, None, order_created, 1000).unwrap();
    event_dag.add_event(cid2, Some(cid1), payment_processed, 2000).unwrap();
    event_dag.add_event(cid3, Some(cid2), order_shipped, 3000).unwrap();

    // Verify integrity
    let chain = event_dag.verify_chain(&cid3, Some(&cid1)).unwrap();
    assert_eq!(chain.len(), 3);

    // Get latest state
    let latest = event_dag.latest_cids();
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0], cid3);
}

/// Test error handling for invalid operations
#[test]
fn test_error_handling() {
    let mut dag = CidDag::<String>::new();

    let cid1 = create_test_cid("node-1");
    let cid2 = create_test_cid("node-2");
    let fake_cid = create_test_cid("fake");

    // Add a node
    dag.add_node(cid1, "First".to_string(), 1000).unwrap();

    // Try to add duplicate CID
    let duplicate_result = dag.add_node(cid1, "Duplicate".to_string(), 2000);
    assert!(matches!(duplicate_result, Err(GraphError::DuplicateCid(_))));

    // Try to add edge with non-existent node
    let edge_result = dag.add_causal_edge(cid1, fake_cid);
    assert!(matches!(edge_result, Err(GraphError::CidNotFound(_))));

    // Try to verify chain with wrong root
    dag.add_node(cid2, "Second".to_string(), 2000).unwrap();
    dag.add_causal_edge(cid1, cid2).unwrap();

    let verify_result = dag.verify_chain(&cid2, Some(&fake_cid));
    assert!(matches!(verify_result, Err(GraphError::ChainVerificationFailed)));
}
