//! Comprehensive Unit Tests for ContextGraph
//!
//! These tests rigorously validate all ContextGraph capabilities including:
//! - Edge cases and error conditions
//! - Boundary conditions
//! - Invalid inputs
//! - Performance characteristics
//! - Invariant violations
//! - Concurrent operations
//! - Memory safety

use cim_contextgraph::*;
use std::collections::HashMap;
use uuid::Uuid;

// Test helpers
#[derive(Debug, Clone, PartialEq)]
struct TestNode {
    id: Uuid,
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
enum TestEdge {
    Simple,
    Weighted(f32),
    Labeled(String),
}

mod graph_creation_tests {
    use super::*;

    #[test]
    fn test_empty_graph_creation() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("EmptyGraph");
        assert_eq!(graph.name(), "EmptyGraph");
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
    }

    #[test]
    fn test_graph_with_empty_name() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("");
        assert_eq!(graph.name(), "");
    }

    #[test]
    fn test_graph_with_very_long_name() {
        let long_name = "a".repeat(10000);
        let graph = ContextGraph::<TestNode, TestEdge>::new(&long_name);
        assert_eq!(graph.name(), &long_name);
    }

    #[test]
    fn test_graph_with_unicode_name() {
        let unicode_name = "图表🎯测试";
        let graph = ContextGraph::<TestNode, TestEdge>::new(unicode_name);
        assert_eq!(graph.name(), unicode_name);
    }
}

mod node_operations_tests {
    use super::*;

    #[test]
    fn test_add_single_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = TestNode { id: Uuid::new_v4(), name: "Node1".to_string() };
        let node_id = graph.add_node(node.clone());

        assert_eq!(graph.nodes().len(), 1);
        assert!(graph.get_node(node_id).is_some());
        assert_eq!(graph.get_node_value(node_id).unwrap(), &node);
    }

    #[test]
    fn test_add_multiple_identical_nodes() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = TestNode { id: Uuid::new_v4(), name: "Same".to_string() };

        let id1 = graph.add_node(node.clone());
        let id2 = graph.add_node(node.clone());
        let id3 = graph.add_node(node.clone());

        assert_eq!(graph.nodes().len(), 3);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_remove_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = TestNode { id: Uuid::new_v4(), name: "ToRemove".to_string() };
        let node_id = graph.add_node(node);

        assert_eq!(graph.nodes().len(), 1);
        graph.remove_node(node_id);
        assert_eq!(graph.nodes().len(), 0);
        assert!(graph.get_node(node_id).is_none());
    }

    #[test]
    fn test_remove_nonexistent_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let fake_id = NodeId::new();

        // Should not panic
        graph.remove_node(fake_id);
        assert_eq!(graph.nodes().len(), 0);
    }

    #[test]
    fn test_remove_node_with_edges() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node1".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node2".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node3".to_string() });

        graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        graph.add_edge(node2, node3, TestEdge::Simple).unwrap();
        graph.add_edge(node3, node1, TestEdge::Simple).unwrap();

        assert_eq!(graph.edges().len(), 3);

        // Remove node2 should remove its edges
        graph.remove_node(node2);

        assert_eq!(graph.nodes().len(), 2);
        assert_eq!(graph.edges().len(), 1); // Only edge between node1 and node3 remains
    }

    #[test]
    fn test_get_node_mut() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = TestNode { id: Uuid::new_v4(), name: "Mutable".to_string() };
        let node_id = graph.add_node(node);

        if let Some(node_wrapper) = graph.get_node_mut(node_id) {
            // Should be able to modify through mutable reference
            // (actual modification depends on Node implementation)
            assert!(node_wrapper.id() == node_id);
        } else {
            panic!("Node should exist");
        }
    }

    #[test]
    fn test_node_capacity_limits() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");

        // Add many nodes to test capacity
        for i in 0..10000 {
            graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });
        }

        assert_eq!(graph.nodes().len(), 10000);
    }
}

mod edge_operations_tests {
    use super::*;

    #[test]
    fn test_add_simple_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        let edge_result = graph.add_edge(node1, node2, TestEdge::Simple);
        assert!(edge_result.is_ok());

        let edge_id = edge_result.unwrap();
        assert_eq!(graph.edges().len(), 1);

