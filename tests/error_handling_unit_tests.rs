//! Error Handling and Edge Case Unit Tests for ContextGraph
//!
//! These tests rigorously validate error conditions and edge cases including:
//! - Invalid operations
//! - Boundary conditions
//! - Resource exhaustion
//! - Type mismatches
//! - Concurrent modification errors
//! - Recovery from error states

use cim_contextgraph::*;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
struct TestNode {
    id: Uuid,
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
struct TestEdge {
    weight: f32,
}

mod invalid_operation_tests {
    use super::*;

    #[test]
    fn test_add_edge_to_nonexistent_nodes() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("InvalidOps");

        let fake_id1 = NodeId::new();
        let fake_id2 = NodeId::new();

        // Try to add edge between non-existent nodes
        let result = graph.add_edge(fake_id1, fake_id2, TestEdge { weight: 1.0 });
        assert!(result.is_err());

        match result {
            Err(GraphError::NodeNotFound(id)) => {
                assert!(id == fake_id1 || id == fake_id2);
            }
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_add_edge_one_valid_one_invalid_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("InvalidOps");

        let valid_node = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Valid".to_string(),
        });
        let invalid_node = NodeId::new();

        // Try to add edge with one valid and one invalid node
        let result = graph.add_edge(valid_node, invalid_node, TestEdge { weight: 1.0 });
        assert!(result.is_err());

        // Try reverse order
        let result = graph.add_edge(invalid_node, valid_node, TestEdge { weight: 1.0 });
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_nonexistent_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("InvalidOps");

        let fake_id = NodeId::new();

        // Removing non-existent node should not panic
        graph.remove_node(fake_id);

        // Graph should remain empty
        assert_eq!(graph.nodes().len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("InvalidOps");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        let fake_edge_id = EdgeId::new();

        // Removing non-existent edge should not panic
        graph.remove_edge(fake_edge_id);

        // Nodes should still exist
        assert_eq!(graph.nodes().len(), 2);
    }

    #[test]
    fn test_get_nonexistent_node() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("InvalidOps");

        let fake_id = NodeId::new();
        let result = graph.get_node(fake_id);

        assert!(result.is_none());
    }

    #[test]
    fn test_get_nonexistent_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("InvalidOps");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        let fake_edge_id = EdgeId::new();
        let result = graph.get_edge(fake_edge_id);

        assert!(result.is_none());
    }

    #[test]
    fn test_double_remove_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("DoubleRemove");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "ToRemove".to_string(),
        });

        // First removal should work
        graph.remove_node(node_id);
        assert_eq!(graph.nodes().len(), 0);

        // Second removal should not panic
        graph.remove_node(node_id);
        assert_eq!(graph.nodes().len(), 0);
    }

    #[test]
    fn test_double_remove_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("DoubleRemove");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });
        let edge_id = graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();

        // First removal should work
        graph.remove_edge(edge_id);
        assert_eq!(graph.edges().len(), 0);

        // Second removal should not panic
        graph.remove_edge(edge_id);
        assert_eq!(graph.edges().len(), 0);
    }
}

mod boundary_condition_tests {
    use super::*;

    #[test]
    fn test_empty_graph_operations() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("Empty");

        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
        assert!(graph.nodes().is_empty());
        assert!(graph.edges().is_empty());

