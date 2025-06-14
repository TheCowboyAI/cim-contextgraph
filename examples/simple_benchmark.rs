//! Simple benchmark demonstrating ContextGraph performance

use cim_contextgraph::{ContextGraph, Label};
use std::time::Instant;

fn main() {
    println!("ContextGraph Performance Benchmark");
    println!("==================================");
    println!("Creating 10,000 person nodes and 5,000 edges\n");

    // Test the PetGraph-based implementation
    println!("Testing ContextGraph (PetGraph backend):");
    let time = benchmark_contextgraph();

    // Show results
    println!("\n\nResults Summary:");
    println!("================");
    println!("Total time: {:.2}s", time.as_secs_f64());
}

fn benchmark_contextgraph() -> std::time::Duration {
    let start = Instant::now();
    let mut graph = ContextGraph::<String, String>::new("PersonNetwork");

    // Add nodes
    let node_start = Instant::now();
    let mut node_ids = Vec::with_capacity(10_000);

    for i in 0..10_000 {
        let person_name = format!("Person_{}", i);
        let node_id = graph.add_node(person_name);

        // Add labels to some nodes
        if i % 100 == 0 {
            if let Some(node) = graph.get_node_mut(node_id) {
                let _ = node.components.add(Label(format!("VIP_{}", i)));
            }
        }

        node_ids.push(node_id);
    }

    println!("  - Added 10,000 nodes in {:.3}s", node_start.elapsed().as_secs_f64());

    // Add edges (simple pattern for reproducibility)
    let edge_start = Instant::now();
    let mut edge_count = 0;

    for i in 0..10_000 {
        for j in 1..=5 {
            let target_idx = (i + j * 1000) % 10_000;
            if i != target_idx && edge_count < 5_000 {
                let relationship = format!("knows_{}", edge_count);
                if graph.add_edge(node_ids[i], node_ids[target_idx], relationship).is_ok() {
                    edge_count += 1;
                }
            }
        }
        if edge_count >= 5_000 {
            break;
        }
    }

    println!("  - Added {} edges in {:.3}s", edge_count, edge_start.elapsed().as_secs_f64());

    // Simple query: find components with labels
    let query_start = Instant::now();
    let labeled_nodes = graph.query_nodes_with_component::<Label>();
    println!("  - Found {} nodes with labels in {:.6}s",
             labeled_nodes.len(), query_start.elapsed().as_secs_f64());

    // Test some PetGraph algorithms
    let algo_start = Instant::now();

    println!("\n  Algorithm tests:");
    println!("  - Is cyclic: {}", graph.is_cyclic());
    println!("  - Strongly connected components: {}", graph.strongly_connected_components().len());

    match graph.topological_sort() {
        Ok(sorted) => println!("  - Topological sort succeeded: {} nodes", sorted.len()),
        Err(_) => println!("  - Topological sort failed (graph has cycles)"),
    }

    println!("  - Algorithm tests took: {:.3}s", algo_start.elapsed().as_secs_f64());

    println!("\n  Final graph stats:");
    println!("  - Total nodes: {}", graph.graph.node_count());
    println!("  - Total edges: {}", graph.graph.edge_count());

    // Test component queries
    let component_start = Instant::now();
    let labeled_nodes = graph.query_nodes_with_component::<Label>();
    println!("  - Nodes with labels: {} (queried in {:.6}s)",
             labeled_nodes.len(),
             component_start.elapsed().as_secs_f64());

    start.elapsed()
}