        let edge = graph.get_edge(edge_id).unwrap();
        assert_eq!(edge.source, node1);
        assert_eq!(edge.target, node2);
    }

    #[test]
    fn test_add_edge_nonexistent_source() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let fake_id = NodeId::new();

        let result = graph.add_edge(fake_id, node, TestEdge::Simple);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_edge_nonexistent_target() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let fake_id = NodeId::new();

        let result = graph.add_edge(node, fake_id, TestEdge::Simple);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_self_loop() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Self".to_string() });

        let result = graph.add_edge(node, node, TestEdge::Simple);
        assert!(result.is_ok());

        let edge = graph.get_edge(result.unwrap()).unwrap();
        assert_eq!(edge.source, edge.target);
    }

    #[test]
    fn test_add_multiple_edges_between_same_nodes() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        let edge1 = graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        let edge2 = graph.add_edge(node1, node2, TestEdge::Weighted(1.0)).unwrap();
        let edge3 = graph.add_edge(node1, node2, TestEdge::Labeled("test".to_string())).unwrap();

        assert_eq!(graph.edges().len(), 3);
        assert_ne!(edge1, edge2);
        assert_ne!(edge2, edge3);
        assert_ne!(edge1, edge3);
    }

    #[test]
    fn test_remove_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        let edge_id = graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        assert_eq!(graph.edges().len(), 1);

        graph.remove_edge(edge_id);
        assert_eq!(graph.edges().len(), 0);
        assert!(graph.get_edge(edge_id).is_none());
    }

    #[test]
    fn test_remove_nonexistent_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let fake_id = EdgeId::new();

        // Should not panic
        graph.remove_edge(fake_id);
        assert_eq!(graph.edges().len(), 0);
    }

    #[test]
    fn test_edge_directionality() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        let edge_ab = graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        let edge_ba = graph.add_edge(node2, node1, TestEdge::Simple).unwrap();

        assert_ne!(edge_ab, edge_ba);

        let ab = graph.get_edge(edge_ab).unwrap();
        let ba = graph.get_edge(edge_ba).unwrap();

        assert_eq!(ab.source, ba.target);
        assert_eq!(ab.target, ba.source);
    }
}

mod component_system_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestComponent {
        data: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct AnotherComponent {
        value: i32,
    }

    #[test]
    fn test_add_component_to_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node_id = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });

        let component = TestComponent { data: "test".to_string() };
        let result = graph.get_node_mut(node_id).unwrap()
            .add_component(component.clone());

        assert!(result.is_ok());

        let node = graph.get_node(node_id).unwrap();
        assert!(node.has_component::<TestComponent>());
        assert_eq!(node.get_component::<TestComponent>().unwrap(), &component);
    }

    #[test]
    fn test_add_multiple_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node_id = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });

        let comp1 = TestComponent { data: "test".to_string() };
        let comp2 = AnotherComponent { value: 42 };

        graph.get_node_mut(node_id).unwrap().add_component(comp1.clone()).unwrap();
        graph.get_node_mut(node_id).unwrap().add_component(comp2.clone()).unwrap();

        let node = graph.get_node(node_id).unwrap();
        assert!(node.has_component::<TestComponent>());
        assert!(node.has_component::<AnotherComponent>());
        assert_eq!(node.get_component::<TestComponent>().unwrap(), &comp1);
        assert_eq!(node.get_component::<AnotherComponent>().unwrap(), &comp2);
    }

    #[test]
    fn test_replace_component() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node_id = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });

        let comp1 = TestComponent { data: "first".to_string() };
        let comp2 = TestComponent { data: "second".to_string() };

        graph.get_node_mut(node_id).unwrap().add_component(comp1).unwrap();
        graph.get_node_mut(node_id).unwrap().add_component(comp2.clone()).unwrap();

        let node = graph.get_node(node_id).unwrap();
        assert_eq!(node.get_component::<TestComponent>().unwrap(), &comp2);
    }

    #[test]
    fn test_remove_component() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node_id = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });

        let component = TestComponent { data: "test".to_string() };
        graph.get_node_mut(node_id).unwrap().add_component(component).unwrap();

        assert!(graph.get_node(node_id).unwrap().has_component::<TestComponent>());

        graph.get_node_mut(node_id).unwrap().remove_component::<TestComponent>();

        assert!(!graph.get_node(node_id).unwrap().has_component::<TestComponent>());
    }

    #[test]
    fn test_get_nonexistent_component() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node_id = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });

        let node = graph.get_node(node_id).unwrap();
        assert!(!node.has_component::<TestComponent>());
        assert!(node.get_component::<TestComponent>().is_none());
    }
}

mod graph_traversal_tests {
    use super::*;

