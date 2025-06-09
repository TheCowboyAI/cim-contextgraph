//! Tests for ContextGraph v2 - Wrapping PetGraph for Advanced Algorithms
//!
//! These tests demonstrate how ContextGraph v2 provides access to all
//! PetGraph algorithms while maintaining our component system.

use cim_contextgraph::{ContextGraph, NodeId, EdgeId, Label, Metadata, Component};
use std::collections::HashMap;

/// Test that we can use PetGraph's shortest path algorithms
#[test]
fn test_shortest_path_algorithm() {
    let mut graph = ContextGraph::<&str, i32>::new("Transportation Network");

    // Create a city network
    let sf = graph.add_node("San Francisco");
    let la = graph.add_node("Los Angeles");
    let vegas = graph.add_node("Las Vegas");
    let denver = graph.add_node("Denver");
    let chicago = graph.add_node("Chicago");
    let nyc = graph.add_node("New York");

    // Add routes with distances as edge weights
    graph.add_edge(sf, la, 380).unwrap();      // SF -> LA: 380 miles
    graph.add_edge(sf, vegas, 570).unwrap();   // SF -> Vegas: 570 miles
    graph.add_edge(la, vegas, 270).unwrap();   // LA -> Vegas: 270 miles
    graph.add_edge(vegas, denver, 750).unwrap(); // Vegas -> Denver: 750 miles
    graph.add_edge(la, denver, 1020).unwrap();  // LA -> Denver: 1020 miles
    graph.add_edge(denver, chicago, 920).unwrap(); // Denver -> Chicago: 920 miles
    graph.add_edge(chicago, nyc, 790).unwrap(); // Chicago -> NYC: 790 miles
    graph.add_edge(denver, nyc, 1780).unwrap(); // Denver -> NYC: 1780 miles (direct)

    // Find shortest path from SF to NYC
    let path = graph.shortest_path(sf, nyc);
    assert!(path.is_some());

    // The shortest path should be SF -> LA -> Vegas -> Denver -> Chicago -> NYC
    // Total: 380 + 270 + 750 + 920 + 790 = 3110 miles
}

/// Test cycle detection using PetGraph
#[test]
fn test_cycle_detection() {
    let mut graph = ContextGraph::<String, &str>::new("Dependency Graph");

    // Create a module dependency graph
    let mod_a = graph.add_node("module_a".to_string());
    let mod_b = graph.add_node("module_b".to_string());
    let mod_c = graph.add_node("module_c".to_string());
    let mod_d = graph.add_node("module_d".to_string());

    // Add dependencies (no cycle yet)
    graph.add_edge(mod_a, mod_b, "depends_on").unwrap();
    graph.add_edge(mod_b, mod_c, "depends_on").unwrap();
    graph.add_edge(mod_c, mod_d, "depends_on").unwrap();

    // Should not be cyclic
    assert!(!graph.is_cyclic());

    // Add a cycle
    graph.add_edge(mod_d, mod_a, "depends_on").unwrap();

    // Now it should be cyclic
    assert!(graph.is_cyclic());
}

/// Test strongly connected components
#[test]
fn test_strongly_connected_components() {
    let mut graph = ContextGraph::<&str, &str>::new("Social Network");

    // Create two separate friend groups
    // Group 1: Mutual friends
    let alice = graph.add_node("Alice");
    let bob = graph.add_node("Bob");
    let charlie = graph.add_node("Charlie");

    // Group 2: Another friend circle
    let david = graph.add_node("David");
    let eve = graph.add_node("Eve");

    // Isolated person
    let frank = graph.add_node("Frank");

    // Group 1 connections (strongly connected)
    graph.add_edge(alice, bob, "follows").unwrap();
    graph.add_edge(bob, charlie, "follows").unwrap();
    graph.add_edge(charlie, alice, "follows").unwrap();

    // Group 2 connections (strongly connected)
    graph.add_edge(david, eve, "follows").unwrap();
    graph.add_edge(eve, david, "follows").unwrap();

    // One-way connection between groups
    graph.add_edge(charlie, david, "follows").unwrap();

    // Get strongly connected components
    let sccs = graph.strongly_connected_components();

    // Should have 3 components:
    // 1. Alice, Bob, Charlie (all follow each other)
    // 2. David, Eve (mutual followers)
    // 3. Frank (isolated)
    assert_eq!(sccs.len(), 3);
}

/// Test topological sort for task scheduling
#[test]
fn test_topological_sort() {
    let mut graph = ContextGraph::<&str, &str>::new("Build Pipeline");

    // Create a build dependency graph
    let checkout = graph.add_node("checkout");
    let deps = graph.add_node("install_deps");
    let compile = graph.add_node("compile");
    let test = graph.add_node("test");
    let lint = graph.add_node("lint");
    let build = graph.add_node("build");
    let deploy = graph.add_node("deploy");

    // Define build order dependencies
    graph.add_edge(checkout, deps, "then").unwrap();
    graph.add_edge(deps, compile, "then").unwrap();
    graph.add_edge(deps, lint, "then").unwrap();
    graph.add_edge(compile, test, "then").unwrap();
    graph.add_edge(compile, build, "then").unwrap();
    graph.add_edge(test, deploy, "then").unwrap();
    graph.add_edge(lint, deploy, "then").unwrap();
    graph.add_edge(build, deploy, "then").unwrap();

    // Get topological sort
    let sorted = graph.topological_sort().unwrap();

    // Verify valid build order
    assert_eq!(sorted.len(), 7);

    // Checkout must come first
    assert_eq!(sorted[0], checkout);

    // Deploy must come last
    assert_eq!(sorted[sorted.len() - 1], deploy);

    // Dependencies must come before compile
    let deps_pos = sorted.iter().position(|&n| n == deps).unwrap();
    let compile_pos = sorted.iter().position(|&n| n == compile).unwrap();
    assert!(deps_pos < compile_pos);
}

