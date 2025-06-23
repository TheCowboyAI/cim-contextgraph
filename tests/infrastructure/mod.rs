//! Infrastructure tests for cim-contextgraph module
//!
//! These tests validate the event-driven architecture integration:
//! - NATS JetStream connectivity for graph events
//! - Context graph event publishing and consumption
//! - Event persistence and replay for graph operations
//! - CID chain integrity for graph mutations

mod event_driven_tests;
mod test_event_stream;
mod test_nats_connection;
mod test_message_routing;
