//! Comprehensive ContextGraph Tests
//!
//! All user stories in this test suite demonstrate the composition of core domain entities:
//! - **People**: Individuals with roles, responsibilities, and relationships
//! - **Organizations**: Companies, departments, or groups that people belong to
//! - **Agents**: AI or automated systems that perform tasks and make decisions
//! - **Locations**: Physical or virtual places where activities occur
//! - **Documents**: Information artifacts that flow through workflows
//! - **Policies**: Rules and constraints that govern behavior and access
//! - **Workflow Management**: Processes that coordinate activities between entities
//!
//! Each test demonstrates how these entities interact within different contextual graphs,
//! showing real-world scenarios that combine multiple entity types to create meaningful
//! business workflows and knowledge representations.

use cim_contextgraph::{
    ContextGraph, NodeId, EdgeId, GraphResult, GraphError,
    types::{Label, Subgraph},
    GraphInvariant,
};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use uuid;

// Domain-specific node types (Nouns/Entities)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum EntityType {
    Person { name: String, age: u32 },
    Organization { name: String, industry: String },
    Document { title: String, content: String },
    System { name: String, version: String },
}

// Domain-specific edge types (Verbs with direction)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum RelationshipType {
    Owns,           // Person/Org -> Asset
    WorksFor,       // Person -> Organization
    Authors,        // Person -> Document
    References,     // Document -> Document
    DependsOn,      // System -> System
    Contains,       // Container -> Item
    Manages,        // Person -> Person/System
}

// Custom components for rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Caption {
    text: String,
    language: String,
}

impl cim_domain::Component for Caption {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn clone_box(&self) -> Box<dyn cim_domain::Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "Caption" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Properties {
    attributes: HashMap<String, serde_json::Value>,
}

impl cim_domain::Component for Properties {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn clone_box(&self) -> Box<dyn cim_domain::Component> { Box::new(self.clone()) }
    fn type_name(&self) -> &'static str { "Properties" }
}

// Event types for event-driven operations
#[derive(Debug, Clone, Serialize, Deserialize)]
enum GraphEvent {
    NodeAdded { id: NodeId, entity: EntityType },
    NodeRemoved { id: NodeId },
    EdgeAdded { id: EdgeId, source: NodeId, target: NodeId, relationship: RelationshipType },
    EdgeRemoved { id: EdgeId },
    ComponentAdded { node_id: NodeId, component_type: String },
    SubgraphEmbedded { parent_node: NodeId, subgraph_id: String },
}

// Event handler trait
trait EventHandler {
    fn handle_event(&mut self, event: GraphEvent) -> GraphResult<()>;
}

// Graph invariants for theory compliance
struct DirectedAcyclicInvariant;

impl<N, E> GraphInvariant<N, E> for DirectedAcyclicInvariant
where
    N: Clone + Debug,
    E: Clone + Debug,
{
    fn check(&self, graph: &ContextGraph<N, E>) -> GraphResult<()> {
        if graph.is_cyclic() {
            Err(GraphError::InvariantViolation("Graph must be acyclic".to_string()))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &str {
        "DirectedAcyclicInvariant"
    }
}

// User Story 1: As a developer, I want to create typed graphs with domain entities
#[test]
fn test_user_story_1_typed_graph_creation() {
    // Given: Domain entity types representing people, organizations, documents, and systems
    #[derive(Debug, Clone, PartialEq)]
    enum DomainEntity {
        Person { name: String, role: String },
        Organization { name: String, type_: String },
        Document { title: String, classification: String },
        Agent { name: String, capabilities: Vec<String> },
        Location { name: String, address: String },
        Policy { name: String, scope: String },
    }

    #[derive(Debug, Clone, PartialEq)]
    enum DomainRelationship {
        WorksFor,       // Person -> Organization
        Authors,        // Person -> Document
        LocatedAt,      // Person/Org -> Location
        EnforcedBy,     // Document -> Policy
        ManagedBy,      // Any -> Agent
        Governs,        // Policy -> Any
    }

    // When: I create a graph with domain entities
    let mut graph = ContextGraph::<DomainEntity, DomainRelationship>::new("DomainGraph");

    // Add people
    let alice = graph.add_node(DomainEntity::Person {
        name: "Alice Johnson".to_string(),
        role: "Software Engineer".to_string(),
    });

    let bob = graph.add_node(DomainEntity::Person {
        name: "Bob Smith".to_string(),
        role: "Product Manager".to_string(),
    });

    // Add organization
    let tech_corp = graph.add_node(DomainEntity::Organization {
        name: "TechCorp".to_string(),
        type_: "Technology Company".to_string(),
    });

    // Add location
    let hq = graph.add_node(DomainEntity::Location {
        name: "TechCorp HQ".to_string(),
        address: "123 Tech Street".to_string(),
    });

    // Add document
    let spec_doc = graph.add_node(DomainEntity::Document {
        title: "Product Specification".to_string(),
        classification: "Internal".to_string(),
    });

    // Add policy
    let doc_policy = graph.add_node(DomainEntity::Policy {
        name: "Document Access Policy".to_string(),
        scope: "Company-wide".to_string(),
    });

    // Add agent
    let workflow_agent = graph.add_node(DomainEntity::Agent {
        name: "Workflow Automation Agent".to_string(),
        capabilities: vec!["document_routing".to_string(), "approval_tracking".to_string()],
    });

    // Connect entities with relationships
    graph.add_edge(alice, tech_corp, DomainRelationship::WorksFor).unwrap();
    graph.add_edge(bob, tech_corp, DomainRelationship::WorksFor).unwrap();
    graph.add_edge(tech_corp, hq, DomainRelationship::LocatedAt).unwrap();
    graph.add_edge(alice, spec_doc, DomainRelationship::Authors).unwrap();
    graph.add_edge(spec_doc, doc_policy, DomainRelationship::EnforcedBy).unwrap();
    graph.add_edge(spec_doc, workflow_agent, DomainRelationship::ManagedBy).unwrap();

    // Then: The graph should contain all domain entities and relationships
    assert_eq!(graph.nodes().len(), 7); // 2 people + 1 org + 1 location + 1 doc + 1 policy + 1 agent
    assert_eq!(graph.edges().len(), 6); // All relationships

    // And: I can query domain-specific relationships
    let alice_works_for = graph.edges().into_iter()
        .find(|&edge_id| {
            graph.get_edge(edge_id)
                .map(|e| e.source == alice && matches!(e.value, DomainRelationship::WorksFor))
                .unwrap_or(false)
        });
    assert!(alice_works_for.is_some());

    // And: I can trace document governance
    let doc_policy_edge = graph.edges().into_iter()
        .find(|&edge_id| {
            graph.get_edge(edge_id)
                .map(|e| e.source == spec_doc && matches!(e.value, DomainRelationship::EnforcedBy))
                .unwrap_or(false)
        });
    assert!(doc_policy_edge.is_some());
}

// User Story 2: As a developer, I want event-driven workflow management for domain entities
#[test]
fn test_user_story_2_event_driven_operations() {
    // Given: A workflow management system with event handling
    struct WorkflowManagementSystem {
        graph: ContextGraph<EntityType, RelationshipType>,
        event_log: Vec<WorkflowEvent>,
    }

    #[derive(Debug, Clone)]
    enum WorkflowEvent {
        PersonJoinedOrg { person_id: NodeId, org_id: NodeId },
        DocumentCreated { doc_id: NodeId, author_id: NodeId },
        PolicyApplied { policy_id: NodeId, target_id: NodeId },
        AgentAssigned { agent_id: NodeId, task_id: NodeId },
        LocationChanged { entity_id: NodeId, location_id: NodeId },
        WorkflowStarted { workflow_name: String, initiator_id: NodeId },
    }

    impl EventHandler for WorkflowManagementSystem {
        fn handle_event(&mut self, event: GraphEvent) -> GraphResult<()> {
            // Log workflow events
            match &event {
                GraphEvent::NodeAdded { id, entity } => {
                    println!("Entity added to workflow: {:?}", entity);
                }
                GraphEvent::EdgeAdded { source, target, relationship, .. } => {
                    // Track workflow-specific events
                    match relationship {
                        RelationshipType::WorksFor => {
                            self.event_log.push(WorkflowEvent::PersonJoinedOrg {
                                person_id: *source,
                                org_id: *target,
                            });
                        }
                        RelationshipType::Authors => {
                            self.event_log.push(WorkflowEvent::DocumentCreated {
                                doc_id: *target,
                                author_id: *source,
                            });
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            Ok(())
        }
    }

    let mut workflow_system = WorkflowManagementSystem {
        graph: ContextGraph::new("HRWorkflow"),
        event_log: Vec::new(),
    };

    // When: Domain entities interact through workflow events

    // Add person
    let sarah = workflow_system.graph.add_node(EntityType::Person {
        name: "Sarah Chen".to_string(),
        age: 28,
    });

    // Add organization
    let startup = workflow_system.graph.add_node(EntityType::Organization {
        name: "AI Startup Inc".to_string(),
        industry: "Artificial Intelligence".to_string(),
    });

    // Add location (using System as proxy for now)
    let office = workflow_system.graph.add_node(EntityType::System {
        name: "Remote Office Hub".to_string(),
        version: "v2.0".to_string(),
    });

    // Add policy (using Document as proxy)
    let onboarding_policy = workflow_system.graph.add_node(EntityType::Document {
        title: "Employee Onboarding Policy".to_string(),
        content: "Standard procedures for new hires".to_string(),
    });

    // Trigger workflow events
    let edge1 = workflow_system.graph.add_edge(sarah, startup, RelationshipType::WorksFor).unwrap();
    workflow_system.handle_event(GraphEvent::EdgeAdded {
        id: edge1,
        source: sarah,
        target: startup,
        relationship: RelationshipType::WorksFor,
    }).unwrap();

    let edge2 = workflow_system.graph.add_edge(sarah, onboarding_policy, RelationshipType::Authors).unwrap();
    workflow_system.handle_event(GraphEvent::EdgeAdded {
        id: edge2,
        source: sarah,
        target: onboarding_policy,
        relationship: RelationshipType::Authors,
    }).unwrap();

    // Add workflow agent
    let hr_agent = workflow_system.graph.add_node(EntityType::System {
        name: "HR Automation Agent".to_string(),
        version: "v1.0".to_string(),
    });

    let edge3 = workflow_system.graph.add_edge(hr_agent, sarah, RelationshipType::Manages).unwrap();
    workflow_system.handle_event(GraphEvent::EdgeAdded {
        id: edge3,
        source: hr_agent,
        target: sarah,
        relationship: RelationshipType::Manages,
    }).unwrap();

    // Then: The workflow system should track all domain entity interactions
    assert_eq!(workflow_system.event_log.len(), 2); // PersonJoinedOrg and DocumentCreated

    // And: The graph should reflect the current workflow state
    assert_eq!(workflow_system.graph.nodes().len(), 5);
    assert_eq!(workflow_system.graph.edges().len(), 3);

    // And: I can query workflow relationships
    let sarah_relationships = workflow_system.graph.edges().into_iter()
        .filter(|&edge_id| {
            workflow_system.graph.get_edge(edge_id)
                .map(|e| e.source == sarah || e.target == sarah)
                .unwrap_or(false)
        })
        .count();

    assert_eq!(sarah_relationships, 3); // Works for startup, authors policy, managed by agent
}

// User Story 3: As a developer, I want to attach policy and document metadata to entities
#[test]
fn test_user_story_3_component_system() {
    // Given: A graph with people, documents, and policies
    let mut graph = ContextGraph::<EntityType, RelationshipType>::new("PolicyManagementGraph");

    // Add person (compliance officer)
    let compliance_officer = graph.add_node(EntityType::Person {
        name: "Maria Rodriguez".to_string(),
        age: 35,
    });

    // Add organization
    let finance_dept = graph.add_node(EntityType::Organization {
        name: "Finance Department".to_string(),
        industry: "Financial Services".to_string(),
    });

    // Add sensitive document
    let financial_report = graph.add_node(EntityType::Document {
        title: "Q4 Financial Report".to_string(),
        content: "Confidential financial data...".to_string(),
    });

    // Add policy document
    let data_policy = graph.add_node(EntityType::Document {
        title: "Data Protection Policy".to_string(),
        content: "All financial documents must be encrypted...".to_string(),
    });

    // When: I add policy-related metadata using components

    // Add security classification to document
    graph.get_node_mut(financial_report).unwrap()
        .add_component(Caption {
            text: "CONFIDENTIAL - Finance Only".to_string(),
            language: "en".to_string(),
        }).unwrap();

    // Add policy metadata
    graph.get_node_mut(data_policy).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("policy_type".to_string(), serde_json::json!("data_protection"));
                attrs.insert("enforcement_level".to_string(), serde_json::json!("mandatory"));
                attrs.insert("applies_to".to_string(), serde_json::json!(["financial_documents", "customer_data"]));
                attrs.insert("last_reviewed".to_string(), serde_json::json!("2024-01-15"));
                attrs.insert("review_frequency_days".to_string(), serde_json::json!(90));
                attrs
            }
        }).unwrap();

    // Add compliance tracking to person
    graph.get_node_mut(compliance_officer).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("role".to_string(), serde_json::json!("Chief Compliance Officer"));
                attrs.insert("certifications".to_string(), serde_json::json!(["CISA", "CRISC"]));
                attrs.insert("clearance_level".to_string(), serde_json::json!("top_secret"));
                attrs.insert("policy_authority".to_string(), serde_json::json!(true));
                attrs
            }
        }).unwrap();

    // Add audit trail to financial report
    graph.get_node_mut(financial_report).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("created_by".to_string(), serde_json::json!("finance_system"));
                attrs.insert("created_at".to_string(), serde_json::json!("2024-01-20T10:00:00Z"));
                attrs.insert("access_log".to_string(), serde_json::json!([
                    {"user": "maria.rodriguez", "action": "view", "timestamp": "2024-01-20T11:00:00Z"},
                    {"user": "john.smith", "action": "download", "timestamp": "2024-01-20T14:30:00Z"}
                ]));
                attrs.insert("encryption_status".to_string(), serde_json::json!("AES-256"));
                attrs
            }
        }).unwrap();

    // Connect entities with policy relationships
    graph.add_edge(compliance_officer, data_policy, RelationshipType::Authors).unwrap();
    graph.add_edge(financial_report, data_policy, RelationshipType::References).unwrap();
    graph.add_edge(compliance_officer, finance_dept, RelationshipType::WorksFor).unwrap();

    // Then: I can query policy metadata
    let policy_node = graph.get_node(data_policy).unwrap();
    assert!(policy_node.has_component::<Properties>());

    if let Some(props) = policy_node.get_component::<Properties>() {
        assert_eq!(
            props.attributes.get("enforcement_level").unwrap(),
            &serde_json::json!("mandatory")
        );
    }

    // And: I can verify document security classification
    let doc_node = graph.get_node(financial_report).unwrap();
    assert!(doc_node.has_component::<Caption>());

    if let Some(caption) = doc_node.get_component::<Caption>() {
        assert!(caption.text.contains("CONFIDENTIAL"));
    }

    // And: I can trace policy compliance
    let compliance_node = graph.get_node(compliance_officer).unwrap();
    if let Some(props) = compliance_node.get_component::<Properties>() {
        assert_eq!(
            props.attributes.get("policy_authority").unwrap(),
            &serde_json::json!(true)
        );
    }
}

