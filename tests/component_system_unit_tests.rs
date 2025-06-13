//! Component System Unit Tests for ContextGraph
//!
//! These tests rigorously validate component functionality including:
//! - Component addition and removal
//! - Component queries and filters
//! - Type safety and serialization
//! - Component inheritance and composition
//! - Performance with many components
//! - Component versioning and migration

use cim_contextgraph::*;
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq)]
struct TestNode {
    id: Uuid,
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
struct TestEdge {
    weight: f32,
}

// Various component types for testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Caption {
    text: String,
    language: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Label {
    name: String,
    color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Properties {
    attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Metadata {
    created_at: u64,
    updated_at: u64,
    version: u32,
}

mod basic_component_tests {
    use super::*;

    #[test]
    fn test_add_component_to_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("ComponentTest");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "TestNode".to_string(),
        });

        let caption = Caption {
            text: "This is a test node".to_string(),
            language: "en".to_string(),
        };

        graph.add_component(node_id, "caption", caption.clone());

        let retrieved: Option<Caption> = graph.get_component(node_id, "caption");
        assert_eq!(retrieved, Some(caption));
    }

    #[test]
    fn test_add_multiple_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("MultiComponent");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "MultiNode".to_string(),
        });

        let caption = Caption {
            text: "Test caption".to_string(),
            language: "en".to_string(),
        };

        let label = Label {
            name: "Important".to_string(),
            color: "#FF0000".to_string(),
        };

        let position = Position {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        };

        graph.add_component(node_id, "caption", caption.clone());
        graph.add_component(node_id, "label", label.clone());
        graph.add_component(node_id, "position", position.clone());

        assert_eq!(graph.get_component::<Caption>(node_id, "caption"), Some(caption));
        assert_eq!(graph.get_component::<Label>(node_id, "label"), Some(label));
        assert_eq!(graph.get_component::<Position>(node_id, "position"), Some(position));
    }

    #[test]
    fn test_replace_component() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("ReplaceComponent");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node".to_string(),
        });

        let caption1 = Caption {
            text: "Original".to_string(),
            language: "en".to_string(),
        };

        let caption2 = Caption {
            text: "Updated".to_string(),
            language: "fr".to_string(),
        };

        graph.add_component(node_id, "caption", caption1);
        graph.add_component(node_id, "caption", caption2.clone());

        let retrieved: Option<Caption> = graph.get_component(node_id, "caption");
        assert_eq!(retrieved, Some(caption2));
    }

    #[test]
    fn test_remove_component() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("RemoveComponent");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node".to_string(),
        });

        let caption = Caption {
            text: "To be removed".to_string(),
            language: "en".to_string(),
        };

        graph.add_component(node_id, "caption", caption);
        assert!(graph.has_component(node_id, "caption"));

        graph.remove_component(node_id, "caption");
        assert!(!graph.has_component(node_id, "caption"));

        let retrieved: Option<Caption> = graph.get_component(node_id, "caption");
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_component_on_nonexistent_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("NonexistentNode");

        let fake_node_id = NodeId::new();
        let caption = Caption {
            text: "Test".to_string(),
            language: "en".to_string(),
        };

        // Adding component to non-existent node should fail gracefully
        graph.add_component(fake_node_id, "caption", caption);

        let retrieved: Option<Caption> = graph.get_component(fake_node_id, "caption");
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_get_all_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("AllComponents");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node".to_string(),
        });

        graph.add_component(node_id, "caption", Caption {
            text: "Test".to_string(),
            language: "en".to_string(),
        });

        graph.add_component(node_id, "label", Label {
            name: "Test".to_string(),
            color: "#000000".to_string(),
        });

        graph.add_component(node_id, "position", Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        });

        let components = graph.get_components(node_id);
        assert_eq!(components.len(), 3);
        assert!(components.contains_key("caption"));
        assert!(components.contains_key("label"));
        assert!(components.contains_key("position"));
    }
}

mod component_query_tests {
    use super::*;

