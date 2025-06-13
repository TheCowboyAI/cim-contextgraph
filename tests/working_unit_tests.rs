//! Working Unit Tests for ContextGraph
//!
//! These tests are designed to work with the actual ContextGraph API
//! and rigorously test all available functionality including edge cases.

use cim_contextgraph::*;
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
struct TestNode {
    id: Uuid,
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
struct TestEdge {
    weight: f32,
}

// Define Caption component for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Caption {
    text: String,
}

impl Component for Caption {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "Caption" }
}

mod graph_creation_tests {
    use super::*;

    #[test]
    fn test_empty_graph_creation() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("EmptyGraph");
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
    }

    #[test]
    fn test_graph_with_metadata() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("TestGraph");
        assert!(graph.metadata.properties.contains_key("name"));
        assert_eq!(
            graph.metadata.properties.get("name").unwrap(),
            &serde_json::json!("TestGraph")
        );
    }

    #[test]
    fn test_graph_id_uniqueness() {
        let graph1 = ContextGraph::<TestNode, TestEdge>::new("Graph1");
        let graph2 = ContextGraph::<TestNode, TestEdge>::new("Graph2");
        assert_ne!(graph1.id, graph2.id);
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
        assert!(graph.nodes().contains(&node_id));

        let retrieved = graph.get_node_value(node_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &node);
    }

    #[test]
    fn test_add_multiple_nodes() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let mut node_ids = Vec::new();

        for i in 0..10 {
            let node = TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            };
            let id = graph.add_node(node);
            node_ids.push(id);
        }

        assert_eq!(graph.nodes().len(), 10);
        for id in &node_ids {
            assert!(graph.nodes().contains(id));
        }
    }

    #[test]
    fn test_remove_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = TestNode { id: Uuid::new_v4(), name: "ToRemove".to_string() };
        let node_id = graph.add_node(node.clone());

        assert_eq!(graph.nodes().len(), 1);

        let removed = graph.remove_node(node_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), node);

        assert_eq!(graph.nodes().len(), 0);
        assert!(graph.get_node(node_id).is_none());
    }

    #[test]
    fn test_remove_nonexistent_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let fake_id = NodeId::new();

        let removed = graph.remove_node(fake_id);
        assert!(removed.is_none());
        assert_eq!(graph.nodes().len(), 0);
    }

    #[test]
    fn test_remove_node_with_edges() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node1".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node2".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Node3".to_string() });

        graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node2, node3, TestEdge { weight: 2.0 }).unwrap();
        graph.add_edge(node3, node1, TestEdge { weight: 3.0 }).unwrap();

        assert_eq!(graph.nodes().len(), 3);
        assert_eq!(graph.edges().len(), 3);

        // Remove node2
        let removed = graph.remove_node(node2);
        assert!(removed.is_some());

        // Basic checks
        assert_eq!(graph.nodes().len(), 2);
        assert!(graph.get_node(node2).is_none());

        // The remaining nodes should still be in the nodes() list
        let remaining_nodes = graph.nodes();
        assert!(remaining_nodes.contains(&node1));
        assert!(remaining_nodes.contains(&node3));
    }

    #[test]
    fn test_get_node_mut() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = TestNode { id: Uuid::new_v4(), name: "Mutable".to_string() };
        let node_id = graph.add_node(node);

        // Add a component through mutable reference
        if let Some(node_entry) = graph.get_node_mut(node_id) {
            let result = node_entry.components.add(Label("TestLabel".to_string()));
            assert!(result.is_ok());
        } else {
            panic!("Node should exist");
        }

        // Verify component was added
        let node_entry = graph.get_node(node_id).unwrap();
        assert!(node_entry.components.has::<Label>());
    }

    #[test]
    fn test_node_degree() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let center = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Center".to_string() });

        assert_eq!(graph.degree(center), 0);

        // Add nodes around center
        for i in 0..5 {
            let node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });
            graph.add_edge(center, node, TestEdge { weight: 1.0 }).unwrap();
        }

        // Center has 5 outgoing edges
        assert_eq!(graph.degree(center), 5);

        // Add incoming edges
        for i in 5..8 {
            let node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });
            graph.add_edge(node, center, TestEdge { weight: 1.0 }).unwrap();
        }

        // Center now has 5 outgoing + 3 incoming = 8 total
        assert_eq!(graph.degree(center), 8);
    }
}