// User Story 4: As a developer, I want edges to have direction and components
#[test]
fn test_user_story_4_directional_edges_with_metadata() {
    // Given: A graph modeling document references
    let mut graph = ContextGraph::<EntityType, RelationshipType>::new("DocumentGraph");

    let doc1 = graph.add_node(EntityType::Document {
        title: "Paper A".to_string(),
        content: "Research on X...".to_string(),
    });

    let doc2 = graph.add_node(EntityType::Document {
        title: "Paper B".to_string(),
        content: "Building on Paper A...".to_string(),
    });

    // When: I create a directional edge with metadata
    let reference = graph.add_edge(doc2, doc1, RelationshipType::References).unwrap();

    // Add metadata to the edge
    let edge = graph.get_edge_mut(reference).unwrap();
    edge.add_component(Label("Citation".to_string())).unwrap();

    let mut props = HashMap::new();
    props.insert("citation_type".to_string(), serde_json::json!("direct"));
    props.insert("page_numbers".to_string(), serde_json::json!([12, 15, 23]));
    edge.add_component(Properties { attributes: props }).unwrap();

    // Then: The edge should maintain direction
    let edge = graph.get_edge(reference).unwrap();
    assert_eq!(edge.source, doc2); // Paper B references Paper A
    assert_eq!(edge.target, doc1);

    // And: Edge metadata should be accessible
    assert!(edge.has_component::<Label>());
    if let Some(props) = edge.get_component::<Properties>() {
        assert_eq!(props.attributes.get("citation_type").unwrap(), &serde_json::json!("direct"));
    }
}

// User Story 5: As a developer, I want to embed and compose graphs
#[test]
fn test_user_story_5_graph_composition() {
    // Given: A main graph and a subgraph
    let mut main_graph = ContextGraph::<EntityType, RelationshipType>::new("MainSystem");
    let mut sub_graph = ContextGraph::<EntityType, RelationshipType>::new("Subsystem");

    // Build the subgraph
    let component1 = sub_graph.add_node(EntityType::System {
        name: "Database".to_string(),
        version: "5.7".to_string(),
    });

    let component2 = sub_graph.add_node(EntityType::System {
        name: "Cache".to_string(),
        version: "6.0".to_string(),
    });

    sub_graph.add_edge(component1, component2, RelationshipType::DependsOn).unwrap();

    // When: I embed the subgraph into a node of the main graph
    let subsystem_node = main_graph.add_node(EntityType::System {
        name: "DataLayer".to_string(),
        version: "1.0".to_string(),
    });

    // Note: In a real implementation, we'd need to implement the embedding
    // For now, we'll add a component that references the subgraph
    let subgraph_component = Subgraph { graph: Box::new(sub_graph) };

    // Then: The structure should be preserved
    // (This would require implementing the actual embedding functionality)

    // And: I should be able to traverse into subgraphs
    assert_eq!(main_graph.nodes().len(), 1);

    // Verify we can count nodes recursively (if implemented)
    // assert_eq!(main_graph.total_node_count(), 3); // 1 in main + 2 in sub
}