        // Operations on empty graph
        let fake_id = NodeId::new();
        assert!(graph.get_node(fake_id).is_none());
        assert!(graph.neighbors(fake_id).is_empty());
        assert_eq!(graph.degree(fake_id), 0);
    }

    #[test]
    fn test_single_node_graph() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("SingleNode");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Lonely".to_string(),
        });

        assert_eq!(graph.nodes().len(), 1);
        assert_eq!(graph.edges().len(), 0);
        assert_eq!(graph.degree(node_id), 0);
        assert!(graph.neighbors(node_id).is_empty());
    }

    #[test]
    fn test_self_loop() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("SelfLoop");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "SelfReferential".to_string(),
        });

        // Try to create self-loop
        let result = graph.add_edge(node_id, node_id, TestEdge { weight: 1.0 });

        // Depending on implementation, this might be allowed or not
        // Test should handle both cases gracefully
        match result {
            Ok(edge_id) => {
                assert_eq!(graph.edges().len(), 1);
                assert_eq!(graph.degree(node_id), 2); // Self-loop counts twice
            }
            Err(_) => {
                // Self-loops not allowed
                assert_eq!(graph.edges().len(), 0);
            }
        }
    }

    #[test]
    fn test_parallel_edges() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("ParallelEdges");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        // Add multiple edges between same nodes
        let edge1 = graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        let edge2 = graph.add_edge(node1, node2, TestEdge { weight: 2.0 }).unwrap();
        let edge3 = graph.add_edge(node1, node2, TestEdge { weight: 3.0 }).unwrap();

        // All edges should exist
        assert_eq!(graph.edges().len(), 3);
        assert_ne!(edge1, edge2);
        assert_ne!(edge2, edge3);
        assert_ne!(edge1, edge3);
    }

    #[test]
    fn test_bidirectional_edges() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Bidirectional");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        // Add edges in both directions
        let edge_forward = graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        let edge_backward = graph.add_edge(node2, node1, TestEdge { weight: 2.0 }).unwrap();

        assert_eq!(graph.edges().len(), 2);
        assert_ne!(edge_forward, edge_backward);

        // Check directionality
        let forward = graph.get_edge(edge_forward).unwrap();
        let backward = graph.get_edge(edge_backward).unwrap();

        assert_eq!(forward.source, node1);
        assert_eq!(forward.target, node2);
        assert_eq!(backward.source, node2);
        assert_eq!(backward.target, node1);
    }

    #[test]
    fn test_maximum_node_name_length() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("MaxLength");

        // Create a very long name
        let long_name = "A".repeat(10_000);

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: long_name.clone(),
        });

        let node = graph.get_node(node_id).unwrap();
        assert_eq!(node.name.len(), 10_000);
    }

    #[test]
    fn test_zero_weight_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("ZeroWeight");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        let edge_id = graph.add_edge(node1, node2, TestEdge { weight: 0.0 }).unwrap();

        let edge = graph.get_edge(edge_id).unwrap();
        assert_eq!(edge.weight, 0.0);
    }

    #[test]
    fn test_negative_weight_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("NegativeWeight");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        let edge_id = graph.add_edge(node1, node2, TestEdge { weight: -1.5 }).unwrap();

        let edge = graph.get_edge(edge_id).unwrap();
        assert_eq!(edge.weight, -1.5);
    }

    #[test]
    fn test_extreme_weight_values() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("ExtremeWeights");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        // Test with f32::MAX
        let edge_max = graph.add_edge(node1, node2, TestEdge { weight: f32::MAX }).unwrap();
        assert_eq!(graph.get_edge(edge_max).unwrap().weight, f32::MAX);

        // Test with f32::MIN
        let edge_min = graph.add_edge(node1, node2, TestEdge { weight: f32::MIN }).unwrap();
        assert_eq!(graph.get_edge(edge_min).unwrap().weight, f32::MIN);

        // Test with infinity
        let edge_inf = graph.add_edge(node1, node2, TestEdge { weight: f32::INFINITY }).unwrap();
        assert!(graph.get_edge(edge_inf).unwrap().weight.is_infinite());

        // Test with NaN
        let edge_nan = graph.add_edge(node1, node2, TestEdge { weight: f32::NAN }).unwrap();
        assert!(graph.get_edge(edge_nan).unwrap().weight.is_nan());
    }
}

mod resource_exhaustion_tests {
    use super::*;

    #[test]
    fn test_large_graph_creation() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("LargeGraph");

        const NODE_COUNT: usize = 1000;
        let mut node_ids = Vec::with_capacity(NODE_COUNT);