mod edge_operations_tests {
    use super::*;

    #[test]
    fn test_add_simple_edge() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        let edge_result = graph.add_edge(node1, node2, TestEdge { weight: 1.5 });
        assert!(edge_result.is_ok());

        let edge_id = edge_result.unwrap();
        assert_eq!(graph.edges().len(), 1);

        let edge = graph.get_edge(edge_id).unwrap();
        assert_eq!(edge.source, node1);
        assert_eq!(edge.target, node2);
        assert_eq!(edge.value.weight, 1.5);
    }

    #[test]
    fn test_add_edge_nonexistent_nodes() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let fake_id = NodeId::new();

        // Nonexistent source
        let result = graph.add_edge(fake_id, node, TestEdge { weight: 1.0 });
        assert!(result.is_err());
        match result {
            Err(GraphError::NodeNotFound(id)) => assert_eq!(id, fake_id),
            _ => panic!("Expected NodeNotFound error"),
        }

        // Nonexistent target
        let result = graph.add_edge(node, fake_id, TestEdge { weight: 1.0 });
        assert!(result.is_err());
        match result {
            Err(GraphError::NodeNotFound(id)) => assert_eq!(id, fake_id),
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[test]
    fn test_self_loop() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Self".to_string() });

        let result = graph.add_edge(node, node, TestEdge { weight: 1.0 });
        assert!(result.is_ok());

        let edge_id = result.unwrap();
        let edge = graph.get_edge(edge_id).unwrap();
        assert_eq!(edge.source, node);
        assert_eq!(edge.target, node);
    }

    #[test]
    fn test_parallel_edges() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        // Add multiple edges between same nodes
        let edge1 = graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        let edge2 = graph.add_edge(node1, node2, TestEdge { weight: 2.0 }).unwrap();
        let edge3 = graph.add_edge(node1, node2, TestEdge { weight: 3.0 }).unwrap();

        assert_eq!(graph.edges().len(), 3);
        assert_ne!(edge1, edge2);
        assert_ne!(edge2, edge3);
        assert_ne!(edge1, edge3);

        // Verify each edge has correct weight
        assert_eq!(graph.get_edge_value(edge1).unwrap().weight, 1.0);
        assert_eq!(graph.get_edge_value(edge2).unwrap().weight, 2.0);
        assert_eq!(graph.get_edge_value(edge3).unwrap().weight, 3.0);
    }

    #[test]
    fn test_bidirectional_edges() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        let edge_forward = graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        let edge_backward = graph.add_edge(node2, node1, TestEdge { weight: 2.0 }).unwrap();

        assert_ne!(edge_forward, edge_backward);

        let forward = graph.get_edge(edge_forward).unwrap();
        let backward = graph.get_edge(edge_backward).unwrap();

        assert_eq!(forward.source, node1);
        assert_eq!(forward.target, node2);
        assert_eq!(backward.source, node2);
        assert_eq!(backward.target, node1);
    }
}

mod component_tests {
    use super::*;

    #[test]
    fn test_node_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node_id = graph.add_node(TestNode { id: Uuid::new_v4(), name: "ComponentNode".to_string() });

        // Add components
        let node = graph.get_node_mut(node_id).unwrap();
        assert!(node.components.add(Label("Important".to_string())).is_ok());
        assert!(node.components.add(Caption { text: "Test caption".to_string() }).is_ok());

        // Verify components
        let node = graph.get_node(node_id).unwrap();
        assert!(node.components.has::<Label>());
        assert!(node.components.has::<Caption>());

        // Get components
        let label = node.get_component::<Label>();
        assert!(label.is_some());
        assert_eq!(label.unwrap().0, "Important");

