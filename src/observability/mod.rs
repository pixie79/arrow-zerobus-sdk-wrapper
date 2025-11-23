//! OpenTelemetry observability integration
//!
//! This module integrates with otlp-rust-service for metrics and traces.

pub mod otlp;

pub use otlp::ObservabilityManager;