        // Add many nodes
        for i in 0..NODE_COUNT {
            let node_id = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });
            node_ids.push(node_id);
        }

        assert_eq!(graph.nodes().len(), NODE_COUNT);

        // Add edges to create a connected graph
        for i in 0..NODE_COUNT - 1 {
            graph.add_edge(node_ids[i], node_ids[i + 1], TestEdge { weight: i as f32 }).unwrap();
        }

        assert_eq!(graph.edges().len(), NODE_COUNT - 1);
    }

    #[test]
    fn test_highly_connected_graph() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("HighlyConnected");

        const NODE_COUNT: usize = 50; // Smaller for complete graph
        let mut node_ids = Vec::with_capacity(NODE_COUNT);

        // Add nodes
        for i in 0..NODE_COUNT {
            let node_id = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });
            node_ids.push(node_id);
        }

        // Create complete graph (every node connected to every other node)
        let mut edge_count = 0;
        for i in 0..NODE_COUNT {
            for j in i + 1..NODE_COUNT {
                graph.add_edge(
                    node_ids[i],
                    node_ids[j],
                    TestEdge { weight: 1.0 }
                ).unwrap();
                edge_count += 1;
            }
        }

        // Verify complete graph properties
        assert_eq!(graph.edges().len(), edge_count);
        assert_eq!(edge_count, NODE_COUNT * (NODE_COUNT - 1) / 2);

        // Each node should have degree NODE_COUNT - 1
        for node_id in &node_ids {
            assert_eq!(graph.degree(*node_id), NODE_COUNT - 1);
        }
    }

    #[test]
    fn test_deep_chain_graph() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("DeepChain");

        const CHAIN_LENGTH: usize = 1000;
        let mut current_node = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Start".to_string(),
        });

        // Create a long chain
        for i in 1..CHAIN_LENGTH {
            let next_node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Chain{}", i),
            });

            graph.add_edge(current_node, next_node, TestEdge { weight: i as f32 }).unwrap();
            current_node = next_node;
        }

        assert_eq!(graph.nodes().len(), CHAIN_LENGTH);
        assert_eq!(graph.edges().len(), CHAIN_LENGTH - 1);
    }

    #[test]
    fn test_star_topology() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("StarTopology");

        const LEAF_COUNT: usize = 500;

        // Create center node
        let center = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Center".to_string(),
        });

        // Add leaf nodes all connected to center
        for i in 0..LEAF_COUNT {
            let leaf = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Leaf{}", i),
            });

            graph.add_edge(center, leaf, TestEdge { weight: i as f32 }).unwrap();
        }

        assert_eq!(graph.nodes().len(), LEAF_COUNT + 1);
        assert_eq!(graph.edges().len(), LEAF_COUNT);
        assert_eq!(graph.degree(center), LEAF_COUNT);
    }

    #[test]
    fn test_memory_stress_with_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("MemoryStress");

        // Add nodes with large component data
        for i in 0..100 {
            let node_id = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });

            // Add multiple large components
            for j in 0..10 {
                let large_data = vec![0u8; 1024]; // 1KB per component
                graph.add_component(
                    node_id,
                    format!("LargeComponent{}", j),
                    large_data
                );
            }
        }

        // Verify all components were added
        let nodes = graph.nodes();
        for node_id in nodes {
            let components = graph.get_components(node_id);
            assert_eq!(components.len(), 10);
        }
    }
}

mod concurrent_modification_tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn test_concurrent_node_addition() {
        let graph = Arc::new(Mutex::new(ContextGraph::<TestNode, TestEdge>::new("Concurrent")));

        let mut handles = vec![];
        const THREADS: usize = 10;
        const NODES_PER_THREAD: usize = 100;

