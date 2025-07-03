//! Benchmark demonstrating ContextGraph performance with PetGraph backend

use cim_contextgraph::{ContextGraph, Label};
use std::time::Instant;
use rand::Rng;

fn main() {
    println!("ContextGraph Performance Benchmark");
    println!("==================================");
    println!("Creating 10,000 person nodes and 5,000 random edges\n");

    // Benchmark the PetGraph-based implementation
    println!("Testing ContextGraph with PetGraph backend:");
    let (graph, time) = benchmark_contextgraph();

    // Show results
    println!("\n\nResults Summary:");
    println!("================");
    println!("ContextGraph Performance:");
    println!("  - Total time: {:.2}s", time.as_secs_f64());
    println!("  - Nodes: {graph.graph.node_count(}"));
    println!("  - Edges: {graph.graph.edge_count(}"));

    // Test some algorithms
    println!("\n\nTesting PetGraph Algorithms:");
    let algo_start = Instant::now();

    println!("  - Is cyclic: {graph.is_cyclic(}"));
    println!("  - Strongly connected components: {graph.strongly_connected_components(}").len());

    // Try topological sort (will fail if cyclic)
    match graph.topological_sort() {
        Ok(sorted) => println!("  - Topological sort succeeded: {sorted.len(} nodes")),
        Err(_) => println!("  - Topological sort failed (graph has cycles)"),
    }

    println!("  - Algorithm tests took: {:.3}s", algo_start.elapsed().as_secs_f64());
}

fn benchmark_contextgraph() -> (ContextGraph<String, String>, std::time::Duration) {
    let start = Instant::now();
    let mut graph = ContextGraph::<String, String>::new("PersonNetwork");

    // Add nodes
    let node_start = Instant::now();
    let mut node_ids = Vec::with_capacity(10_000);

    for i in 0..10_000 {
        let person_name = format!("Person_{i}");
        let node_id = graph.add_node(person_name.clone());

        // Add a label component to some nodes
        if i % 100 == 0 {
            if let Some(node) = graph.get_node_mut(node_id) {
                let _ = node.components.add(Label(format!("VIP_{i}")));
            }
        }

        node_ids.push(node_id);
    }

    println!("  - Added 10,000 nodes in {:.3}s", node_start.elapsed().as_secs_f64());

    // Add random edges
    let edge_start = Instant::now();
    let mut rng = rand::thread_rng();
    let mut edge_count = 0;

    while edge_count < 5_000 {
        let from_idx = rng.gen_range(0..10_000);
        let to_idx = rng.gen_range(0..10_000);

        if from_idx != to_idx {
            let relationship = format!("knows_{edge_count}");
            if graph.add_edge(node_ids[from_idx], node_ids[to_idx], relationship).is_ok() {
                edge_count += 1;
            }
        }
    }

    println!("  - Added 5,000 edges in {:.3}s", edge_start.elapsed().as_secs_f64());

    let total_time = start.elapsed();
    (graph, total_time)
}