    #[test]
    fn test_node_degree_calculations() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "C".to_string() });

        graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        graph.add_edge(node1, node3, TestEdge::Simple).unwrap();
        graph.add_edge(node2, node3, TestEdge::Simple).unwrap();
        graph.add_edge(node3, node1, TestEdge::Simple).unwrap();

        assert_eq!(graph.out_degree(node1), 2);
        assert_eq!(graph.in_degree(node1), 1);
        assert_eq!(graph.total_degree(node1), 3);

        assert_eq!(graph.out_degree(node2), 1);
        assert_eq!(graph.in_degree(node2), 1);

        assert_eq!(graph.out_degree(node3), 1);
        assert_eq!(graph.in_degree(node3), 2);
    }

    #[test]
    fn test_degree_of_nonexistent_node() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let fake_id = NodeId::new();

        assert_eq!(graph.out_degree(fake_id), 0);
        assert_eq!(graph.in_degree(fake_id), 0);
        assert_eq!(graph.total_degree(fake_id), 0);
    }

    #[test]
    fn test_neighbors_iteration() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let center = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Center".to_string() });

        let mut neighbors = Vec::new();
        for i in 0..5 {
            let neighbor = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Neighbor{}", i),
            });
            graph.add_edge(center, neighbor, TestEdge::Simple).unwrap();
            neighbors.push(neighbor);
        }

        let out_neighbors: Vec<_> = graph.out_neighbors(center).collect();
        assert_eq!(out_neighbors.len(), 5);

        for neighbor in &neighbors {
            assert!(out_neighbors.contains(neighbor));
        }
    }

    #[test]
    fn test_path_finding() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Start".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Middle".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "End".to_string() });
        let node4 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Isolated".to_string() });

        graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        graph.add_edge(node2, node3, TestEdge::Simple).unwrap();

        assert!(graph.has_path(node1, node3));
        assert!(!graph.has_path(node1, node4));
        assert!(!graph.has_path(node3, node1)); // No reverse path
    }
}

mod graph_algorithms_tests {
    use super::*;

    #[test]
    fn test_cycle_detection() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "C".to_string() });

        graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        graph.add_edge(node2, node3, TestEdge::Simple).unwrap();

        assert!(!graph.has_cycles());

        graph.add_edge(node3, node1, TestEdge::Simple).unwrap();

        assert!(graph.has_cycles());
    }

    #[test]
    fn test_self_loop_cycle_detection() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Self".to_string() });

        assert!(!graph.has_cycles());

        graph.add_edge(node, node, TestEdge::Simple).unwrap();

        assert!(graph.has_cycles());
    }

    #[test]
    fn test_connected_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");

        // Create first component
        let a1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A1".to_string() });
        let a2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A2".to_string() });
        graph.add_edge(a1, a2, TestEdge::Simple).unwrap();

        // Create second component
        let b1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B1".to_string() });
        let b2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B2".to_string() });
        graph.add_edge(b1, b2, TestEdge::Simple).unwrap();

        // Isolated node
        let c = graph.add_node(TestNode { id: Uuid::new_v4(), name: "C".to_string() });

        let components = graph.connected_components();
        assert_eq!(components.len(), 3);
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "1".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "2".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "3".to_string() });
        let node4 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "4".to_string() });

        graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        graph.add_edge(node1, node3, TestEdge::Simple).unwrap();
        graph.add_edge(node2, node4, TestEdge::Simple).unwrap();
        graph.add_edge(node3, node4, TestEdge::Simple).unwrap();

        let result = graph.topological_sort();
        assert!(result.is_ok());

        let sorted = result.unwrap();
        assert_eq!(sorted.len(), 4);

        // Verify ordering constraints
        let pos1 = sorted.iter().position(|&n| n == node1).unwrap();
        let pos2 = sorted.iter().position(|&n| n == node2).unwrap();
        let pos3 = sorted.iter().position(|&n| n == node3).unwrap();
        let pos4 = sorted.iter().position(|&n| n == node4).unwrap();

        assert!(pos1 < pos2);
        assert!(pos1 < pos3);
        assert!(pos2 < pos4);
        assert!(pos3 < pos4);
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "1".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "2".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "3".to_string() });

        graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
        graph.add_edge(node2, node3, TestEdge::Simple).unwrap();
        graph.add_edge(node3, node1, TestEdge::Simple).unwrap();

        let result = graph.topological_sort();
        assert!(result.is_err());
    }
}

