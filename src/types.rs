//! Core type definitions for the CIM-ContextGraph system

use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;
use cid::Cid;

/// Unique identifier for a ContextGraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextGraphId(Uuid);

impl ContextGraphId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ContextGraphId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ContextGraphId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a ConceptGraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptGraphId(Uuid);

impl ConceptGraphId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ConceptGraphId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ConceptGraphId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Node identifier within a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Edge identifier within a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(Uuid);

impl EdgeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trait for components that can be attached to nodes or edges
/// Components are immutable once created
pub trait Component: Any + Send + Sync {
    /// Get the component as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Clone the component into a box
    fn clone_box(&self) -> Box<dyn Component>;

    /// Get the name of this component type
    fn type_name(&self) -> &'static str;
}

/// Storage for components attached to a graph element
/// Components are immutable once added
#[derive(Default)]
pub struct ComponentStorage {
    components: HashMap<TypeId, Box<dyn Component>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Add a component (can only be done once per type)
    pub fn add<T: Component + 'static>(&mut self, component: T) -> Result<(), GraphError> {
        let type_id = TypeId::of::<T>();
        if self.components.contains_key(&type_id) {
            return Err(GraphError::ComponentAlreadyExists(component.type_name().to_string()));
        }
        self.components.insert(type_id, Box::new(component));
        Ok(())
    }

    /// Get a component by type (immutable access only)
    pub fn get<T: Component + 'static>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref::<T>())
    }

    /// Remove a component by type (returns the component)
    pub fn remove<T: Component + 'static>(&mut self) -> Option<Box<dyn Component>> {
        self.components.remove(&TypeId::of::<T>())
    }

    /// Check if a component type exists
    pub fn has<T: Component + 'static>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    /// Iterate over all components
    pub fn iter(&self) -> impl Iterator<Item = (&TypeId, &Box<dyn Component>)> {
        self.components.iter()
    }

    /// Get the number of components
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

impl Clone for ComponentStorage {
    fn clone(&self) -> Self {
        let mut storage = ComponentStorage::new();
        for (type_id, component) in &self.components {
            storage.components.insert(*type_id, component.clone_box());
        }
        storage
    }
}

impl fmt::Debug for ComponentStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let component_names: Vec<&str> = self.components
            .values()
            .map(|c| c.type_name())
            .collect();
        f.debug_struct("ComponentStorage")
            .field("components", &component_names)
            .finish()
    }
}

/// A node entry that contains the typed value and its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEntry<N> {
    pub id: NodeId,
    pub value: N,
    #[serde(skip)]
    pub components: ComponentStorage,
}

impl<N> NodeEntry<N> {
    pub fn new(value: N) -> Self {
        Self {
            id: NodeId::new(),
            value,
            components: ComponentStorage::new(),
        }
    }

    pub fn with_id(id: NodeId, value: N) -> Self {
        Self {
            id,
            value,
            components: ComponentStorage::new(),
        }
    }

    /// Add a component to this node
    pub fn add_component<T: Component + 'static>(&mut self, component: T) -> Result<(), GraphError> {
        self.components.add(component)
    }

    /// Builder method to add a component
    pub fn with_component<T: Component + 'static>(mut self, component: T) -> Result<Self, GraphError> {
        self.components.add(component)?;
        Ok(self)
    }

    /// Get a component from this node (immutable)
    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.components.get::<T>()
    }

    /// Check if a component exists
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components.has::<T>()
    }
}

/// An edge entry that contains the typed value and its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeEntry<E> {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub value: E,
    #[serde(skip)]
    pub components: ComponentStorage,
}

impl<E> EdgeEntry<E> {
    pub fn new(source: NodeId, target: NodeId, value: E) -> Self {
        Self {
            id: EdgeId::new(),
            source,
            target,
            value,
            components: ComponentStorage::new(),
        }
    }

    /// Add a component to this edge
    pub fn add_component<T: Component + 'static>(&mut self, component: T) -> Result<(), GraphError> {
        self.components.add(component)
    }

    /// Builder method to add a component
    pub fn with_component<T: Component + 'static>(mut self, component: T) -> Result<Self, GraphError> {
        self.components.add(component)?;
        Ok(self)
    }

    /// Get a component from this edge (immutable)
    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.components.get::<T>()
    }

    /// Check if a component exists
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components.has::<T>()
    }
}

/// Common components that might be attached to nodes/edges

/// Label component for naming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label(pub String);

impl Component for Label {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "Label" }
}

/// Metadata component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: serde_json::Map<String, serde_json::Value>,
}

impl Component for Metadata {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "Metadata" }
}

/// Graph reference component (for nodes that reference other graphs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphReference(pub ContextGraphId);

impl Component for GraphReference {
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "GraphReference" }
}

/// Subgraph component (for nodes that contain entire graphs)
#[derive(Debug)]
pub struct Subgraph<N, E>
where
    N: Clone,
    E: Clone,
{
    pub graph: Box<crate::context_graph::ContextGraph<N, E>>,
}

impl<N, E> Clone for Subgraph<N, E>
where
    N: Clone,
    E: Clone,
{
    fn clone(&self) -> Self {
        Self {
            graph: self.graph.clone()
        }
    }
}

impl<N, E> Component for Subgraph<N, E>
where
    N: 'static + Send + Sync + Clone,
    E: 'static + Send + Sync + Clone,
{
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(Subgraph {
            graph: self.graph.clone()
        })
    }
    fn type_name(&self) -> &'static str { "Subgraph" }
}

/// Error types for graph operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),

    #[error("Graph not found: {0}")]
    GraphNotFound(ContextGraphId),

    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    #[error("Component already exists: {0}")]
    ComponentAlreadyExists(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Composition error: {0}")]
    CompositionError(String),

    #[error("Morphism error: {0}")]
    MorphismError(String),

    #[error("CID {0} not found")]
    CidNotFound(Cid),

    #[error("Duplicate CID {0}")]
    DuplicateCid(Cid),

    #[error("Cycle detected in DAG")]
    CycleDetected,

    #[error("Chain verification failed")]
    ChainVerificationFailed,
}

/// Result type for graph operations
pub type GraphResult<T> = Result<T, GraphError>;
