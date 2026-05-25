//! Bpmn adapter error surface for `xiuxian-qianji`.

use xiuxian_qianji_bpmn_engine::{BpmnEngineError, HostBridgeError};

/// Error returned by the `xiuxian-qianji` BPMN adapter layer.
#[derive(Debug, thiserror::Error, Clone)]
pub enum BpmnAdapterError {
    /// Returned when the BPMN engine rejects the current runtime state.
    #[error("BPMN engine error: {0}")]
    Engine(#[from] BpmnEngineError),
    /// Returned when the host bridge cannot service the current request.
    #[error("BPMN host bridge error: {0}")]
    Host(#[from] HostBridgeError),
}