/// Test finding all simple paths
#[test]
fn test_all_simple_paths() {
    let mut graph = ContextGraph::<&str, &str>::new("Route Network");

    // Create a small network with multiple paths
    let start = graph.add_node("Start");
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let end = graph.add_node("End");

    // Create multiple paths from Start to End
    graph.add_edge(start, a, "path").unwrap();
    graph.add_edge(start, b, "path").unwrap();
    graph.add_edge(a, c, "path").unwrap();
    graph.add_edge(b, c, "path").unwrap();
    graph.add_edge(a, end, "path").unwrap();
    graph.add_edge(c, end, "path").unwrap();

    // Find all paths from Start to End with max length 4
    let paths = graph.all_simple_paths(start, end, 4);

    // Should find 3 paths:
    // 1. Start -> A -> End
    // 2. Start -> A -> C -> End
    // 3. Start -> B -> C -> End
    assert_eq!(paths.len(), 3);
}

/// Test component queries work with PetGraph backend
#[test]
fn test_component_queries_with_algorithms() {
    let mut graph = ContextGraph::<String, f64>::new("Product Recommendation");

    // Create product nodes
    let laptop = graph.add_node("Laptop".to_string());
    let mouse = graph.add_node("Mouse".to_string());
    let keyboard = graph.add_node("Keyboard".to_string());
    let monitor = graph.add_node("Monitor".to_string());
    let webcam = graph.add_node("Webcam".to_string());

    // Add category labels
    graph.get_node_mut(laptop).unwrap()
        .add_component(Label("Electronics".to_string()))
        .unwrap();
    graph.get_node_mut(mouse).unwrap()
        .add_component(Label("Accessories".to_string()))
        .unwrap();
    graph.get_node_mut(keyboard).unwrap()
        .add_component(Label("Accessories".to_string()))
        .unwrap();
    graph.get_node_mut(monitor).unwrap()
        .add_component(Label("Electronics".to_string()))
        .unwrap();
    graph.get_node_mut(webcam).unwrap()
        .add_component(Label("Accessories".to_string()))
        .unwrap();

    // Add "frequently bought together" edges with confidence scores
    graph.add_edge(laptop, mouse, 0.85).unwrap();
    graph.add_edge(laptop, keyboard, 0.90).unwrap();
    graph.add_edge(laptop, monitor, 0.75).unwrap();
    graph.add_edge(monitor, webcam, 0.60).unwrap();

    // Query all accessories
    let accessories = graph.query_nodes_with_component::<Label>()
        .into_iter()
        .filter(|node_id| {
            graph.get_node(*node_id)
                .and_then(|node| node.get_component::<Label>())
                .map(|label| label.0 == "Accessories")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    assert_eq!(accessories.len(), 3); // mouse, keyboard, webcam
}

/// Test recursive graph traversal
#[test]
fn test_recursive_graph_traversal() {
    use cim_contextgraph::Subgraph;

    // Create a file system structure
    let mut fs = ContextGraph::<String, String>::new("FileSystem");

    let root = fs.add_node("/".to_string());
    let home = fs.add_node("home".to_string());
    let usr = fs.add_node("usr".to_string());

    fs.add_edge(root, home, "contains".to_string()).unwrap();
    fs.add_edge(root, usr, "contains".to_string()).unwrap();

    // Create home directory subgraph
    let mut home_contents = ContextGraph::<String, String>::new("HomeContents");
    let user1 = home_contents.add_node("alice".to_string());
    let user2 = home_contents.add_node("bob".to_string());
    let docs = home_contents.add_node("documents".to_string());

    home_contents.add_edge(user1, docs, "owns".to_string()).unwrap();

    // Attach subgraph to home node
    fs.get_node_mut(home).unwrap()
        .add_component(Subgraph {
            graph: Box::new(home_contents)
        })
        .unwrap();

    // Count nodes recursively
    let mut total_nodes = 0;
    fs.visit_recursive(|graph, depth| {
        total_nodes += graph.graph.node_count();
        println!("Level {}: {} nodes", depth, graph.graph.node_count());
    });

    assert_eq!(total_nodes, 6); // 3 in main + 3 in subgraph
}

/// Demonstrate graph visualization with mermaid
/// ```mermaid
/// graph TD
///     subgraph "Main Graph"
///         A[Node A] -->|weight: 10| B[Node B]
///         B -->|weight: 20| C[Node C]
///         A -->|weight: 15| C
///     end
///
///     subgraph "Algorithms"
///         SP[Shortest Path: A->C = 15]
///         TS[Topo Sort: A, B, C]
///         SCC[SCCs: 3 components]
///     end
/// ```
#[test]
fn test_algorithm_visualization() {
    let mut graph = ContextGraph::<&str, i32>::new("Algorithm Demo");

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    graph.add_edge(a, b, 10).unwrap();
    graph.add_edge(b, c, 20).unwrap();
    graph.add_edge(a, c, 15).unwrap();

    // Shortest path from A to C is direct (weight 15)
    // Not through B (weight 10 + 20 = 30)

    // Topological sort gives: A, B, C
    let topo = graph.topological_sort().unwrap();
    assert_eq!(topo[0], a);

    // Each node is its own strongly connected component
    let sccs = graph.strongly_connected_components();
    assert_eq!(sccs.len(), 3);
}