    #[test]
    fn test_query_nodes_with_component() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("QueryComponent");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node1".to_string(),
        });

        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node2".to_string(),
        });

        let node3 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node3".to_string(),
        });

        // Add labels to node1 and node3 only
        graph.add_component(node1, "label", Label {
            name: "Important".to_string(),
            color: "#FF0000".to_string(),
        });

        graph.add_component(node3, "label", Label {
            name: "Critical".to_string(),
            color: "#FF0000".to_string(),
        });

        let labeled_nodes = graph.nodes_with_component("label");
        assert_eq!(labeled_nodes.len(), 2);
        assert!(labeled_nodes.contains(&node1));
        assert!(labeled_nodes.contains(&node3));
        assert!(!labeled_nodes.contains(&node2));
    }

    #[test]
    fn test_query_nodes_with_multiple_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("MultiQuery");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node1".to_string(),
        });

        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node2".to_string(),
        });

        let node3 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node3".to_string(),
        });

        // Node1: label + position
        graph.add_component(node1, "label", Label {
            name: "A".to_string(),
            color: "#000000".to_string(),
        });
        graph.add_component(node1, "position", Position {
            x: 1.0, y: 1.0, z: 1.0,
        });

        // Node2: label only
        graph.add_component(node2, "label", Label {
            name: "B".to_string(),
            color: "#FFFFFF".to_string(),
        });

        // Node3: position only
        graph.add_component(node3, "position", Position {
            x: 2.0, y: 2.0, z: 2.0,
        });

        let nodes_with_both = graph.nodes_with_components(&["label", "position"]);
        assert_eq!(nodes_with_both.len(), 1);
        assert!(nodes_with_both.contains(&node1));
    }

    #[test]
    fn test_filter_nodes_by_component_value() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("FilterValue");

        let mut red_nodes = Vec::new();
        let mut blue_nodes = Vec::new();

        // Add nodes with different colored labels
        for i in 0..5 {
            let node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Red{}", i),
            });
            graph.add_component(node, "label", Label {
                name: format!("Label{}", i),
                color: "#FF0000".to_string(),
            });
            red_nodes.push(node);
        }

        for i in 0..3 {
            let node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Blue{}", i),
            });
            graph.add_component(node, "label", Label {
                name: format!("Label{}", i),
                color: "#0000FF".to_string(),
            });
            blue_nodes.push(node);
        }

        // Filter by color
        let all_labeled = graph.nodes_with_component("label");
        let red_filtered: Vec<_> = all_labeled.into_iter()
            .filter(|&node_id| {
                if let Some(label) = graph.get_component::<Label>(node_id, "label") {
                    label.color == "#FF0000"
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(red_filtered.len(), 5);
        for node in red_nodes {
            assert!(red_filtered.contains(&node));
        }
    }

    #[test]
    fn test_component_type_safety() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("TypeSafety");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Node".to_string(),
        });

        // Add a Label component
        graph.add_component(node_id, "label", Label {
            name: "Test".to_string(),
            color: "#000000".to_string(),
        });

        // Try to retrieve as wrong type
        let wrong_type: Option<Caption> = graph.get_component(node_id, "label");
        assert!(wrong_type.is_none());

        // Retrieve as correct type
        let correct_type: Option<Label> = graph.get_component(node_id, "label");
        assert!(correct_type.is_some());
    }
}

mod component_edge_tests {
    use super::*;