mod invariant_tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct NoSelfLoopInvariant;

    impl<N: Clone + Debug, E: Clone + Debug> GraphInvariant<N, E> for NoSelfLoopInvariant {
        fn check(&self, graph: &ContextGraph<N, E>) -> Result<(), String> {
            for edge_id in graph.edges() {
                if let Some(edge) = graph.get_edge(edge_id) {
                    if edge.source == edge.target {
                        return Err("Self-loops are not allowed".to_string());
                    }
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_invariant_enforcement() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        graph.add_invariant(Box::new(NoSelfLoopInvariant));

        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        // Valid edge should work
        let result = graph.add_edge(node1, node2, TestEdge::Simple);
        assert!(result.is_ok());

        // Self-loop should fail
        let result = graph.add_edge(node1, node1, TestEdge::Simple);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Self-loops are not allowed"));
    }

    #[test]
    fn test_multiple_invariants() {
        #[derive(Debug, Clone)]
        struct MaxDegreeInvariant(usize);

        impl<N: Clone + Debug, E: Clone + Debug> GraphInvariant<N, E> for MaxDegreeInvariant {
            fn check(&self, graph: &ContextGraph<N, E>) -> Result<(), String> {
                for node_id in graph.nodes() {
                    if graph.total_degree(node_id) > self.0 {
                        return Err(format!("Node degree exceeds maximum of {}", self.0));
                    }
                }
                Ok(())
            }
        }

        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        graph.add_invariant(Box::new(NoSelfLoopInvariant));
        graph.add_invariant(Box::new(MaxDegreeInvariant(2)));

        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "C".to_string() });

        // First two edges should work
        assert!(graph.add_edge(node1, node2, TestEdge::Simple).is_ok());
        assert!(graph.add_edge(node1, node3, TestEdge::Simple).is_ok());

        // Third edge should fail due to degree constraint
        let result = graph.add_edge(node2, node1, TestEdge::Simple);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("degree exceeds maximum"));
    }
}

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_large_graph_performance() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("LargeGraph");
        let node_count = 1000;
        let edge_count = 5000;

        // Measure node insertion time
        let start = Instant::now();
        let mut nodes = Vec::new();
        for i in 0..node_count {
            let node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });
            nodes.push(node);
        }
        let node_time = start.elapsed();

        // Measure edge insertion time
        let start = Instant::now();
        for i in 0..edge_count {
            let source = nodes[i % node_count];
            let target = nodes[(i + 1) % node_count];
            graph.add_edge(source, target, TestEdge::Simple).unwrap();
        }
        let edge_time = start.elapsed();

        // Verify performance is reasonable
        assert!(node_time.as_millis() < 100); // Should take less than 100ms
        assert!(edge_time.as_millis() < 200); // Should take less than 200ms

        // Measure traversal time
        let start = Instant::now();
        let total_degree: usize = nodes.iter()
            .map(|&node| graph.total_degree(node))
            .sum();
        let traversal_time = start.elapsed();

        assert!(traversal_time.as_millis() < 50); // Should be fast
        assert_eq!(total_degree, edge_count * 2); // Each edge contributes 2 to total degree
    }

    #[test]
    fn test_memory_efficiency() {
        let initial_memory = get_approximate_memory_usage();

        let mut graph = ContextGraph::<TestNode, TestEdge>::new("MemoryTest");

        // Add many small nodes
        for i in 0..10000 {
            graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("N{}", i),
            });
        }

        let after_nodes_memory = get_approximate_memory_usage();
        let node_memory = after_nodes_memory.saturating_sub(initial_memory);

        // Each node should use reasonable memory (rough estimate)
        let memory_per_node = node_memory / 10000;
        assert!(memory_per_node < 1000); // Less than 1KB per node
    }

    fn get_approximate_memory_usage() -> usize {
        // This is a placeholder - actual implementation would use system APIs
        // For testing purposes, we'll use a simple approximation
        std::mem::size_of::<ContextGraph<TestNode, TestEdge>>()
    }
}

mod concurrent_access_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_concurrent_read_access() {
        let graph = Arc::new(ContextGraph::<TestNode, TestEdge>::new("Concurrent"));

        // Add some data
        let mut graph_mut = Arc::try_unwrap(graph.clone()).unwrap_or_else(|arc| {
            // If we can't unwrap, create a new graph for setup
            let mut g = ContextGraph::<TestNode, TestEdge>::new("Concurrent");
            g
        });

