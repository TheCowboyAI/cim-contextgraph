//! Infrastructure Layer 1.3: NATS Connection Tests for cim-contextgraph
//! 
//! User Story: As a context graph system, I need to publish and consume graph events via NATS
//!
//! Test Requirements:
//! - Verify NATS connection establishment
//! - Verify graph event stream creation and configuration
//! - Verify graph event publishing and consumption
//! - Verify reconnection handling for graph event streams
//!
//! Event Sequence:
//! 1. NATSConnected
//! 2. GraphStreamCreated { stream_name, subjects }
//! 3. GraphEventPublished { subject, event_id }
//! 4. GraphEventConsumed { subject, event_id }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Connect NATS]
//!     B --> C[NATSConnected]
//!     C --> D[Create Graph Stream]
//!     D --> E[GraphStreamCreated]
//!     E --> F[Publish Event]
//!     F --> G[GraphEventPublished]
//!     G --> H[Consume Event]
//!     H --> I[GraphEventConsumed]
//!     I --> J[Test Success]
//! ```

use std::time::{Duration, SystemTime};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

/// Context graph event for NATS testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextGraphEvent {
    pub event_id: String,
    pub graph_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: SystemTime,
}

/// NATS events for testing
#[derive(Debug, Clone, PartialEq)]
pub enum NATSEvent {
    NATSConnected,
    NATSDisconnected,
    GraphStreamCreated {
        stream_name: String,
        subjects: Vec<String>,
    },
    GraphEventPublished {
        subject: String,
        event_id: String,
    },
    GraphEventConsumed {
        subject: String,
        event_id: String,
    },
    ConsumerCreated {
        consumer_name: String,
        stream_name: String,
    },
    ReconnectionAttempt {
        attempt: u32,
    },
    ReconnectionSuccessful,
}

/// Mock NATS client for context graph testing
pub struct MockContextGraphNATSClient {
    connected: bool,
    streams: Vec<String>,
    published_events: Vec<(String, ContextGraphEvent)>,
    consumed_events: Arc<Mutex<Vec<ContextGraphEvent>>>,
}

impl MockContextGraphNATSClient {
    pub fn new() -> Self {
        Self {
            connected: false,
            streams: Vec::new(),
            published_events: Vec::new(),
            consumed_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        if self.connected {
            return Err("Already connected".to_string());
        }
        
        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        self.connected = true;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        
        self.connected = false;
        Ok(())
    }

    pub async fn create_graph_stream(
        &mut self,
        stream_name: String,
        subjects: Vec<String>,
    ) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }

        if self.streams.contains(&stream_name) {
            return Err("Stream already exists".to_string());
        }

        self.streams.push(stream_name);
        Ok(())
    }

    pub async fn publish_graph_event(
        &mut self,
        subject: String,
        event: ContextGraphEvent,
    ) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }

        self.published_events.push((subject, event));
        Ok(())
    }

    pub async fn create_consumer(
        &self,
        stream_name: &str,
        consumer_name: &str,
    ) -> Result<MockContextGraphConsumer, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }

        if !self.streams.contains(&stream_name.to_string()) {
            return Err("Stream not found".to_string());
        }

        Ok(MockContextGraphConsumer {
            name: consumer_name.to_string(),
            stream: stream_name.to_string(),
            consumed_events: Arc::clone(&self.consumed_events),
        })
    }

    pub async fn handle_reconnection(&mut self, max_attempts: u32) -> Result<u32, String> {
        let mut attempts = 0;
        
        while attempts < max_attempts && !self.connected {
            attempts += 1;
            
            // Simulate reconnection delay with exponential backoff
            let delay = Duration::from_millis(100 * 2u64.pow(attempts - 1));
            tokio::time::sleep(delay).await;
            
            // Simulate 50% success rate
            if attempts % 2 == 0 {
                self.connected = true;
                return Ok(attempts);
            }
        }
        
        if self.connected {
            Ok(attempts)
        } else {
            Err("Max reconnection attempts reached".to_string())
        }
    }
}

/// Mock consumer for context graph events
pub struct MockContextGraphConsumer {
    pub name: String,
    pub stream: String,
    consumed_events: Arc<Mutex<Vec<ContextGraphEvent>>>,
}

impl MockContextGraphConsumer {
    pub async fn consume_event(&self, event: ContextGraphEvent) -> Result<(), String> {
        let mut events = self.consumed_events.lock().await;
        events.push(event);
        Ok(())
    }

    pub async fn get_consumed_count(&self) -> usize {
        self.consumed_events.lock().await.len()
    }
}

/// NATS event validator for testing
pub struct NATSEventValidator {
    expected_events: Vec<NATSEvent>,
    captured_events: Vec<NATSEvent>,
}