    #[test]
    fn test_edge_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("EdgeComponents");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "A".to_string(),
        });

        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "B".to_string(),
        });

        let edge_id = graph.add_edge(node1, node2, TestEdge { weight: 1.0 }).unwrap();

        // Add components to edge
        graph.add_edge_component(edge_id, "label", Label {
            name: "Connection".to_string(),
            color: "#00FF00".to_string(),
        });

        graph.add_edge_component(edge_id, "metadata", Metadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        });

        // Retrieve edge components
        let label: Option<Label> = graph.get_edge_component(edge_id, "label");
        assert!(label.is_some());
        assert_eq!(label.unwrap().name, "Connection");

        let metadata: Option<Metadata> = graph.get_edge_component(edge_id, "metadata");
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().version, 1);
    }

    #[test]
    fn test_query_edges_with_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("EdgeQuery");

        let nodes: Vec<_> = (0..4).map(|i| {
            graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            })
        }).collect();

        // Create edges with different components
        let edge1 = graph.add_edge(nodes[0], nodes[1], TestEdge { weight: 1.0 }).unwrap();
        graph.add_edge_component(edge1, "important", true);

        let edge2 = graph.add_edge(nodes[1], nodes[2], TestEdge { weight: 2.0 }).unwrap();
        // No component on edge2

        let edge3 = graph.add_edge(nodes[2], nodes[3], TestEdge { weight: 3.0 }).unwrap();
        graph.add_edge_component(edge3, "important", true);

        let important_edges = graph.edges_with_component("important");
        assert_eq!(important_edges.len(), 2);
        assert!(important_edges.contains(&edge1));
        assert!(important_edges.contains(&edge3));
        assert!(!important_edges.contains(&edge2));
    }
}

mod component_performance_tests {
    use super::*;

    #[test]
    fn test_many_components_per_node() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("ManyComponents");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "ComponentHeavy".to_string(),
        });

        // Add many different components
        for i in 0..100 {
            let component_name = format!("component_{}", i);
            let properties = Properties {
                attributes: vec![
                    (format!("key_{}", i), format!("value_{}", i))
                ].into_iter().collect(),
            };
            graph.add_component(node_id, &component_name, properties);
        }

        // Verify all components exist
        let all_components = graph.get_components(node_id);
        assert_eq!(all_components.len(), 100);

        // Access specific components
        for i in 0..100 {
            let component_name = format!("component_{}", i);
            assert!(graph.has_component(node_id, &component_name));
        }
    }

    #[test]
    fn test_large_component_data() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("LargeData");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "DataNode".to_string(),
        });

        // Create a large properties component
        let mut attributes = HashMap::new();
        for i in 0..1000 {
            attributes.insert(
                format!("key_{}", i),
                "x".repeat(100), // 100 character value
            );
        }

        let large_properties = Properties { attributes: attributes.clone() };
        graph.add_component(node_id, "large_data", large_properties);

        // Retrieve and verify
        let retrieved: Option<Properties> = graph.get_component(node_id, "large_data");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().attributes.len(), 1000);
    }

    #[test]
    fn test_component_query_performance() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("QueryPerf");

        // Create many nodes with various components
        let mut labeled_nodes = Vec::new();
        let mut positioned_nodes = Vec::new();

        for i in 0..1000 {
            let node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });

            if i % 2 == 0 {
                graph.add_component(node, "label", Label {
                    name: format!("Label{}", i),
                    color: "#000000".to_string(),
                });
                labeled_nodes.push(node);
            }

            if i % 3 == 0 {
                graph.add_component(node, "position", Position {
                    x: i as f32,
                    y: i as f32,
                    z: i as f32,
                });
                positioned_nodes.push(node);
            }
        }

        // Query performance
        let start = std::time::Instant::now();
        let found_labeled = graph.nodes_with_component("label");
        let query_time = start.elapsed();

        assert_eq!(found_labeled.len(), 500);
        assert!(query_time < std::time::Duration::from_millis(10));
    }
}

mod component_serialization_tests {
    use super::*;

    #[test]
    fn test_component_serialization() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Serialization");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "SerNode".to_string(),
        });

        let original_caption = Caption {
            text: "Test caption with special chars: 你好 🌍".to_string(),
            language: "multi".to_string(),
        };

        graph.add_component(node_id, "caption", original_caption.clone());

        // Simulate serialization/deserialization
        let components = graph.get_components(node_id);
        if let Some(component_data) = components.get("caption") {
            // In real implementation, this would use actual serialization
            let retrieved: Caption = serde_json::from_value(
                serde_json::to_value(component_data).unwrap()
            ).unwrap();

            assert_eq!(retrieved, original_caption);
        } else {
            panic!("Component not found");
        }
    }

    #[test]
    fn test_component_versioning() {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct VersionedComponent {
            version: u32,
            data: String,
        }

        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Versioning");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "VersionNode".to_string(),
        });

        // Add v1 component
        let v1 = VersionedComponent {
            version: 1,
            data: "Version 1 data".to_string(),
        };
        graph.add_component(node_id, "versioned", v1);

        // Update to v2
        let v2 = VersionedComponent {
            version: 2,
            data: "Version 2 data with more info".to_string(),
        };
        graph.add_component(node_id, "versioned", v2.clone());

        let retrieved: Option<VersionedComponent> = graph.get_component(node_id, "versioned");
        assert_eq!(retrieved, Some(v2));
    }
}