        let caption = node.get_component::<Caption>();
        assert!(caption.is_some());
        assert_eq!(caption.unwrap().text, "Test caption");
    }

    #[test]
    fn test_query_nodes_with_component() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");

        // Add nodes with and without labels
        let labeled_nodes: Vec<NodeId> = (0..5).map(|i| {
            let id = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Labeled{}", i),
            });
            graph.get_node_mut(id).unwrap()
                .components.add(Label(format!("Label{}", i))).unwrap();
            id
        }).collect();

        let unlabeled_nodes: Vec<NodeId> = (0..3).map(|i| {
            graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Unlabeled{}", i),
            })
        }).collect();

        // Query labeled nodes
        let found = graph.query_nodes_with_component::<Label>();
        assert_eq!(found.len(), 5);

        for id in &labeled_nodes {
            assert!(found.contains(id));
        }

        for id in &unlabeled_nodes {
            assert!(!found.contains(id));
        }
    }

    #[test]
    fn test_subgraph_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Parent");
        let node_id = graph.add_node(TestNode { id: Uuid::new_v4(), name: "SubgraphNode".to_string() });

        // Create a subgraph
        let subgraph = ContextGraph::<TestNode, TestEdge>::new("Child");
        let subgraph_component = Subgraph { graph: Box::new(subgraph) };

        // Add subgraph as component
        graph.get_node_mut(node_id).unwrap()
            .components.add(subgraph_component).unwrap();

        // Query subgraph nodes
        let subgraph_nodes = graph.get_subgraph_nodes();
        assert_eq!(subgraph_nodes.len(), 1);
        assert!(subgraph_nodes.contains(&node_id));
    }
}

mod algorithm_tests {
    use super::*;

    #[test]
    fn test_cycle_detection() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "C".to_string() });

        graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node2, node3, TestEdge { weight: 1.0 }).unwrap();

        assert!(!graph.is_cyclic());

        // Add edge to create cycle
        graph.add_edge(node3, node1, TestEdge { weight: 1.0 }).unwrap();

        assert!(graph.is_cyclic());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "1".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "2".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "3".to_string() });
        let node4 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "4".to_string() });

        // Create DAG
        graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node1, node3, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node2, node4, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node3, node4, TestEdge { weight: 1.0 }).unwrap();

        let sorted = graph.topological_sort();
        assert!(sorted.is_ok());

        let order = sorted.unwrap();
        assert_eq!(order.len(), 4);

        // node1 should come before node2 and node3
        let pos1 = order.iter().position(|&x| x == node1).unwrap();
        let pos2 = order.iter().position(|&x| x == node2).unwrap();
        let pos3 = order.iter().position(|&x| x == node3).unwrap();
        let pos4 = order.iter().position(|&x| x == node4).unwrap();

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

        graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node2, node1, TestEdge { weight: 1.0 }).unwrap(); // Cycle

        let sorted = graph.topological_sort();
        assert!(sorted.is_err());
        match sorted {
            Err(GraphError::CycleDetected) => {},
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_strongly_connected_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");

        // Create first SCC
        let a1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A1".to_string() });
        let a2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A2".to_string() });
        graph.add_edge(a1, a2, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(a2, a1, TestEdge { weight: 1.0 }).unwrap();

        // Create second SCC
        let b1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B1".to_string() });
        let b2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B2".to_string() });
        graph.add_edge(b1, b2, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(b2, b1, TestEdge { weight: 1.0 }).unwrap();

        // Connect SCCs (one-way)
        graph.add_edge(a1, b1, TestEdge { weight: 1.0 }).unwrap();

        // Isolated node
        let c = graph.add_node(TestNode { id: Uuid::new_v4(), name: "C".to_string() });

        let sccs = graph.strongly_connected_components();
        assert_eq!(sccs.len(), 3);

        // Each SCC should have the right nodes
        for scc in &sccs {
            if scc.contains(&a1) {
                assert!(scc.contains(&a2));
                assert_eq!(scc.len(), 2);
            } else if scc.contains(&b1) {
                assert!(scc.contains(&b2));
                assert_eq!(scc.len(), 2);
            } else if scc.contains(&c) {
                assert_eq!(scc.len(), 1);
            } else {
                panic!("Unexpected SCC");
            }
        }
    }

    #[test]
    fn test_all_simple_paths() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "1".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "2".to_string() });
        let node3 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "3".to_string() });
        let node4 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "4".to_string() });

        // Create multiple paths from 1 to 4
        graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node2, node4, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node1, node3, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node3, node4, TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge(node2, node3, TestEdge { weight: 1.0 }).unwrap();

        let paths = graph.all_simple_paths(node1, node4, 10);
        assert!(paths.len() >= 2); // At least two paths: 1->2->4 and 1->3->4

        // Verify all paths start with node1 and end with node4
        for path in &paths {
            assert!(!path.is_empty());
            assert_eq!(path[0], node1);
            assert_eq!(path[path.len() - 1], node4);
        }
    }
}