impl NATSEventValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
        }
    }

    pub fn expect_sequence(mut self, events: Vec<NATSEvent>) -> Self {
        self.expected_events = events;
        self
    }

    pub fn capture_event(&mut self, event: NATSEvent) {
        self.captured_events.push(event);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.captured_events.len() != self.expected_events.len() {
            return Err(format!(
                "Event count mismatch: expected {}, got {}",
                self.expected_events.len(),
                self.captured_events.len()
            ));
        }

        for (i, (expected, actual)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected != actual {
                return Err(format!(
                    "Event mismatch at position {}: expected {:?}, got {:?}",
                    i, expected, actual
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_graph_nats_connection() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        let mut validator = NATSEventValidator::new()
            .expect_sequence(vec![
                NATSEvent::NATSConnected,
            ]);

        // Act
        client.connect().await.unwrap();
        validator.capture_event(NATSEvent::NATSConnected);

        // Assert
        assert!(client.connected);
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_graph_event_stream_creation() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        let mut validator = NATSEventValidator::new();

        // Connect first
        client.connect().await.unwrap();

        // Act
        let stream_name = "CONTEXT_GRAPH_EVENTS".to_string();
        let subjects = vec![
            "graph.created".to_string(),
            "graph.node.added".to_string(),
            "graph.edge.added".to_string(),
            "graph.context.updated".to_string(),
        ];

        client.create_graph_stream(stream_name.clone(), subjects.clone()).await.unwrap();

        // Assert
        assert!(client.streams.contains(&stream_name));
        
        validator.capture_event(NATSEvent::GraphStreamCreated {
            stream_name,
            subjects,
        });
    }

    #[tokio::test]
    async fn test_context_graph_event_publishing() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        let mut validator = NATSEventValidator::new();

        client.connect().await.unwrap();
        client.create_graph_stream(
            "CONTEXT_GRAPH_EVENTS".to_string(),
            vec!["graph.*".to_string()],
        ).await.unwrap();

        // Act
        let event = ContextGraphEvent {
            event_id: "evt-001".to_string(),
            graph_id: "context-graph-1".to_string(),
            event_type: "GraphCreated".to_string(),
            payload: serde_json::json!({
                "name": "Test Context Graph",
                "context_type": "Semantic"
            }),
            timestamp: SystemTime::now(),
        };

        let subject = "graph.created".to_string();
        client.publish_graph_event(subject.clone(), event.clone()).await.unwrap();

        // Assert
        assert_eq!(client.published_events.len(), 1);
        assert_eq!(client.published_events[0].0, subject);
        assert_eq!(client.published_events[0].1, event);

        validator.capture_event(NATSEvent::GraphEventPublished {
            subject,
            event_id: event.event_id,
        });
    }

    #[tokio::test]
    async fn test_context_graph_event_consumption() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        let mut validator = NATSEventValidator::new();

        client.connect().await.unwrap();
        client.create_graph_stream(
            "CONTEXT_GRAPH_EVENTS".to_string(),
            vec!["graph.*".to_string()],
        ).await.unwrap();

        // Create consumer
        let consumer = client.create_consumer(
            "CONTEXT_GRAPH_EVENTS",
            "graph-processor"
        ).await.unwrap();

        // Act
        let event = ContextGraphEvent {
            event_id: "evt-002".to_string(),
            graph_id: "context-graph-2".to_string(),
            event_type: "NodeAdded".to_string(),
            payload: serde_json::json!({
                "node_id": "node-1",
                "node_type": "Concept",
                "position": [0.0, 0.0, 0.0]
            }),
            timestamp: SystemTime::now(),
        };

        consumer.consume_event(event.clone()).await.unwrap();

        // Assert
        assert_eq!(consumer.get_consumed_count().await, 1);

        validator.capture_event(NATSEvent::GraphEventConsumed {
            subject: "graph.node.added".to_string(),
            event_id: event.event_id,
        });
    }

    #[tokio::test]
    async fn test_multiple_graph_streams() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        client.connect().await.unwrap();

        // Act - Create multiple streams for different graph types
        let streams = vec![
            ("SEMANTIC_GRAPHS", vec!["semantic.graph.*"]),
            ("SPATIAL_GRAPHS", vec!["spatial.graph.*"]),
            ("TEMPORAL_GRAPHS", vec!["temporal.graph.*"]),
        ];

        for (stream_name, subjects) in &streams {
            client.create_graph_stream(
                stream_name.to_string(),
                subjects.iter().map(|s| s.to_string()).collect(),
            ).await.unwrap();
        }

        // Assert
        assert_eq!(client.streams.len(), 3);
        for (stream_name, _) in &streams {
            assert!(client.streams.contains(&stream_name.to_string()));
        }
    }

    #[tokio::test]
    async fn test_graph_event_subject_filtering() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        client.connect().await.unwrap();

        // Act - Publish events to different subjects
        let events = vec![
            ("graph.created", "evt-1", "GraphCreated"),
            ("graph.node.added", "evt-2", "NodeAdded"),
            ("graph.edge.added", "evt-3", "EdgeAdded"),
            ("graph.context.updated", "evt-4", "ContextUpdated"),
        ];

        for (subject, event_id, event_type) in events {
            let event = ContextGraphEvent {
                event_id: event_id.to_string(),
                graph_id: "test-graph".to_string(),
                event_type: event_type.to_string(),
                payload: serde_json::json!({}),
                timestamp: SystemTime::now(),
            };

            client.publish_graph_event(subject.to_string(), event).await.unwrap();
        }

        // Assert
        assert_eq!(client.published_events.len(), 4);
        
        // Verify subject-based filtering
        let node_events: Vec<_> = client.published_events.iter()
            .filter(|(subject, _)| subject.contains("node"))
            .collect();
        assert_eq!(node_events.len(), 1);
    }

    #[tokio::test]
    async fn test_disconnection_handling() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        let mut validator = NATSEventValidator::new();

        // Connect and create resources
        client.connect().await.unwrap();
        client.create_graph_stream(
            "TEST_STREAM".to_string(),
            vec!["test.*".to_string()],
        ).await.unwrap();

        // Act - Disconnect
        client.disconnect().await.unwrap();
        validator.capture_event(NATSEvent::NATSDisconnected);

        // Try to publish (should fail)
        let event = ContextGraphEvent {
            event_id: "evt-fail".to_string(),
            graph_id: "test".to_string(),
            event_type: "Test".to_string(),
            payload: serde_json::json!({}),
            timestamp: SystemTime::now(),
        };

        let result = client.publish_graph_event("test.event".to_string(), event).await;

        // Assert
        assert!(!client.connected);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected");
    }

    #[tokio::test]
    async fn test_reconnection_with_backoff() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        let mut validator = NATSEventValidator::new();

        // Disconnect to simulate connection loss
        client.connected = false;

        // Act - Attempt reconnection
        let attempts = client.handle_reconnection(5).await.unwrap();

        // Assert
        assert!(client.connected);
        assert!(attempts > 0);
        assert!(attempts <= 5);

        validator.capture_event(NATSEvent::ReconnectionAttempt { attempt: attempts });
        validator.capture_event(NATSEvent::ReconnectionSuccessful);
    }

    #[tokio::test]
    async fn test_consumer_creation() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        let mut validator = NATSEventValidator::new();

        client.connect().await.unwrap();
        let stream_name = "GRAPH_EVENTS";
        client.create_graph_stream(
            stream_name.to_string(),
            vec!["graph.*".to_string()],
        ).await.unwrap();

        // Act
        let consumer_name = "graph-processor";
        let consumer = client.create_consumer(stream_name, consumer_name).await.unwrap();

        // Assert
        assert_eq!(consumer.name, consumer_name);
        assert_eq!(consumer.stream, stream_name);

        validator.capture_event(NATSEvent::ConsumerCreated {
            consumer_name: consumer_name.to_string(),
            stream_name: stream_name.to_string(),
        });
    }

    #[tokio::test]
    async fn test_concurrent_event_consumption() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        client.connect().await.unwrap();
        client.create_graph_stream(
            "CONCURRENT_STREAM".to_string(),
            vec!["concurrent.*".to_string()],
        ).await.unwrap();

        let consumer = client.create_consumer(
            "CONCURRENT_STREAM",
            "concurrent-consumer"
        ).await.unwrap();

        // Act - Consume multiple events concurrently
        let mut handles = vec![];
        
        for i in 0..5 {
            let consumer_clone = MockContextGraphConsumer {
                name: consumer.name.clone(),
                stream: consumer.stream.clone(),
                consumed_events: Arc::clone(&consumer.consumed_events),
            };

            let handle = tokio::spawn(async move {
                let event = ContextGraphEvent {
                    event_id: format!("concurrent-{}", i),
                    graph_id: "test-graph".to_string(),
                    event_type: "Test".to_string(),
                    payload: serde_json::json!({ "index": i }),
                    timestamp: SystemTime::now(),
                };

                consumer_clone.consume_event(event).await.unwrap();
            });

            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Assert
        assert_eq!(consumer.get_consumed_count().await, 5);
    }

    #[tokio::test]
    async fn test_graph_merge_event_flow() {
        // Arrange
        let mut client = MockContextGraphNATSClient::new();
        client.connect().await.unwrap();
        client.create_graph_stream(
            "MERGE_EVENTS".to_string(),
            vec!["graph.merge.*".to_string()],
        ).await.unwrap();

        // Act - Publish graph merge event
        let merge_event = ContextGraphEvent {
            event_id: "merge-001".to_string(),
            graph_id: "merged-graph".to_string(),
            event_type: "GraphMerged".to_string(),
            payload: serde_json::json!({
                "source_graph_id": "graph-a",
                "target_graph_id": "graph-b",
                "merge_strategy": "Union",
                "node_count": 15,
                "edge_count": 23
            }),
            timestamp: SystemTime::now(),
        };

        client.publish_graph_event(
            "graph.merge.completed".to_string(),
            merge_event.clone()
        ).await.unwrap();

        // Assert
        assert_eq!(client.published_events.len(), 1);
        let (subject, event) = &client.published_events[0];
        assert_eq!(subject, "graph.merge.completed");
        assert_eq!(event.event_type, "GraphMerged");
    }
} 