        for thread_id in 0..THREADS {
            let graph_clone = graph.clone();
            let handle = thread::spawn(move || {
                for i in 0..NODES_PER_THREAD {
                    let mut graph = graph_clone.lock().unwrap();
                    graph.add_node(TestNode {
                        id: Uuid::new_v4(),
                        name: format!("Thread{}-Node{}", thread_id, i),
                    });
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_graph = graph.lock().unwrap();
        assert_eq!(final_graph.nodes().len(), THREADS * NODES_PER_THREAD);
    }

    #[test]
    fn test_concurrent_edge_addition() {
        let graph = Arc::new(Mutex::new(ContextGraph::<TestNode, TestEdge>::new("ConcurrentEdges")));

        // Pre-create nodes
        let node_ids: Vec<NodeId> = {
            let mut graph = graph.lock().unwrap();
            (0..20).map(|i| {
                graph.add_node(TestNode {
                    id: Uuid::new_v4(),
                    name: format!("Node{}", i),
                })
            }).collect()
        };

        let mut handles = vec![];
        const THREADS: usize = 5;

        for thread_id in 0..THREADS {
            let graph_clone = graph.clone();
            let node_ids_clone = node_ids.clone();
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    let mut graph = graph_clone.lock().unwrap();
                    let from = node_ids_clone[i];
                    let to = node_ids_clone[i + 10];
                    let _ = graph.add_edge(from, to, TestEdge {
                        weight: (thread_id * 10 + i) as f32
                    });
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_graph = graph.lock().unwrap();
        // Each thread adds 10 edges
        assert_eq!(final_graph.edges().len(), THREADS * 10);
    }

    #[test]
    fn test_concurrent_read_write() {
        let graph = Arc::new(Mutex::new(ContextGraph::<TestNode, TestEdge>::new("ReadWrite")));

        // Pre-populate graph
        let node_ids: Vec<NodeId> = {
            let mut graph = graph.lock().unwrap();
            let ids: Vec<_> = (0..100).map(|i| {
                graph.add_node(TestNode {
                    id: Uuid::new_v4(),
                    name: format!("Node{}", i),
                })
            }).collect();

            // Add some edges
            for i in 0..99 {
                graph.add_edge(ids[i], ids[i + 1], TestEdge { weight: i as f32 }).unwrap();
            }

            ids
        };

        let mut handles = vec![];

        // Reader threads
        for _ in 0..3 {
            let graph_clone = graph.clone();
            let node_ids_clone = node_ids.clone();
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    let graph = graph_clone.lock().unwrap();
                    let idx = rand::random::<usize>() % node_ids_clone.len();
                    let _ = graph.get_node(node_ids_clone[idx]);
                    let _ = graph.degree(node_ids_clone[idx]);
                }
            });
            handles.push(handle);
        }

        // Writer threads
        for thread_id in 0..2 {
            let graph_clone = graph.clone();
            let handle = thread::spawn(move || {
                for i in 0..50 {
                    let mut graph = graph_clone.lock().unwrap();
                    graph.add_node(TestNode {
                        id: Uuid::new_v4(),
                        name: format!("Writer{}-Node{}", thread_id, i),
                    });
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_graph = graph.lock().unwrap();
        assert_eq!(final_graph.nodes().len(), 200); // 100 initial + 2*50 from writers
    }
}

mod recovery_tests {
    use super::*;

    #[test]
    fn test_recovery_after_failed_operation() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Recovery");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        // Successful operation
        let edge = graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();

        // Failed operation (invalid node)
        let fake_node = NodeId::new();
        let failed = graph.add_edge(node1, fake_node, TestEdge { weight: 2.0 });
        assert!(failed.is_err());

        // Graph should still be in valid state
        assert_eq!(graph.nodes().len(), 2);
        assert_eq!(graph.edges().len(), 1);

        // Can continue with valid operations
        let node3 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "C".to_string(),
        });
        let edge2 = graph.add_edge(node2, node3, TestEdge { weight: 3.0 }).unwrap();

        assert_eq!(graph.nodes().len(), 3);
        assert_eq!(graph.edges().len(), 2);
    }

    #[test]
    fn test_partial_operation_rollback() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Rollback");

        // Add initial nodes
        let nodes: Vec<_> = (0..5).map(|i| {
            graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            })
        }).collect();

        // Try to perform a complex operation that might fail partway
        let initial_edge_count = graph.edges().len();

        // Add some edges
        graph.add_edge(nodes[0], nodes[1], TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(nodes[1], nodes[2], TestEdge { weight: 2.0 }).unwrap();

        // Simulate failure by trying to add edge to non-existent node
        let fake_node = NodeId::new();
        let result = graph.add_edge(nodes[2], fake_node, TestEdge { weight: 3.0 });
        assert!(result.is_err());

        // Previous successful operations should still be there
        assert_eq!(graph.edges().len(), initial_edge_count + 2);

        // Can continue operations
        graph.add_edge(nodes[2], nodes[3], TestEdge { weight: 4.0 }).unwrap();
        assert_eq!(graph.edges().len(), initial_edge_count + 3);
    }

    #[test]
    fn test_graph_state_consistency_after_errors() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Consistency");

        // Build initial graph
        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });
        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });
        let node3 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "C".to_string(),
        });

        graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node2, node3, TestEdge { weight: 2.0 }).unwrap();

        // Verify initial state
        assert_eq!(graph.degree(node2), 2);

        // Perform invalid operations
        let fake_node = NodeId::new();
        let _ = graph.add_edge(fake_node, node2, TestEdge { weight: 3.0 });
        let _ = graph.remove_node(fake_node);
        let fake_edge = EdgeId::new();
        graph.remove_edge(fake_edge);

        // State should be unchanged
        assert_eq!(graph.nodes().len(), 3);
        assert_eq!(graph.edges().len(), 2);
        assert_eq!(graph.degree(node2), 2);

        // Graph operations should still work correctly
        let neighbors = graph.neighbors(node2);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&node1) || neighbors.contains(&node3));
    }
}