mod invariant_tests {
    use super::*;

    struct NoSelfLoopInvariant;

    impl<N: Clone + Debug, E: Clone + Debug> GraphInvariant<N, E> for NoSelfLoopInvariant {
        fn check(&self, graph: &ContextGraph<N, E>) -> GraphResult<()> {
            for edge_id in graph.edges() {
                if let Some(edge) = graph.get_edge(edge_id) {
                    if edge.source == edge.target {
                        return Err(GraphError::InvariantViolation(
                            "Self-loops are not allowed".to_string()
                        ));
                    }
                }
            }
            Ok(())
        }

        fn name(&self) -> &str {
            "NoSelfLoopInvariant"
        }
    }

    #[test]
    fn test_invariant_checking() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Test");
        graph.invariants.push(Box::new(NoSelfLoopInvariant));

        let node1 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "A".to_string() });
        let node2 = graph.add_node(TestNode { id: Uuid::new_v4(), name: "B".to_string() });

        // Normal edge should work
        let result = graph.add_edge(node1, node2, TestEdge { weight: 1.0 });
        assert!(result.is_ok());

        // Self-loop should fail
        let result = graph.add_edge(node1, node1, TestEdge { weight: 1.0 });
        assert!(result.is_err());
        match result {
            Err(GraphError::InvariantViolation(msg)) => {
                assert!(msg.contains("Self-loops"));
            }
            _ => panic!("Expected InvariantViolation error"),
        }
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_graph_operations() {
        let graph = ContextGraph::<TestNode, TestEdge>::new("Empty");

        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
        assert!(!graph.is_cyclic());

        let sccs = graph.strongly_connected_components();
        assert_eq!(sccs.len(), 0);

        let topo = graph.topological_sort();
        assert!(topo.is_ok());
        assert_eq!(topo.unwrap().len(), 0);
    }

    #[test]
    fn test_single_node_operations() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Single");
        let node = graph.add_node(TestNode { id: Uuid::new_v4(), name: "Lonely".to_string() });

        assert_eq!(graph.degree(node), 0);
        assert!(!graph.is_cyclic());

        let sccs = graph.strongly_connected_components();
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 1);
        assert_eq!(sccs[0][0], node);
    }

    #[test]
    fn test_large_graph_performance() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Large");
        let mut nodes = Vec::new();

        // Add many nodes
        for i in 0..1000 {
            let node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });
            nodes.push(node);
        }

        // Add edges to create a connected graph
        for i in 0..999 {
            graph.add_edge(nodes[i], nodes[i + 1], TestEdge { weight: i as f32 }).unwrap();
        }

        assert_eq!(graph.nodes().len(), 1000);
        assert_eq!(graph.edges().len(), 999);

        // Test algorithms on large graph
        assert!(!graph.is_cyclic());

        let topo = graph.topological_sort();
        assert!(topo.is_ok());
        assert_eq!(topo.unwrap().len(), 1000);
    }
}
