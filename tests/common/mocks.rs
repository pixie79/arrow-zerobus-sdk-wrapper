//! Mock infrastructure for Zerobus SDK and Stream
//!
//! This module provides mock implementations for testing without requiring
//! actual Zerobus SDK credentials.

use std::sync::{Arc, Mutex};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use prost_types::DescriptorProto;

/// Mock behavior configuration for tests
#[derive(Clone, Debug)]
pub enum MockBehavior {
    /// Stream works normally
    Success,
    /// Stream closes on first record
    CloseOnFirstRecord,
    /// Stream closes after N records
    CloseAfterNRecords(usize),
    /// Stream always closes
    AlwaysClose,
    /// Return error 6006
    Error6006,
    /// Return connection error
    ConnectionError(String),
}

/// Mock stream state
pub struct MockStreamState {
    pub records_sent: usize,
    pub behavior: MockBehavior,
    pub closed: bool,
}

impl MockStreamState {
    pub fn new(behavior: MockBehavior) -> Self {
        Self {
            records_sent: 0,
            behavior,
            closed: false,
        }
    }

    pub fn should_close(&self) -> bool {
        match &self.behavior {
            MockBehavior::Success => false,
            MockBehavior::CloseOnFirstRecord => self.records_sent == 0,
            MockBehavior::CloseAfterNRecords(n) => self.records_sent >= *n,
            MockBehavior::AlwaysClose => true,
            MockBehavior::Error6006 => false,
            MockBehavior::ConnectionError(_) => false,
        }
    }

    pub fn get_error(&self) -> Option<String> {
        if self.should_close() {
            Some("Stream is closed".to_string())
        } else {
            match &self.behavior {
                MockBehavior::Error6006 => Some("Error 6006: Pipeline blocked".to_string()),
                MockBehavior::ConnectionError(msg) => Some(msg.clone()),
                _ => None,
            }
        }
    }
}

/// Test helper to simulate stream closure scenarios
pub struct StreamClosureSimulator {
    state: Arc<Mutex<MockStreamState>>,
}

impl StreamClosureSimulator {
    pub fn new(behavior: MockBehavior) -> Self {
        Self {
            state: Arc::new(Mutex::new(MockStreamState::new(behavior))),
        }
    }

    pub fn simulate_ingest(&self, bytes: &[u8]) -> Result<MockIngestFuture, String> {
        let mut state = self.state.lock().unwrap();
        
        if state.closed {
            return Err("Stream is closed".to_string());
        }

        if let Some(error) = state.get_error() {
            state.closed = true;
            return Err(error);
        }

        state.records_sent += 1;

        // Check if we should close after this record
        if state.should_close() {
            state.closed = true;
            return Err("Stream is closed".to_string());
        }

        Ok(MockIngestFuture {
            bytes: bytes.to_vec(),
            state: self.state.clone(),
        })
    }

    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.records_sent = 0;
        state.closed = false;
    }

    pub fn get_records_sent(&self) -> usize {
        self.state.lock().unwrap().records_sent
    }
}

/// Mock future for ingest_record
pub struct MockIngestFuture {
    bytes: Vec<u8>,
    state: Arc<Mutex<MockStreamState>>,
}

impl Future for MockIngestFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Simulate async operation - immediately ready
        Poll::Ready(Ok(()))
    }
}

/// Test utilities for stream closure scenarios
pub mod test_utils {
    use super::*;

    /// Create a simulator that closes on first record
    pub fn create_close_on_first_simulator() -> StreamClosureSimulator {
        StreamClosureSimulator::new(MockBehavior::CloseOnFirstRecord)
    }

    /// Create a simulator that closes after N records
    pub fn create_close_after_n_simulator(n: usize) -> StreamClosureSimulator {
        StreamClosureSimulator::new(MockBehavior::CloseAfterNRecords(n))
    }

    /// Create a simulator that always closes
    pub fn create_always_close_simulator() -> StreamClosureSimulator {
        StreamClosureSimulator::new(MockBehavior::AlwaysClose)
    }

    /// Create a simulator that returns error 6006
    pub fn create_error_6006_simulator() -> StreamClosureSimulator {
        StreamClosureSimulator::new(MockBehavior::Error6006)
    }
}