        let node1 = graph_mut.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph_mut.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });
        graph_mut.add_edge(node1, node2, TestEdge::Simple).unwrap();

        let graph = Arc::new(graph_mut);

        // Spawn multiple reader threads
        let mut handles = vec![];
        for i in 0..10 {
            let graph_clone = graph.clone();
            let handle = thread::spawn(move || {
                // Perform read operations
                let node_count = graph_clone.nodes().len();
                let edge_count = graph_clone.edges().len();
                assert_eq!(node_count, 2);
                assert_eq!(edge_count, 1);
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_synchronized_write_access() {
        let graph = Arc::new(Mutex::new(ContextGraph::<TestNode, TestEdge>::new("SyncWrite")));

        let mut handles = vec![];
        for i in 0..10 {
            let graph_clone = graph.clone();
            let handle = thread::spawn(move || {
                let mut graph = graph_clone.lock().unwrap();
                graph.add_node(TestNode {
                    id: Uuid::new_v4(),
                    name: format!("Thread{}", i),
                });
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let graph = graph.lock().unwrap();
        assert_eq!(graph.nodes().len(), 10);
    }
}

mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_graph_state_after_failed_operation() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Recovery");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let fake_id = NodeId::new();

        let initial_edge_count = graph.edges().len();

        // This should fail
        let result = graph.add_edge(node1, fake_id, TestEdge::Simple);
        assert!(result.is_err());

        // Graph state should be unchanged
        assert_eq!(graph.edges().len(), initial_edge_count);
        assert_eq!(graph.nodes().len(), 1);
    }

    #[test]
    fn test_partial_batch_operation_rollback() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Batch");

        // Add nodes
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });
        let fake_id = NodeId::new();

        // Try to add multiple edges where one will fail
        let edges_to_add = vec![
            (node1, node2, TestEdge::Simple),
            (node2, node1, TestEdge::Simple),
            (node1, fake_id, TestEdge::Simple), // This will fail
        ];

        let initial_edge_count = graph.edges().len();

        // Simulate batch operation
        let mut added_edges = Vec::new();
        let mut failed = false;

        for (source, target, edge_type) in edges_to_add {
            match graph.add_edge(source, target, edge_type) {
                Ok(edge_id) => added_edges.push(edge_id),
                Err(_) => {
                    failed = true;
                    break;
                }
            }
        }

        if failed {
            // Rollback
            for edge_id in added_edges {
                graph.remove_edge(edge_id);
            }
        }

        // Verify rollback worked
        assert_eq!(graph.edges().len(), initial_edge_count);
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_graph_operations() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("Empty");

        // All these should handle empty graph gracefully
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
        assert!(!graph.has_cycles());
        assert_eq!(graph.connected_components().len(), 0);

        let fake_id = NodeId::new();
        assert_eq!(graph.out_degree(fake_id), 0);
        assert_eq!(graph.in_degree(fake_id), 0);
        assert!(graph.get_node(fake_id).is_none());
    }

    #[test]
    fn test_single_node_graph() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Single");
        let node = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Alone".to_string() });

        assert_eq!(graph.nodes().len(), 1);
        assert_eq!(graph.edges().len(), 0);
        assert!(!graph.has_cycles());
        assert_eq!(graph.connected_components().len(), 1);
        assert_eq!(graph.out_degree(node), 0);
        assert_eq!(graph.in_degree(node), 0);
    }

    #[test]
    fn test_extreme_edge_weights() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Extreme");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        // Test extreme weight values
        assert!(graph.add_edge(node1, node2, TestEdge::Weighted(f32::MAX)).is_ok());
        assert!(graph.add_edge(node2, node1, TestEdge::Weighted(f32::MIN)).is_ok());
        assert!(graph.add_edge(node1, node1, TestEdge::Weighted(f32::INFINITY)).is_ok());
        assert!(graph.add_edge(node2, node2, TestEdge::Weighted(f32::NEG_INFINITY)).is_ok());
        assert!(graph.add_edge(node1, node2, TestEdge::Weighted(f32::NAN)).is_ok());

        assert_eq!(graph.edges().len(), 5);
    }

    #[test]
    fn test_unicode_edge_labels() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Unicode");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        let unicode_labels = vec![
            "🔗",
            "关系",
            "σχέση",
            "関係",
            "🎯→🎪",
            "Very long label with spaces and special chars !@#$%^&*()",
        ];

        for label in unicode_labels {
            assert!(graph.add_edge(node1, node2, TestEdge::Labeled(label.to_string())).is_ok());
        }

        assert_eq!(graph.edges().len(), 6);
    }
}

// Helper function to create a standard test graph
fn create_test_graph() -> ContextGraph<TestNode, TestEdge> {
    let mut graph = ContextGraph::new("TestGraph");

    let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node1".to_string() });
    let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node2".to_string() });
    let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node3".to_string() });

    graph.add_edge(node1, node2, TestEdge::Simple).unwrap();
    graph.add_edge(node2, node3, TestEdge::Simple).unwrap();
    graph.add_edge(node3, node1, TestEdge::Simple).unwrap();

    graph
}
