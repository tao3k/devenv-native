use crate::bpmn::BpmnAdapterError;
#[cfg(feature = "duckdb")]
use crate::bpmn::data_store::QianjiBpmnDataStoreError;
use qianji_bpmn_engine::BpmnEngineError;
use std::io;
use std::path::PathBuf;

/// Error returned by the host-owned BPMN orchestration facade.
#[derive(Debug, thiserror::Error)]
pub enum BpmnOrchestrationError {
    /// Returned when the BPMN engine rejects bundle, checkpoint, or runtime state.
    #[error("BPMN engine error: {0}")]
    Engine(#[from] BpmnEngineError),
    /// Returned when local `DuckDB` workflow-state storage fails.
    #[cfg(feature = "duckdb")]
    #[error("BPMN DuckDB workflow-state error: {0}")]
    DuckDbWorkflowState(#[from] QianjiBpmnDataStoreError),
    /// Returned when the xiuxian BPMN adapter cannot complete host work.
    #[error("BPMN adapter error: {0}")]
    Adapter(#[from] BpmnAdapterError),
    /// Returned when one BPMN source file cannot be read from disk.
    #[error("Failed to read BPMN source file '{path}': {source}")]
    ReadBpmnSource {
        /// BPMN source path that could not be read.
        path: PathBuf,
        /// Filesystem read error returned by the host.
        #[source]
        source: io::Error,
    },
    /// Returned when one DMN source file cannot be read from disk.
    #[error("Failed to read DMN source file '{path}': {source}")]
    ReadDmnSource {
        /// DMN source path that could not be read.
        path: PathBuf,
        /// Filesystem read error returned by the host.
        #[source]
        source: io::Error,
    },
    /// Returned when a fresh BPMN run is requested without initial variables
    /// and no checkpoint can be resumed.
    #[error(
        "BPMN process '{process_id}' instance '{instance_id}' requires initial variables for a fresh run because no resumable checkpoint was found"
    )]
    FreshContextRequired {
        /// Process id for the requested BPMN run.
        process_id: String,
        /// Instance id for the requested BPMN run.
        instance_id: String,
    },
    /// Returned when a checkpoint references a BPMN process that is missing
    /// from the loaded package.
    #[error(
        "Checkpoint for process '{process_id}' cannot be resumed because the loaded BPMN package does not contain that process"
    )]
    CheckpointProcessMissing {
        /// Process id referenced by the stored checkpoint.
        process_id: String,
    },
    /// Returned when a checkpoint process identity drifts from the loaded package.
    #[error(
        "Checkpoint process '{process_id}' does not match the loaded BPMN package identity: checkpoint package '{checkpoint_package_id}' digest '{checkpoint_spec_digest}', loaded package '{loaded_package_id}' digest '{loaded_spec_digest}'"
    )]
    CheckpointProcessIdentityDrift {
        /// Process id referenced by the stored checkpoint.
        process_id: String,
        /// Package id stored in the checkpoint envelope.
        checkpoint_package_id: String,
        /// Spec digest stored in the checkpoint envelope.
        checkpoint_spec_digest: String,
        /// Package id resolved from the currently loaded BPMN package.
        loaded_package_id: String,
        /// Spec digest resolved from the currently loaded BPMN package.
        loaded_spec_digest: String,
    },
    /// Returned when one BPMN scheduler lease is requested without a
    /// Valkey-backed checkpoint backend.
    #[error(
        "BPMN checkpoint lease ownership requires a Valkey-backed checkpoint store, got backend '{backend}'"
    )]
    CheckpointLeaseUnsupportedBackend {
        /// Human-readable backend name.
        backend: String,
    },
    /// Returned when another scheduler owner already holds the BPMN checkpoint
    /// lease for the same instance.
    #[error(
        "BPMN checkpoint lease for instance '{instance_id}' is already held by another owner; '{owner_token}' could not acquire it"
    )]
    CheckpointLeaseConflict {
        /// Workflow instance identifier.
        instance_id: String,
        /// Owner token that lost the lease race.
        owner_token: String,
    },
    /// Returned when BPMN lease ownership is requested from one scheduler
    /// identity that does not expose a stable agent id.
    #[error(
        "BPMN checkpoint lease ownership requires SchedulerAgentIdentity.agent_id; role-only or empty identities are not stable single-writer owners"
    )]
    CheckpointLeaseAgentIdRequired,
}