mod component_composition_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct CompositeComponent {
        caption: Caption,
        position: Position,
        metadata: Metadata,
    }

    #[test]
    fn test_composite_components() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Composite");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "CompositeNode".to_string(),
        });

        let composite = CompositeComponent {
            caption: Caption {
                text: "Composite test".to_string(),
                language: "en".to_string(),
            },
            position: Position {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            metadata: Metadata {
                created_at: 1000,
                updated_at: 2000,
                version: 1,
            },
        };

        graph.add_component(node_id, "composite", composite.clone());

        let retrieved: Option<CompositeComponent> = graph.get_component(node_id, "composite");
        assert_eq!(retrieved, Some(composite));
    }

    #[test]
    fn test_component_inheritance_pattern() {
        // Base component
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct BaseEntity {
            id: Uuid,
            created_at: u64,
        }

        // Extended component
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Person {
            base: BaseEntity,
            name: String,
            age: u32,
        }

        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Inheritance");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "PersonNode".to_string(),
        });

        let person = Person {
            base: BaseEntity {
                id: Uuid::new_v4(),
                created_at: 1234567890,
            },
            name: "Alice".to_string(),
            age: 30,
        };

        graph.add_component(node_id, "person", person.clone());

        let retrieved: Option<Person> = graph.get_component(node_id, "person");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Alice");
    }
}

mod component_lifecycle_tests {
    use super::*;

    #[test]
    fn test_component_lifecycle_with_node_removal() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Lifecycle");

        let node_id = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "LifecycleNode".to_string(),
        });

        // Add components
        graph.add_component(node_id, "label", Label {
            name: "Test".to_string(),
            color: "#000000".to_string(),
        });

        graph.add_component(node_id, "position", Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        });

        // Remove node
        graph.remove_node(node_id);

        // Components should be gone
        assert!(!graph.has_component(node_id, "label"));
        assert!(!graph.has_component(node_id, "position"));

        let components = graph.get_components(node_id);
        assert!(components.is_empty());
    }

    #[test]
    fn test_component_transfer_between_nodes() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Transfer");

        let node1 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Source".to_string(),
        });

        let node2 = graph.add_node(TestNode {
            id: Uuid::new_v4(),
            name: "Target".to_string(),
        });

        // Add component to node1
        let label = Label {
            name: "Transferable".to_string(),
            color: "#FF0000".to_string(),
        };
        graph.add_component(node1, "label", label.clone());

        // Transfer to node2
        graph.remove_component(node1, "label");
        graph.add_component(node2, "label", label.clone());

        // Verify transfer
        assert!(!graph.has_component(node1, "label"));
        assert!(graph.has_component(node2, "label"));

        let retrieved: Option<Label> = graph.get_component(node2, "label");
        assert_eq!(retrieved, Some(label));
    }

    #[test]
    fn test_component_cleanup_on_graph_clear() {
        let mut graph = ContextGraph::<TestNode, TestEdge>::new("Cleanup");

        // Add nodes with components
        for i in 0..10 {
            let node = graph.add_node(TestNode {
                id: Uuid::new_v4(),
                name: format!("Node{}", i),
            });

            graph.add_component(node, "index", i);
            graph.add_component(node, "label", Label {
                name: format!("Label{}", i),
                color: "#000000".to_string(),
            });
        }

        // Clear graph
        graph.clear();

        // Verify everything is cleaned up
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);

        // Component queries should return empty
        let labeled = graph.nodes_with_component("label");
        assert!(labeled.is_empty());
    }
}