// User Story 6: As a developer, I want the graph to follow Graph Theory principles
#[test]
fn test_user_story_6_graph_theory_compliance() {
    // Given: A graph that should follow graph theory
    let mut graph = ContextGraph::<EntityType, RelationshipType>::new("GraphTheory");

    // Create a small network
    let a = graph.add_node(EntityType::System { name: "A".to_string(), version: "1.0".to_string() });
    let b = graph.add_node(EntityType::System { name: "B".to_string(), version: "1.0".to_string() });
    let c = graph.add_node(EntityType::System { name: "C".to_string(), version: "1.0".to_string() });
    let d = graph.add_node(EntityType::System { name: "D".to_string(), version: "1.0".to_string() });

    graph.add_edge(a, b, RelationshipType::DependsOn).unwrap();
    graph.add_edge(b, c, RelationshipType::DependsOn).unwrap();
    graph.add_edge(c, d, RelationshipType::DependsOn).unwrap();

    // Then: Basic graph properties should be calculable

    // Degree (in + out)
    assert_eq!(graph.degree(b), 2); // 1 in, 1 out

    // Path finding
    let paths = graph.find_paths(a, d);
    assert!(!paths.is_empty());

    // Cycle detection
    assert!(!graph.is_cyclic());

    // Add a cycle
    graph.add_edge(d, a, RelationshipType::DependsOn).unwrap();
    assert!(graph.is_cyclic());

    // Topological sort (should fail with cycle)
    assert!(graph.topological_sort().is_err());
}

// User Story 7: As a developer, I want to enforce graph invariants
#[test]
fn test_user_story_7_invariant_enforcement() {
    // Given: A graph with invariants
    let mut graph = ContextGraph::<EntityType, RelationshipType>::new("InvariantGraph");

    // Add DAG invariant
    graph.invariants.push(Box::new(DirectedAcyclicInvariant));

    // When: I try to create a valid DAG
    let a = graph.add_node(EntityType::System { name: "A".to_string(), version: "1.0".to_string() });
    let b = graph.add_node(EntityType::System { name: "B".to_string(), version: "1.0".to_string() });
    let c = graph.add_node(EntityType::System { name: "C".to_string(), version: "1.0".to_string() });

    assert!(graph.add_edge(a, b, RelationshipType::DependsOn).is_ok());
    assert!(graph.add_edge(b, c, RelationshipType::DependsOn).is_ok());

    // Then: Creating a cycle should fail
    assert!(graph.add_edge(c, a, RelationshipType::DependsOn).is_err());
}

// User Story 8: As a developer, I want to query nodes by their components
#[test]
fn test_user_story_8_component_queries() {
    // Given: A graph with various nodes and components
    let mut graph = ContextGraph::<EntityType, RelationshipType>::new("QueryGraph");

    // Add nodes with different labels
    let doc1 = graph.add_node(EntityType::Document {
        title: "Doc1".to_string(),
        content: "Content1".to_string(),
    });
    graph.get_node_mut(doc1).unwrap()
        .add_component(Label("Important".to_string())).unwrap();

    let doc2 = graph.add_node(EntityType::Document {
        title: "Doc2".to_string(),
        content: "Content2".to_string(),
    });
    graph.get_node_mut(doc2).unwrap()
        .add_component(Label("Draft".to_string())).unwrap();

    let doc3 = graph.add_node(EntityType::Document {
        title: "Doc3".to_string(),
        content: "Content3".to_string(),
    });
    graph.get_node_mut(doc3).unwrap()
        .add_component(Label("Important".to_string())).unwrap();

    // When: I query for nodes with specific components
    let important_nodes = graph.query_nodes_with_component::<Label>();

    // Then: I should get the correct nodes
    assert_eq!(important_nodes.len(), 3); // All have labels

    // And: I can filter further by component values
    let important_docs: Vec<_> = important_nodes.iter()
        .filter(|&&node_id| {
            if let Some(node) = graph.get_node(node_id) {
                if let Some(label) = node.get_component::<Label>() {
                    return label.0 == "Important";
                }
            }
            false
        })
        .collect();

    assert_eq!(important_docs.len(), 2);
}

// User Story 9: As a developer, I want to perform network analysis
#[test]
fn test_user_story_9_network_analysis() {
    // Given: A social network graph
    let mut graph = ContextGraph::<EntityType, RelationshipType>::new("SocialNetwork");

    // Create a network
    let alice = graph.add_node(EntityType::Person { name: "Alice".to_string(), age: 30 });
    let bob = graph.add_node(EntityType::Person { name: "Bob".to_string(), age: 25 });
    let charlie = graph.add_node(EntityType::Person { name: "Charlie".to_string(), age: 35 });
    let diana = graph.add_node(EntityType::Person { name: "Diana".to_string(), age: 28 });

    // Create relationships
    graph.add_edge(alice, bob, RelationshipType::Manages).unwrap();
    graph.add_edge(alice, charlie, RelationshipType::Manages).unwrap();
    graph.add_edge(bob, diana, RelationshipType::WorksFor).unwrap();
    graph.add_edge(charlie, diana, RelationshipType::WorksFor).unwrap();

    // Then: I should be able to perform network analysis

    // Find strongly connected components
    let components = graph.strongly_connected_components();
    assert!(!components.is_empty());

    // Calculate centrality (simplified - just using degree)
    let alice_centrality = graph.degree(alice);
    let diana_centrality = graph.degree(diana);
    assert!(alice_centrality > 0);
    assert!(diana_centrality > 0);
}

// User Story 10: As a developer, I want the graph to support Category Theory concepts
#[test]
fn test_user_story_10_category_theory() {
    // Given: Graphs that can be composed as functors
    let mut graph1 = ContextGraph::<EntityType, RelationshipType>::new("Domain1");
    let mut graph2 = ContextGraph::<EntityType, RelationshipType>::new("Domain2");

    // Build graph1
    let person = graph1.add_node(EntityType::Person { name: "John".to_string(), age: 40 });
    let org = graph1.add_node(EntityType::Organization {
        name: "TechCorp".to_string(),
        industry: "Software".to_string()
    });
    graph1.add_edge(person, org, RelationshipType::WorksFor).unwrap();

    // Build graph2 (could be a transformation of graph1)
    let system = graph2.add_node(EntityType::System {
        name: "HR System".to_string(),
        version: "2.0".to_string()
    });
    let doc = graph2.add_node(EntityType::Document {
        title: "Employee Record".to_string(),
        content: "John's employment details".to_string(),
    });
    graph2.add_edge(system, doc, RelationshipType::Contains).unwrap();

    // Then: We should be able to define morphisms between graphs
    // (This would require implementing functor/morphism support)

    // Identity morphism (graph maps to itself)
    assert_eq!(graph1.nodes().len(), 2);
    assert_eq!(graph1.edges().len(), 1);

    // Composition of morphisms should be associative
    // (Would need actual morphism implementation)
}

