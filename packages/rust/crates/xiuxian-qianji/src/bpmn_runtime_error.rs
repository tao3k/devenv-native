use crate::bpmn::BpmnAdapterError;
use qianji_bpmn_engine::{BpmnEngineError, BpmnPendingHostWorkIdentityMismatch};
use std::io;
use std::path::PathBuf;
#[cfg(feature = "duckdb")]
use xiuxian_db_store::qianji_bpmn::QianjiBpmnDataStoreError;

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
    /// Returned when a start-at run would overwrite an existing checkpoint.
    #[error(
        "BPMN start-at requires a fresh instance id; checkpoint already exists for instance '{instance_id}'"
    )]
    StartAtCheckpointExists {
        /// Workflow instance identifier that already exists.
        instance_id: String,
    },
    /// Returned when a start-at run targets a node that is not in the process.
    #[error("BPMN start-at target node '{node_id}' was not found in process '{process_id}'")]
    StartAtNodeMissing {
        /// Process id for the requested BPMN run.
        process_id: String,
        /// Requested BPMN node id.
        node_id: String,
    },
    /// Returned when a start-at run targets an unsupported node kind.
    #[error(
        "BPMN start-at target node '{node_id}' in process '{process_id}' has unsupported kind '{node_kind}'"
    )]
    StartAtNodeUnsupported {
        /// Process id for the requested BPMN run.
        process_id: String,
        /// Requested BPMN node id.
        node_id: String,
        /// Human-readable BPMN node kind.
        node_kind: String,
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
    /// Returned when an explicit task-completion request targets a pending
    /// host-work item whose BPMN identity does not match the checkpointed
    /// work.
    #[error("{0}")]
    PendingHostWorkIdentityMismatch(Box<BpmnPendingHostWorkIdentityMismatch>),
    /// Returned when checkpointed pending host work is already claimed and a
    /// completion request does not supply the same claimant.
    #[error(
        "pending host work for instance '{instance_id}' token {token_id} is claimed by '{claimed_by}'; completion must include the matching claimant"
    )]
    PendingHostWorkClaimRequired {
        /// Workflow instance identifier.
        instance_id: String,
        /// Runtime token identifier for the pending host work.
        token_id: u64,
        /// Checkpointed claimant that owns the pending human work.
        claimed_by: String,
    },
    /// Returned when checkpointed pending host work is claimed by one
    /// claimant, but a different claimant attempts completion.
    #[error(
        "pending host work claimant mismatch for instance '{instance_id}' token {token_id}: expected claimant '{expected_claimant}', got '{actual_claimant}'"
    )]
    PendingHostWorkClaimantMismatch {
        /// Workflow instance identifier.
        instance_id: String,
        /// Runtime token identifier for the pending host work.
        token_id: u64,
        /// Checkpointed claimant that owns the pending human work.
        expected_claimant: String,
        /// Claimant supplied by the completion request.
        actual_claimant: String,
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
    /// Returned when checkpoint instance listing is requested from a backend
    /// that does not expose an enumerable local index.
    #[error("BPMN checkpoint instance listing is not supported by backend '{backend}'")]
    CheckpointListUnsupportedBackend {
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

impl BpmnOrchestrationError {
    pub(crate) fn pending_host_work_identity_mismatch(
        instance_id: String,
        token_id: u64,
        expected_process_id: String,
        expected_activity_id: String,
        actual_process_id: String,
        actual_activity_id: String,
    ) -> Self {
        Self::PendingHostWorkIdentityMismatch(Box::new(BpmnPendingHostWorkIdentityMismatch {
            instance: instance_id,
            token: token_id,
            expected_process: expected_process_id,
            expected_activity: expected_activity_id,
            actual_process: actual_process_id,
            actual_activity: actual_activity_id,
        }))
    }
}
