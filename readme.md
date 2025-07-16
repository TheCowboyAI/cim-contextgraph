# CIM-ContextGraph

Fundamental graph abstractions for the Composable Information Machine (CIM).

## Overview

This crate provides a hierarchy of graph abstractions, each optimized for specific use cases while sharing common patterns:

- **ContextGraph**: Universal base graph that can represent ANY graph structure
- **CidDag**: Content-addressed DAG for Event Store and Object Store
- **ConceptGraph** (coming soon): Knowledge representation with semantic relationships
- **WorkflowGraph** (coming soon): Business process and state machine modeling

## Key Features

### 1. Type Flexibility
Nodes and edges can be ANY type, including primitives:
```rust
// String nodes with float edge weights
let mut social = ContextGraph::<String, f64>::new("Social Network");

// Integer nodes and edges
let mut numbers = ContextGraph::<i32, i32>::new("Number Graph");

// Boolean nodes with unit edges
let mut flags = ContextGraph::<bool, ()>::new("Flag Graph");
```

### 2. Component System
Attach metadata without changing core types:
```rust
// Add components to nodes
node.add_component(Label("Important"));
node.add_component(Position { x: 10.0, y: 20.0 });
node.add_component(CustomMetadata { ... });

// Query by component type
let labeled_nodes = graph.query_nodes_with_component::<Label>();
```

### 3. Recursive Composition
Graphs can contain graphs:
```rust
// Attach a subgraph to a node
node.add_component(Subgraph {
    graph: Box::new(inner_graph)
});

// Count all nodes recursively
let total = graph.total_node_count();
```

### 4. Algorithm Access
Leverage PetGraph's algorithms:
```rust
// Shortest path
let path = graph.shortest_path(start, end);

// Cycle detection
if graph.is_cyclic() { ... }

// Topological sort
let build_order = graph.topological_sort()?;

// Strongly connected components
let components = graph.strongly_connected_components();
```

### 5. Content-Addressed Storage
CidDag provides cryptographic integrity:
```rust
// Event sourcing with CID chains
let mut events = EventDag::new();
events.add_event(cid, Some(previous_cid), event, timestamp)?;

// Verify integrity
let chain = events.verify_chain(&latest_cid, Some(&root_cid))?;
```

## Usage Examples

### Basic Graph Creation
```rust
use cim_contextgraph::{ContextGraph, Label};

let mut graph = ContextGraph::<String, f64>::new("My Graph");

// Add nodes
let alice = graph.add_node("Alice".to_string());
let bob = graph.add_node("Bob".to_string());

// Add edge with weight
let edge = graph.add_edge(alice, bob, 0.8)?;

// Add components
graph.get_node_mut(alice).unwrap()
    .add_component(Label("Friend".to_string()))?;
```

### Event Store Pattern
```rust
use cim_contextgraph::{EventDag, EventNode};

let mut event_store = EventDag::new();

let event = EventNode {
    event_id: "evt-001".to_string(),
    aggregate_id: "order-123".to_string(),
    event_type: "OrderCreated".to_string(),
    sequence: 1,
    payload_cid: None,
};

event_store.add_event(event_cid, None, event, timestamp)?;
```

### Custom Components
```rust
#[derive(Debug, Clone)]
struct BusinessMetadata {
    department: String,
    priority: Priority,
    owner: String,
}

impl Component for BusinessMetadata {
    fn as_any(&self) -> &dyn Any { self }
}

// Use in graphs
node.add_component(BusinessMetadata {
    department: "Engineering".to_string(),
    priority: Priority::High,
    owner: "Alice".to_string(),
})?;
```

## Running Examples

```bash
# Run the comprehensive example
cargo run --example graph_composition
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test context_graph_tests
cargo test cid_dag_tests
cargo test context_graph_v2_tests
```

## Architecture

The crate follows these design principles:

1. **Specialization Over Generalization**: Each graph type optimizes for its use case
2. **Composition Over Inheritance**: Graphs compose rather than inherit
3. **Type Safety**: Leverage Rust's type system for correctness
4. **Performance**: Use specialized libraries (PetGraph, Daggy) under the hood
5. **Flexibility**: Component system allows extension without modification

## Graph Type Comparison

| Graph Type | Best For | Node Types | Edge Types | Special Features |
|------------|----------|------------|------------|------------------|
| ContextGraph | General use | Any type | Any type | Components, recursion |
| CidDag | Event/content storage | CID-indexed | Causal/reference | Integrity verification |
| ConceptGraph | Knowledge representation | Concepts | Semantic relations | Similarity metrics |
| WorkflowGraph | Process modeling | States/activities | Transitions | Conditional flow |

## Future Development

- **ConceptGraph**: Semantic knowledge representation
- **WorkflowGraph**: Business process modeling
- **DomainGraph**: DDD aggregate visualization
- **More algorithms**: Community detection, centrality measures
- **Visualization**: Direct rendering support

## License

Licensed under either of:
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
