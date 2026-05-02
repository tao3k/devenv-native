//! Transmuter module: content washing and streaming event parsing.
//!
//! This module provides:
//! - `ZhenfaTransmuter`: Lightweight content washing for Spider ingress
//! - `streaming`: Unified streaming parser for multi-agent CLI outputs

mod washing;

/// Streaming parser and pipeline contracts for CLI agent output.
pub mod streaming;

pub use washing::{ZhenfaResolveAndWashError, ZhenfaTransmuter, ZhenfaTransmuterError};
