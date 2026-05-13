//! Formal logic and safety auditing.

use crate::QianjiError;
use crate::engine::QianjiEngine;
use petgraph::algo::is_cyclic_directed;

/// Lightweight formal-logic helpers for Synapse-Audit traces.
#[path = "safety_logic.rs"]
pub mod logic;

/// Guard responsible for static and structural safety of the Qianji Box.
pub struct QianjiSafetyGuard {
    /// Maximum allowed loop iterations before forced termination.
    pub max_loop_iterations: u32,
}

impl QianjiSafetyGuard {
    /// Creates a new guard.
    #[must_use]
    pub fn new(max_loop_iterations: u32) -> Self {
        Self {
            max_loop_iterations,
        }
    }

    /// Performs a static analysis of the graph topology.
    /// Checks for unauthorized cycles that could lead to infinite execution.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when the topology contains cycles without an exit.
    #[allow(clippy::unused_self)]
    pub fn audit_topology(&self, engine: &QianjiEngine) -> Result<(), QianjiError> {
        if is_cyclic_directed(&engine.graph) {
            return Err(QianjiError::Topology(
                "Infinite cycle detected without exit condition".to_string(),
            ));
        }
        Ok(())
    }
}