// User Story 11: As a developer, I want to compose cim-domain entities into graphs
#[test]
fn test_user_story_11_compose_domain_entities() {
    // Given: A graph that can hold domain entity information
    #[derive(Debug, Clone, PartialEq)]
    struct DomainEntity {
        entity_type: String,
        entity_id: uuid::Uuid,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum DomainRelationship {
        WorksFor,
        Authors,
        References,
        Contains,
        Owns,
        BelongsTo,
    }

    let mut graph = ContextGraph::<DomainEntity, DomainRelationship>::new("DomainEntityGraph");

    // When: I add domain entities as nodes

    // Add a Person entity
    let person_entity = DomainEntity {
        entity_type: "Person".to_string(),
        entity_id: uuid::Uuid::new_v4(),
        name: "Alice Smith".to_string(),
    };
    let alice_node = graph.add_node(person_entity);

    // Add components to the person node
    graph.get_node_mut(alice_node).unwrap()
        .add_component(Label("Employee".to_string())).unwrap();

    graph.get_node_mut(alice_node).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("email".to_string(), serde_json::json!("alice@example.com"));
                attrs.insert("phone".to_string(), serde_json::json!("+1-555-0123"));
                attrs.insert("position".to_string(), serde_json::json!("Senior Architect"));
                attrs
            }
        }).unwrap();

    // Add an Organization entity
    let org_entity = DomainEntity {
        entity_type: "Organization".to_string(),
        entity_id: uuid::Uuid::new_v4(),
        name: "TechCorp".to_string(),
    };
    let techcorp_node = graph.add_node(org_entity);

    graph.get_node_mut(techcorp_node).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("industry".to_string(), serde_json::json!("Technology"));
                attrs.insert("size".to_string(), serde_json::json!("Large"));
                attrs
            }
        }).unwrap();

    // Add a Document entity
    let doc_entity = DomainEntity {
        entity_type: "Document".to_string(),
        entity_id: uuid::Uuid::new_v4(),
        name: "Architecture Design v2.0".to_string(),
    };
    let doc_node = graph.add_node(doc_entity);

    // Add an Agent entity
    let agent_entity = DomainEntity {
        entity_type: "Agent".to_string(),
        entity_id: uuid::Uuid::new_v4(),
        name: "AI Assistant".to_string(),
    };
    let agent_node = graph.add_node(agent_entity);

    // Add a Workflow entity
    let workflow_entity = DomainEntity {
        entity_type: "Workflow".to_string(),
        entity_id: uuid::Uuid::new_v4(),
        name: "Document Review Process".to_string(),
    };
    let workflow_node = graph.add_node(workflow_entity);

    // Connect entities with domain relationships
    graph.add_edge(alice_node, techcorp_node, DomainRelationship::WorksFor).unwrap();
    graph.add_edge(alice_node, doc_node, DomainRelationship::Authors).unwrap();
    graph.add_edge(agent_node, doc_node, DomainRelationship::References).unwrap();
    graph.add_edge(workflow_node, doc_node, DomainRelationship::Contains).unwrap();

    // Then: The graph should contain all domain entities
    assert_eq!(graph.nodes().len(), 5);
    assert_eq!(graph.edges().len(), 4);

    // And: I can query domain entities by their type
    let person_nodes = graph.nodes().into_iter()
        .filter(|&node_id| {
            graph.get_node_value(node_id)
                .map(|entity| entity.entity_type == "Person")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    assert_eq!(person_nodes.len(), 1);

    // And: I can traverse relationships between domain entities
    let alice_relationships = graph.edges().into_iter()
        .filter_map(|edge_id| {
            graph.get_edge(edge_id).and_then(|edge| {
                if edge.source == alice_node {
                    Some((edge.target, edge.value.clone()))
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(alice_relationships.len(), 2); // WorksFor and Authors
}

// User Story 12: As a developer, I want to create workflow graphs from domain workflows
#[test]
fn test_user_story_12_workflow_graph_composition() {
    // Given: A workflow graph with states and transitions
    #[derive(Debug, Clone, PartialEq)]
    struct WorkflowState {
        name: String,
        is_terminal: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct WorkflowTransition {
        name: String,
        guard: Option<String>,
    }

    let mut workflow = ContextGraph::<WorkflowState, WorkflowTransition>::new("WorkflowGraph");

    // Create workflow states as nodes
    let draft = workflow.add_node(WorkflowState {
        name: "Draft".to_string(),
        is_terminal: false
    });

    let review = workflow.add_node(WorkflowState {
        name: "UnderReview".to_string(),
        is_terminal: false
    });

    let approved = workflow.add_node(WorkflowState {
        name: "Approved".to_string(),
        is_terminal: false
    });

    let published = workflow.add_node(WorkflowState {
        name: "Published".to_string(),
        is_terminal: true
    });

    // Add state metadata using components
    workflow.get_node_mut(draft).unwrap()
        .add_component(Label("Initial State".to_string())).unwrap();

    workflow.get_node_mut(published).unwrap()
        .add_component(Label("Terminal State".to_string())).unwrap();

    // Create transitions as edges
    workflow.add_edge(draft, review, WorkflowTransition {
        name: "Submit".to_string(),
        guard: Some("content_complete".to_string()),
    }).unwrap();

    workflow.add_edge(review, approved, WorkflowTransition {
        name: "Approve".to_string(),
        guard: Some("has_approval_authority".to_string()),
    }).unwrap();

    workflow.add_edge(review, draft, WorkflowTransition {
        name: "RequestChanges".to_string(),
        guard: None,
    }).unwrap();

    workflow.add_edge(approved, published, WorkflowTransition {
        name: "Publish".to_string(),
        guard: Some("publication_authorized".to_string()),
    }).unwrap();

    // Then: The workflow should be properly structured
    assert_eq!(workflow.nodes().len(), 4); // 4 states: draft, review, approved, published
    assert_eq!(workflow.edges().len(), 4); // 4 transitions: submit, approve, request changes, publish

    // And: I can validate workflow properties

    // Check for cycles (review -> draft -> review is allowed)
    assert!(workflow.is_cyclic());

    // Find all paths from draft to published
    let paths = workflow.find_paths(draft, published);
    assert!(!paths.is_empty());

    // Verify terminal states have no outgoing edges
    let published_outgoing = workflow.edges().into_iter()
        .filter(|&edge_id| {
            workflow.get_edge(edge_id)
                .map(|e| e.source == published)
                .unwrap_or(false)
        })
        .count();
    assert_eq!(published_outgoing, 0);
}

// User Story 13: As a developer, I want to embed concept graphs from domain
#[test]
fn test_user_story_13_concept_graph_embedding() {
    // Given: A concept graph structure
    #[derive(Debug, Clone, PartialEq)]
    struct Concept {
        name: String,
        concept_type: String,
        definition: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum ConceptRelation {
        IsA,
        PartOf,
        RelatedTo,
        DependsOn,
    }

    let mut main_graph = ContextGraph::<String, String>::new("KnowledgeGraph");
    let mut concept_graph = ContextGraph::<Concept, ConceptRelation>::new("ConceptSubgraph");

    // Build a concept hierarchy
    let ml_concept = concept_graph.add_node(Concept {
        name: "Machine Learning".to_string(),
        concept_type: "Abstract".to_string(),
        definition: Some("Study of algorithms that improve through experience".to_string()),
    });

    let nn_concept = concept_graph.add_node(Concept {
        name: "Neural Network".to_string(),
        concept_type: "Concrete".to_string(),
        definition: Some("Computing system inspired by biological neural networks".to_string()),
    });

    let dl_concept = concept_graph.add_node(Concept {
        name: "Deep Learning".to_string(),
        concept_type: "Composite".to_string(),
        definition: Some("Machine learning using deep neural networks".to_string()),
    });

    // Connect concepts
    concept_graph.add_edge(nn_concept, ml_concept, ConceptRelation::IsA).unwrap();
    concept_graph.add_edge(dl_concept, nn_concept, ConceptRelation::DependsOn).unwrap();

    // Add conceptual space mapping
    for node_id in concept_graph.nodes() {
        if let Some(node) = concept_graph.get_node_mut(node_id) {
            node.add_component(Properties {
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("embedding".to_string(), serde_json::json!([0.1, 0.2, 0.3]));
                    attrs.insert("confidence".to_string(), serde_json::json!(0.95));
                    attrs
                }
            }).unwrap();
        }
    }

    // When: I create a reference to the concept graph in the main graph
    let knowledge_domain = main_graph.add_node("ML Knowledge Domain".to_string());

    // Add metadata about the embedded graph
    main_graph.get_node_mut(knowledge_domain).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("concept_count".to_string(), serde_json::json!(concept_graph.nodes().len()));
                attrs.insert("relation_count".to_string(), serde_json::json!(concept_graph.edges().len()));
                attrs.insert("domain".to_string(), serde_json::json!("Machine Learning"));
                attrs
            }
        }).unwrap();

    // Store reference to the concept subgraph
    main_graph.get_node_mut(knowledge_domain).unwrap()
        .add_component(Subgraph {
            graph: Box::new(concept_graph)
        }).unwrap();

    // Then: The main graph contains the embedded concept graph
    assert_eq!(main_graph.nodes().len(), 1);

    // And: I can access the embedded concepts
    if let Some(node) = main_graph.get_node(knowledge_domain) {
        assert!(node.has_component::<Subgraph<Concept, ConceptRelation>>());
    }
}

// User Story 14: As a developer, I want to visualize domain events as a graph
#[test]
fn test_user_story_14_event_flow_graph() {
    // Given: Event and aggregate node types
    #[derive(Debug, Clone, PartialEq)]
    enum EventNode {
        Event {
            event_type: String,
            aggregate_id: uuid::Uuid,
            timestamp: String,
        },
        Aggregate {
            aggregate_type: String,
            aggregate_id: uuid::Uuid,
        },
    }

    #[derive(Debug, Clone, PartialEq)]
    enum EventEdge {
        Triggers,
        BelongsTo,
        CausedBy,
        Temporal,
    }

    let mut event_graph = ContextGraph::<EventNode, EventEdge>::new("EventFlowGraph");

    // Add aggregate nodes
    let person_agg = event_graph.add_node(EventNode::Aggregate {
        aggregate_type: "Person".to_string(),
        aggregate_id: uuid::Uuid::new_v4(),
    });

    let org_agg = event_graph.add_node(EventNode::Aggregate {
        aggregate_type: "Organization".to_string(),
        aggregate_id: uuid::Uuid::new_v4(),
    });

    // Add event nodes
    let person_registered = event_graph.add_node(EventNode::Event {
        event_type: "PersonRegistered".to_string(),
        aggregate_id: uuid::Uuid::new_v4(),
        timestamp: "2024-01-15T10:00:00Z".to_string(),
    });

    let org_created = event_graph.add_node(EventNode::Event {
        event_type: "OrganizationCreated".to_string(),
        aggregate_id: uuid::Uuid::new_v4(),
        timestamp: "2024-01-15T10:05:00Z".to_string(),
    });

    let workflow_started = event_graph.add_node(EventNode::Event {
        event_type: "WorkflowStarted".to_string(),
        aggregate_id: uuid::Uuid::new_v4(),
        timestamp: "2024-01-15T10:10:00Z".to_string(),
    });

    // Connect events to aggregates
    event_graph.add_edge(person_registered, person_agg, EventEdge::BelongsTo).unwrap();
    event_graph.add_edge(org_created, org_agg, EventEdge::BelongsTo).unwrap();

    // Connect events causally
    event_graph.add_edge(person_registered, org_created, EventEdge::Triggers).unwrap();
    event_graph.add_edge(org_created, workflow_started, EventEdge::CausedBy).unwrap();

    // Add temporal ordering
    event_graph.add_edge(person_registered, org_created, EventEdge::Temporal).unwrap();
    event_graph.add_edge(org_created, workflow_started, EventEdge::Temporal).unwrap();

    // Add event metadata
    event_graph.get_node_mut(person_registered).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("correlation_id".to_string(), serde_json::json!("session-123"));
                attrs.insert("user_id".to_string(), serde_json::json!("user-456"));
                attrs
            }
        }).unwrap();

    // Then: The event flow graph captures the event relationships
    assert_eq!(event_graph.nodes().len(), 5); // 2 aggregates + 3 events
    assert_eq!(event_graph.edges().len(), 6); // Various relationships

    // And: I can trace event causality
    let causal_chain = event_graph.find_paths(person_registered, workflow_started);
    assert!(!causal_chain.is_empty());

    // And: I can query events by aggregate
    let person_events = event_graph.edges().into_iter()
        .filter_map(|edge_id| {
            event_graph.get_edge(edge_id).and_then(|edge| {
                if edge.target == person_agg && &edge.value == &EventEdge::BelongsTo {
                    Some(edge.source)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(person_events.len(), 1);
}

// User Story 15: As a developer, I want to ensure domain entity composition preserves invariants
#[test]
fn test_user_story_15_domain_invariant_preservation() {
    // Given: A graph with domain-specific invariants
    #[derive(Debug, Clone, PartialEq)]
    struct DomainNode {
        node_type: String,
        id: uuid::Uuid,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum DomainEdge {
        WorksFor,
        Owns,
        BelongsTo,
        Governs,
    }

    // Custom invariant for domain entities
    struct DomainEntityInvariant;

    impl GraphInvariant<DomainNode, DomainEdge> for DomainEntityInvariant {
        fn check(&self, graph: &ContextGraph<DomainNode, DomainEdge>) -> GraphResult<()> {
            // Check domain-specific rules
            // Example: Every Person must belong to an Organization
            for node_id in graph.nodes() {
                if let Some(node) = graph.get_node_value(node_id) {
                    if node.node_type == "Person" {
                        // Check if this person has a WorksFor edge
                        let has_org = graph.edges().iter().any(|&edge_id| {
                            graph.get_edge(edge_id)
                                .map(|e| e.source == node_id && matches!(e.value, DomainEdge::WorksFor))
                                .unwrap_or(false)
                        });

                        if !has_org {
                            return Err(GraphError::InvariantViolation(
                                format!("Person {} must belong to an organization", node.name)
                            ));
                        }
                    }
                }
            }
            Ok(())
        }

        fn name(&self) -> &str {
            "DomainEntityInvariant"
        }
    }

    let mut graph = ContextGraph::<DomainNode, DomainEdge>::new("InvariantGraph");
    graph.invariants.push(Box::new(DomainEntityInvariant));

    // When: I build a valid domain graph
    let person = graph.add_node(DomainNode {
        node_type: "Person".to_string(),
        id: uuid::Uuid::new_v4(),
        name: "John Doe".to_string(),
    });

    let org = graph.add_node(DomainNode {
        node_type: "Organization".to_string(),
        id: uuid::Uuid::new_v4(),
        name: "Acme Corp".to_string(),
    });

    // This should succeed - person belongs to organization
    assert!(graph.add_edge(person, org, DomainEdge::WorksFor).is_ok());

    // Then: The invariants are maintained
    assert!(graph.check_invariants().is_ok());

    // And: I can compose complex domain structures while maintaining invariants
    let policy = graph.add_node(DomainNode {
        node_type: "Policy".to_string(),
        id: uuid::Uuid::new_v4(),
        name: "Approval Policy".to_string(),
    });

    // Connect policy to organization
    assert!(graph.add_edge(org, policy, DomainEdge::Owns).is_ok());

    // Verify the complete domain graph maintains all invariants
    assert!(graph.check_invariants().is_ok());

    // Test invariant violation
    let orphan_person = graph.add_node(DomainNode {
        node_type: "Person".to_string(),
        id: uuid::Uuid::new_v4(),
        name: "Orphan Person".to_string(),
    });

    // This should fail the invariant check
    assert!(graph.check_invariants().is_err());
}

// User Story 16: As a developer, I want to compose a document approval workflow
#[test]
fn test_user_story_16_document_approval_workflow() {
    // Given: A document approval workflow with people, organizations, and policies
    #[derive(Debug, Clone, PartialEq)]
    enum WorkflowNode {
        Person { id: uuid::Uuid, name: String, role: String },
        Document { id: uuid::Uuid, title: String, version: String },
        Policy { id: uuid::Uuid, name: String, policy_type: String },
        ApprovalState { name: String, approved_by: Option<uuid::Uuid> },
    }

    #[derive(Debug, Clone, PartialEq)]
    enum WorkflowEdge {
        Authors,
        RequiresApproval,
        ApprovedBy,
        GovernedBy,
        Transitions,
    }

    let mut workflow = ContextGraph::<WorkflowNode, WorkflowEdge>::new("DocumentApprovalWorkflow");

    // Add people
    let alice_id = uuid::Uuid::new_v4();
    let alice = workflow.add_node(WorkflowNode::Person {
        id: alice_id,
        name: "Alice Smith".to_string(),
        role: "Author".to_string(),
    });

    let bob_id = uuid::Uuid::new_v4();
    let bob = workflow.add_node(WorkflowNode::Person {
        id: bob_id,
        name: "Bob Johnson".to_string(),
        role: "Manager".to_string(),
    });

    let carol_id = uuid::Uuid::new_v4();
    let carol = workflow.add_node(WorkflowNode::Person {
        id: carol_id,
        name: "Carol Davis".to_string(),
        role: "Compliance Officer".to_string(),
    });

    // Add document
    let doc = workflow.add_node(WorkflowNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "Q4 Financial Report".to_string(),
        version: "1.0".to_string(),
    });

    // Add policy
    let policy = workflow.add_node(WorkflowNode::Policy {
        id: uuid::Uuid::new_v4(),
        name: "Financial Document Approval Policy".to_string(),
        policy_type: "Sequential Approval".to_string(),
    });

    // Add approval states
    let draft = workflow.add_node(WorkflowNode::ApprovalState {
        name: "Draft".to_string(),
        approved_by: None,
    });

    let manager_review = workflow.add_node(WorkflowNode::ApprovalState {
        name: "Manager Review".to_string(),
        approved_by: None,
    });

    let compliance_review = workflow.add_node(WorkflowNode::ApprovalState {
        name: "Compliance Review".to_string(),
        approved_by: None,
    });

    let approved = workflow.add_node(WorkflowNode::ApprovalState {
        name: "Approved".to_string(),
        approved_by: Some(carol_id),
    });

    // Connect the workflow
    workflow.add_edge(alice, doc, WorkflowEdge::Authors).unwrap();
    workflow.add_edge(doc, draft, WorkflowEdge::RequiresApproval).unwrap();
    workflow.add_edge(draft, manager_review, WorkflowEdge::Transitions).unwrap();
    workflow.add_edge(manager_review, compliance_review, WorkflowEdge::Transitions).unwrap();
    workflow.add_edge(compliance_review, approved, WorkflowEdge::Transitions).unwrap();

    workflow.add_edge(bob, manager_review, WorkflowEdge::ApprovedBy).unwrap();
    workflow.add_edge(carol, compliance_review, WorkflowEdge::ApprovedBy).unwrap();

    workflow.add_edge(doc, policy, WorkflowEdge::GovernedBy).unwrap();

    // Add metadata to track approval history
    workflow.get_node_mut(manager_review).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("approved_at".to_string(), serde_json::json!("2024-01-15T14:30:00Z"));
                attrs.insert("comments".to_string(), serde_json::json!("Looks good, forwarding to compliance"));
                attrs
            }
        }).unwrap();

    // Then: The workflow should be properly structured
    assert_eq!(workflow.nodes().len(), 9); // 3 people + 1 doc + 1 policy + 4 states
    assert_eq!(workflow.edges().len(), 8); // Fixed count: 1 authors + 1 requires + 3 transitions + 2 approved by + 1 governed by

    // And: I can trace the approval path
    let approval_path = workflow.find_paths(draft, approved);
    assert!(!approval_path.is_empty());

    // And: I can query who can approve at each stage
    let manager_approvers = workflow.edges().into_iter()
        .filter_map(|edge_id| {
            workflow.get_edge(edge_id).and_then(|edge| {
                if edge.target == manager_review && matches!(edge.value, WorkflowEdge::ApprovedBy) {
                    Some(edge.source)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(manager_approvers.len(), 1);
    assert_eq!(manager_approvers[0], bob);
}

// User Story 17: As a developer, I want to compose an agent-assisted research workflow
#[test]
fn test_user_story_17_agent_research_workflow() {
    // Given: A research workflow with agents, people, and documents
    #[derive(Debug, Clone, PartialEq)]
    enum ResearchNode {
        Person { id: uuid::Uuid, name: String, expertise: String },
        Agent { id: uuid::Uuid, name: String, capabilities: Vec<String> },
        Document { id: uuid::Uuid, title: String, doc_type: String },
        Location { id: uuid::Uuid, name: String, url: Option<String> },
        Task { id: uuid::Uuid, name: String, status: String },
    }

    #[derive(Debug, Clone, PartialEq)]
    enum ResearchEdge {
        Assigns,
        Executes,
        Produces,
        References,
        LocatedAt,
        DependsOn,
    }

    let mut workflow = ContextGraph::<ResearchNode, ResearchEdge>::new("ResearchWorkflow");

    // Add researcher
    let researcher = workflow.add_node(ResearchNode::Person {
        id: uuid::Uuid::new_v4(),
        name: "Dr. Sarah Chen".to_string(),
        expertise: "Machine Learning".to_string(),
    });

    // Add AI agents
    let search_agent = workflow.add_node(ResearchNode::Agent {
        id: uuid::Uuid::new_v4(),
        name: "Literature Search Agent".to_string(),
        capabilities: vec!["arxiv_search".to_string(), "semantic_search".to_string()],
    });

    let analysis_agent = workflow.add_node(ResearchNode::Agent {
        id: uuid::Uuid::new_v4(),
        name: "Paper Analysis Agent".to_string(),
        capabilities: vec!["summarization".to_string(), "key_point_extraction".to_string()],
    });

    let synthesis_agent = workflow.add_node(ResearchNode::Agent {
        id: uuid::Uuid::new_v4(),
        name: "Research Synthesis Agent".to_string(),
        capabilities: vec!["comparison".to_string(), "insight_generation".to_string()],
    });

    // Add tasks
    let search_task = workflow.add_node(ResearchNode::Task {
        id: uuid::Uuid::new_v4(),
        name: "Search Recent ML Papers".to_string(),
        status: "Completed".to_string(),
    });

    let analysis_task = workflow.add_node(ResearchNode::Task {
        id: uuid::Uuid::new_v4(),
        name: "Analyze Top 10 Papers".to_string(),
        status: "In Progress".to_string(),
    });

    let synthesis_task = workflow.add_node(ResearchNode::Task {
        id: uuid::Uuid::new_v4(),
        name: "Create Research Summary".to_string(),
        status: "Pending".to_string(),
    });

    // Add documents
    let search_results = workflow.add_node(ResearchNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "ML Papers Search Results".to_string(),
        doc_type: "Search Results".to_string(),
    });

    let paper_summaries = workflow.add_node(ResearchNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "Individual Paper Summaries".to_string(),
        doc_type: "Analysis".to_string(),
    });

    let research_report = workflow.add_node(ResearchNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "Comprehensive ML Research Report".to_string(),
        doc_type: "Synthesis".to_string(),
    });

    // Add locations
    let arxiv = workflow.add_node(ResearchNode::Location {
        id: uuid::Uuid::new_v4(),
        name: "arXiv".to_string(),
        url: Some("https://arxiv.org".to_string()),
    });

    // Connect the workflow
    workflow.add_edge(researcher, search_task, ResearchEdge::Assigns).unwrap();
    workflow.add_edge(researcher, analysis_task, ResearchEdge::Assigns).unwrap();
    workflow.add_edge(researcher, synthesis_task, ResearchEdge::Assigns).unwrap();

    workflow.add_edge(search_agent, search_task, ResearchEdge::Executes).unwrap();
    workflow.add_edge(analysis_agent, analysis_task, ResearchEdge::Executes).unwrap();
    workflow.add_edge(synthesis_agent, synthesis_task, ResearchEdge::Executes).unwrap();

    workflow.add_edge(search_task, search_results, ResearchEdge::Produces).unwrap();
    workflow.add_edge(analysis_task, paper_summaries, ResearchEdge::Produces).unwrap();
    workflow.add_edge(synthesis_task, research_report, ResearchEdge::Produces).unwrap();

    workflow.add_edge(search_results, arxiv, ResearchEdge::LocatedAt).unwrap();
    workflow.add_edge(analysis_task, search_results, ResearchEdge::References).unwrap();
    workflow.add_edge(synthesis_task, paper_summaries, ResearchEdge::References).unwrap();

    workflow.add_edge(analysis_task, search_task, ResearchEdge::DependsOn).unwrap();
    workflow.add_edge(synthesis_task, analysis_task, ResearchEdge::DependsOn).unwrap();

    // Add execution metrics
    workflow.get_node_mut(search_task).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("papers_found".to_string(), serde_json::json!(127));
                attrs.insert("execution_time_ms".to_string(), serde_json::json!(3500));
                attrs.insert("relevance_threshold".to_string(), serde_json::json!(0.85));
                attrs
            }
        }).unwrap();

    // Then: The workflow captures the agent-assisted research process
    assert_eq!(workflow.nodes().len(), 11);
    assert_eq!(workflow.edges().len(), 14);

    // And: I can identify task dependencies
    let task_deps = workflow.edges().into_iter()
        .filter(|&edge_id| {
            workflow.get_edge_value(edge_id)
                .map(|v| matches!(v, ResearchEdge::DependsOn))
                .unwrap_or(false)
        })
        .count();

    assert_eq!(task_deps, 2);

    // And: I can trace which agents produce which documents
    let agent_outputs = workflow.edges().into_iter()
        .filter_map(|edge_id| {
            workflow.get_edge(edge_id).and_then(|edge| {
                if matches!(edge.value, ResearchEdge::Produces) {
                    workflow.get_node_value(edge.source).and_then(|node| {
                        match node {
                            ResearchNode::Task { name, .. } => Some(name.clone()),
                            _ => None
                        }
                    })
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(agent_outputs.len(), 3);
}

// User Story 18: As a developer, I want to compose a location-based service deployment workflow
#[test]
fn test_user_story_18_location_based_deployment() {
    // Given: A service deployment workflow across multiple locations
    #[derive(Debug, Clone, PartialEq)]
    enum DeploymentNode {
        Organization { id: uuid::Uuid, name: String },
        Location { id: uuid::Uuid, name: String, region: String, location_type: String },
        Agent { id: uuid::Uuid, name: String, agent_type: String },
        Policy { id: uuid::Uuid, name: String, scope: String },
        Service { id: uuid::Uuid, name: String, version: String },
    }

    #[derive(Debug, Clone, PartialEq)]
    enum DeploymentEdge {
        Owns,
        LocatedAt,
        DeployedAt,
        Manages,
        EnforcedBy,
        Monitors,
    }

    let mut workflow = ContextGraph::<DeploymentNode, DeploymentEdge>::new("ServiceDeploymentWorkflow");

    // Add organization
    let org = workflow.add_node(DeploymentNode::Organization {
        id: uuid::Uuid::new_v4(),
        name: "GlobalTech Inc".to_string(),
    });

    // Add locations
    let hq = workflow.add_node(DeploymentNode::Location {
        id: uuid::Uuid::new_v4(),
        name: "Headquarters".to_string(),
        region: "US-East".to_string(),
        location_type: "Physical".to_string(),
    });

    let eu_dc = workflow.add_node(DeploymentNode::Location {
        id: uuid::Uuid::new_v4(),
        name: "EU Data Center".to_string(),
        region: "EU-West".to_string(),
        location_type: "Physical".to_string(),
    });

    let cloud_region = workflow.add_node(DeploymentNode::Location {
        id: uuid::Uuid::new_v4(),
        name: "AWS us-east-1".to_string(),
        region: "US-East".to_string(),
        location_type: "Virtual".to_string(),
    });

    // Add deployment agents
    let deploy_agent = workflow.add_node(DeploymentNode::Agent {
        id: uuid::Uuid::new_v4(),
        name: "Deployment Orchestrator".to_string(),
        agent_type: "Automation".to_string(),
    });

    let monitor_agent = workflow.add_node(DeploymentNode::Agent {
        id: uuid::Uuid::new_v4(),
        name: "Health Monitor".to_string(),
        agent_type: "Monitoring".to_string(),
    });

    // Add policies
    let gdpr_policy = workflow.add_node(DeploymentNode::Policy {
        id: uuid::Uuid::new_v4(),
        name: "GDPR Compliance Policy".to_string(),
        scope: "EU Region".to_string(),
    });

    let security_policy = workflow.add_node(DeploymentNode::Policy {
        id: uuid::Uuid::new_v4(),
        name: "Zero Trust Security Policy".to_string(),
        scope: "Global".to_string(),
    });

    // Add services
    let api_service = workflow.add_node(DeploymentNode::Service {
        id: uuid::Uuid::new_v4(),
        name: "Customer API".to_string(),
        version: "2.3.0".to_string(),
    });

    let db_service = workflow.add_node(DeploymentNode::Service {
        id: uuid::Uuid::new_v4(),
        name: "Customer Database".to_string(),
        version: "5.7".to_string(),
    });

    // Connect the deployment topology
    workflow.add_edge(org, hq, DeploymentEdge::Owns).unwrap();
    workflow.add_edge(org, eu_dc, DeploymentEdge::Owns).unwrap();
    workflow.add_edge(org, cloud_region, DeploymentEdge::Owns).unwrap();

    workflow.add_edge(deploy_agent, hq, DeploymentEdge::LocatedAt).unwrap();
    workflow.add_edge(monitor_agent, cloud_region, DeploymentEdge::LocatedAt).unwrap();

    workflow.add_edge(api_service, cloud_region, DeploymentEdge::DeployedAt).unwrap();
    workflow.add_edge(api_service, eu_dc, DeploymentEdge::DeployedAt).unwrap();
    workflow.add_edge(db_service, eu_dc, DeploymentEdge::DeployedAt).unwrap();

    workflow.add_edge(deploy_agent, api_service, DeploymentEdge::Manages).unwrap();
    workflow.add_edge(deploy_agent, db_service, DeploymentEdge::Manages).unwrap();
    workflow.add_edge(monitor_agent, api_service, DeploymentEdge::Monitors).unwrap();
    workflow.add_edge(monitor_agent, db_service, DeploymentEdge::Monitors).unwrap();

    workflow.add_edge(eu_dc, gdpr_policy, DeploymentEdge::EnforcedBy).unwrap();
    workflow.add_edge(api_service, security_policy, DeploymentEdge::EnforcedBy).unwrap();

    // Add deployment metadata
    workflow.get_node_mut(api_service).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("replicas".to_string(), serde_json::json!(3));
                attrs.insert("load_balancer".to_string(), serde_json::json!("round-robin"));
                attrs.insert("health_check_url".to_string(), serde_json::json!("/health"));
                attrs
            }
        }).unwrap();

    // Then: The deployment topology is properly mapped
    assert_eq!(workflow.nodes().len(), 10);
    assert_eq!(workflow.edges().len(), 14);

    // And: I can identify location-specific policies
    let eu_policies = workflow.edges().into_iter()
        .filter_map(|edge_id| {
            workflow.get_edge(edge_id).and_then(|edge| {
                if edge.source == eu_dc && matches!(edge.value, DeploymentEdge::EnforcedBy) {
                    Some(edge.target)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(eu_policies.len(), 1);

    // And: I can trace service deployment locations
    let api_locations = workflow.edges().into_iter()
        .filter_map(|edge_id| {
            workflow.get_edge(edge_id).and_then(|edge| {
                if edge.source == api_service && matches!(edge.value, DeploymentEdge::DeployedAt) {
                    workflow.get_node_value(edge.target)
                } else {
                    None
                }
            })
        })
        .count();

    assert_eq!(api_locations, 2); // Deployed in 2 locations
}

// User Story 19: As a developer, I want to compose a multi-organization collaboration workflow
#[test]
fn test_user_story_19_multi_org_collaboration() {
    // Given: A collaboration workflow between multiple organizations
    #[derive(Debug, Clone, PartialEq)]
    enum CollaborationNode {
        Organization { id: uuid::Uuid, name: String, org_type: String },
        Person { id: uuid::Uuid, name: String, org_id: uuid::Uuid },
        Document { id: uuid::Uuid, title: String, classification: String },
        Policy { id: uuid::Uuid, name: String, policy_type: String },
        Agreement { id: uuid::Uuid, name: String, status: String },
    }

    #[derive(Debug, Clone, PartialEq)]
    enum CollaborationEdge {
        EmployedBy,
        PartyTo,
        Shares,
        RestrictedBy,
        Negotiates,
        Signs,
    }

    let mut workflow = ContextGraph::<CollaborationNode, CollaborationEdge>::new("MultiOrgCollaboration");

    // Add organizations
    let corp_a_id = uuid::Uuid::new_v4();
    let corp_a = workflow.add_node(CollaborationNode::Organization {
        id: corp_a_id,
        name: "TechCorp A".to_string(),
        org_type: "Technology".to_string(),
    });

    let corp_b_id = uuid::Uuid::new_v4();
    let corp_b = workflow.add_node(CollaborationNode::Organization {
        id: corp_b_id,
        name: "FinanceOrg B".to_string(),
        org_type: "Financial Services".to_string(),
    });

    // Add people from each organization
    let alice = workflow.add_node(CollaborationNode::Person {
        id: uuid::Uuid::new_v4(),
        name: "Alice (Legal Counsel)".to_string(),
        org_id: corp_a_id,
    });

    let bob = workflow.add_node(CollaborationNode::Person {
        id: uuid::Uuid::new_v4(),
        name: "Bob (Product Manager)".to_string(),
        org_id: corp_a_id,
    });

    let charlie = workflow.add_node(CollaborationNode::Person {
        id: uuid::Uuid::new_v4(),
        name: "Charlie (Compliance Officer)".to_string(),
        org_id: corp_b_id,
    });

    let diana = workflow.add_node(CollaborationNode::Person {
        id: uuid::Uuid::new_v4(),
        name: "Diana (Technical Lead)".to_string(),
        org_id: corp_b_id,
    });

    // Add shared documents
    let tech_spec = workflow.add_node(CollaborationNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "API Integration Specification".to_string(),
        classification: "Confidential".to_string(),
    });

    let data_agreement = workflow.add_node(CollaborationNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "Data Sharing Agreement".to_string(),
        classification: "Legal".to_string(),
    });

    // Add policies
    let nda_policy = workflow.add_node(CollaborationNode::Policy {
        id: uuid::Uuid::new_v4(),
        name: "Mutual NDA".to_string(),
        policy_type: "Legal".to_string(),
    });

    let data_policy = workflow.add_node(CollaborationNode::Policy {
        id: uuid::Uuid::new_v4(),
        name: "Data Protection Policy".to_string(),
        policy_type: "Compliance".to_string(),
    });

    // Add agreement
    let partnership = workflow.add_node(CollaborationNode::Agreement {
        id: uuid::Uuid::new_v4(),
        name: "Strategic Partnership Agreement".to_string(),
        status: "Under Negotiation".to_string(),
    });

    // Connect the collaboration network
    workflow.add_edge(alice, corp_a, CollaborationEdge::EmployedBy).unwrap();
    workflow.add_edge(bob, corp_a, CollaborationEdge::EmployedBy).unwrap();
    workflow.add_edge(charlie, corp_b, CollaborationEdge::EmployedBy).unwrap();
    workflow.add_edge(diana, corp_b, CollaborationEdge::EmployedBy).unwrap();

    workflow.add_edge(corp_a, partnership, CollaborationEdge::PartyTo).unwrap();
    workflow.add_edge(corp_b, partnership, CollaborationEdge::PartyTo).unwrap();

    workflow.add_edge(bob, tech_spec, CollaborationEdge::Shares).unwrap();
    workflow.add_edge(diana, tech_spec, CollaborationEdge::Shares).unwrap();

    workflow.add_edge(alice, data_agreement, CollaborationEdge::Negotiates).unwrap();
    workflow.add_edge(charlie, data_agreement, CollaborationEdge::Negotiates).unwrap();

    workflow.add_edge(tech_spec, nda_policy, CollaborationEdge::RestrictedBy).unwrap();
    workflow.add_edge(data_agreement, data_policy, CollaborationEdge::RestrictedBy).unwrap();

    workflow.add_edge(alice, partnership, CollaborationEdge::Signs).unwrap();
    workflow.add_edge(charlie, partnership, CollaborationEdge::Signs).unwrap();

    // Add collaboration metadata
    workflow.get_node_mut(partnership).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("start_date".to_string(), serde_json::json!("2024-02-01"));
                attrs.insert("duration_months".to_string(), serde_json::json!(24));
                attrs.insert("value".to_string(), serde_json::json!("$5M"));
                attrs
            }
        }).unwrap();

    // Then: The collaboration structure is properly captured
    assert_eq!(workflow.nodes().len(), 11); // Fixed count: 2 orgs + 4 people + 2 docs + 2 policies + 1 agreement
    assert_eq!(workflow.edges().len(), 14);

    // And: I can identify cross-organization document sharing
    let shared_docs = workflow.edges().into_iter()
        .filter(|&edge_id| {
            workflow.get_edge_value(edge_id)
                .map(|v| matches!(v, CollaborationEdge::Shares))
                .unwrap_or(false)
        })
        .count();

    assert_eq!(shared_docs, 2);

    // And: I can trace who can sign agreements
    let signers = workflow.edges().into_iter()
        .filter_map(|edge_id| {
            workflow.get_edge(edge_id).and_then(|edge| {
                if matches!(edge.value, CollaborationEdge::Signs) {
                    workflow.get_node_value(edge.source)
                } else {
                    None
                }
            })
        })
        .count();

    assert_eq!(signers, 2);
}

// User Story 20: As a developer, I want to compose a policy-driven access control workflow
#[test]
fn test_user_story_20_policy_driven_access_control() {
    // Given: An access control workflow with policies, people, and resources
    #[derive(Debug, Clone, PartialEq)]
    enum AccessControlNode {
        Person { id: uuid::Uuid, name: String, clearance: String },
        Organization { id: uuid::Uuid, name: String },
        Document { id: uuid::Uuid, title: String, sensitivity: String },
        Location { id: uuid::Uuid, name: String, security_zone: String },
        Policy { id: uuid::Uuid, name: String, enforcement: String },
        Role { id: uuid::Uuid, name: String, permissions: Vec<String> },
    }

    #[derive(Debug, Clone, PartialEq)]
    enum AccessControlEdge {
        BelongsTo,
        HasRole,
        CanAccess,
        LocatedIn,
        EnforcedAt,
        Requires,
        Grants,
    }

    let mut workflow = ContextGraph::<AccessControlNode, AccessControlEdge>::new("AccessControlWorkflow");

    // Add organization
    let org = workflow.add_node(AccessControlNode::Organization {
        id: uuid::Uuid::new_v4(),
        name: "SecureGov Agency".to_string(),
    });

    // Add people with different clearance levels
    let alice = workflow.add_node(AccessControlNode::Person {
        id: uuid::Uuid::new_v4(),
        name: "Alice Anderson".to_string(),
        clearance: "Top Secret".to_string(),
    });

    let bob = workflow.add_node(AccessControlNode::Person {
        id: uuid::Uuid::new_v4(),
        name: "Bob Brown".to_string(),
        clearance: "Secret".to_string(),
    });

    let charlie = workflow.add_node(AccessControlNode::Person {
        id: uuid::Uuid::new_v4(),
        name: "Charlie Chen".to_string(),
        clearance: "Confidential".to_string(),
    });

    // Add roles
    let admin_role = workflow.add_node(AccessControlNode::Role {
        id: uuid::Uuid::new_v4(),
        name: "System Administrator".to_string(),
        permissions: vec!["read".to_string(), "write".to_string(), "delete".to_string()],
    });

    let analyst_role = workflow.add_node(AccessControlNode::Role {
        id: uuid::Uuid::new_v4(),
        name: "Intelligence Analyst".to_string(),
        permissions: vec!["read".to_string(), "annotate".to_string()],
    });

    let auditor_role = workflow.add_node(AccessControlNode::Role {
        id: uuid::Uuid::new_v4(),
        name: "Compliance Auditor".to_string(),
        permissions: vec!["read".to_string(), "audit_log".to_string()],
    });

    // Add documents with different sensitivity levels
    let top_secret_doc = workflow.add_node(AccessControlNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "Operation Thunderbolt Plans".to_string(),
        sensitivity: "Top Secret".to_string(),
    });

    let secret_doc = workflow.add_node(AccessControlNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "Quarterly Intelligence Report".to_string(),
        sensitivity: "Secret".to_string(),
    });

    let confidential_doc = workflow.add_node(AccessControlNode::Document {
        id: uuid::Uuid::new_v4(),
        title: "Personnel Records".to_string(),
        sensitivity: "Confidential".to_string(),
    });

    // Add locations
    let scif = workflow.add_node(AccessControlNode::Location {
        id: uuid::Uuid::new_v4(),
        name: "SCIF Room 101".to_string(),
        security_zone: "Restricted".to_string(),
    });

    let secure_server = workflow.add_node(AccessControlNode::Location {
        id: uuid::Uuid::new_v4(),
        name: "Secure Server Room".to_string(),
        security_zone: "Controlled".to_string(),
    });

    // Add policies
    let clearance_policy = workflow.add_node(AccessControlNode::Policy {
        id: uuid::Uuid::new_v4(),
        name: "Clearance-Based Access Policy".to_string(),
        enforcement: "Mandatory".to_string(),
    });

    let need_to_know = workflow.add_node(AccessControlNode::Policy {
        id: uuid::Uuid::new_v4(),
        name: "Need-to-Know Policy".to_string(),
        enforcement: "Mandatory".to_string(),
    });

    let audit_policy = workflow.add_node(AccessControlNode::Policy {
        id: uuid::Uuid::new_v4(),
        name: "Access Audit Policy".to_string(),
        enforcement: "Automatic".to_string(),
    });

    // Connect the access control structure
    workflow.add_edge(alice, org, AccessControlEdge::BelongsTo).unwrap();
    workflow.add_edge(bob, org, AccessControlEdge::BelongsTo).unwrap();
    workflow.add_edge(charlie, org, AccessControlEdge::BelongsTo).unwrap();

    workflow.add_edge(alice, admin_role, AccessControlEdge::HasRole).unwrap();
    workflow.add_edge(bob, analyst_role, AccessControlEdge::HasRole).unwrap();
    workflow.add_edge(charlie, auditor_role, AccessControlEdge::HasRole).unwrap();

    workflow.add_edge(top_secret_doc, scif, AccessControlEdge::LocatedIn).unwrap();
    workflow.add_edge(secret_doc, secure_server, AccessControlEdge::LocatedIn).unwrap();
    workflow.add_edge(confidential_doc, secure_server, AccessControlEdge::LocatedIn).unwrap();

    workflow.add_edge(top_secret_doc, clearance_policy, AccessControlEdge::Requires).unwrap();
    workflow.add_edge(secret_doc, clearance_policy, AccessControlEdge::Requires).unwrap();
    workflow.add_edge(confidential_doc, clearance_policy, AccessControlEdge::Requires).unwrap();

    workflow.add_edge(scif, need_to_know, AccessControlEdge::EnforcedAt).unwrap();
    workflow.add_edge(secure_server, audit_policy, AccessControlEdge::EnforcedAt).unwrap();

    // Grant access based on clearance and role
    workflow.add_edge(alice, top_secret_doc, AccessControlEdge::CanAccess).unwrap();
    workflow.add_edge(alice, secret_doc, AccessControlEdge::CanAccess).unwrap();
    workflow.add_edge(alice, confidential_doc, AccessControlEdge::CanAccess).unwrap();

    workflow.add_edge(bob, secret_doc, AccessControlEdge::CanAccess).unwrap();
    workflow.add_edge(bob, confidential_doc, AccessControlEdge::CanAccess).unwrap();

    workflow.add_edge(charlie, confidential_doc, AccessControlEdge::CanAccess).unwrap();

    workflow.add_edge(admin_role, top_secret_doc, AccessControlEdge::Grants).unwrap();
    workflow.add_edge(analyst_role, secret_doc, AccessControlEdge::Grants).unwrap();
    workflow.add_edge(auditor_role, confidential_doc, AccessControlEdge::Grants).unwrap();

    // Add access control metadata
    workflow.get_node_mut(alice).unwrap()
        .add_component(Properties {
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("clearance_granted".to_string(), serde_json::json!("2020-01-15"));
                attrs.insert("last_review".to_string(), serde_json::json!("2023-12-01"));
                attrs.insert("access_logs_enabled".to_string(), serde_json::json!(true));
                attrs
            }
        }).unwrap();

    // Then: The access control structure is properly defined
    assert_eq!(workflow.nodes().len(), 15);
    assert_eq!(workflow.edges().len(), 23); // Fixed count: 3 BelongsTo + 3 HasRole + 3 LocatedIn + 3 Requires + 2 EnforcedAt + 6 CanAccess + 3 Grants

    // And: I can verify clearance-based access
    let alice_access = workflow.edges().into_iter()
        .filter(|&edge_id| {
            workflow.get_edge(edge_id)
                .map(|e| e.source == alice && matches!(e.value, AccessControlEdge::CanAccess))
                .unwrap_or(false)
        })
        .count();

    assert_eq!(alice_access, 3); // Can access all three documents

    let bob_access = workflow.edges().into_iter()
        .filter(|&edge_id| {
            workflow.get_edge(edge_id)
                .map(|e| e.source == bob && matches!(e.value, AccessControlEdge::CanAccess))
                .unwrap_or(false)
        })
        .count();

    assert_eq!(bob_access, 2); // Can access secret and confidential

    let charlie_access = workflow.edges().into_iter()
        .filter(|&edge_id| {
            workflow.get_edge(edge_id)
                .map(|e| e.source == charlie && matches!(e.value, AccessControlEdge::CanAccess))
                .unwrap_or(false)
        })
        .count();

    assert_eq!(charlie_access, 1); // Can only access confidential

    // And: I can trace policy enforcement locations
    let scif_policies = workflow.edges().into_iter()
        .filter_map(|edge_id| {
            workflow.get_edge(edge_id).and_then(|edge| {
                if edge.source == scif && matches!(edge.value, AccessControlEdge::EnforcedAt) {
                    Some(edge.target)
                } else {
                    None
                }
            })
        })
        .count();

    assert_eq!(scif_policies, 1); // Need-to-know policy enforced at SCIF
}
