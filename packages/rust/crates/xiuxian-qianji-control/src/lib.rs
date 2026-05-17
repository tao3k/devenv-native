//! Workflow-neutral Qianji control-plane contracts.
//!
//! This crate owns run and step management state, not workflow semantics.
//! Workflow engines, BPMN adapters, and Agent workers should depend on this
//! crate when they need auditable lifecycle, lease, evidence, gate, and cost
//! tracking.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public control-plane API"),
        )
    }
);

#[cfg(feature = "duckdb")]
mod duckdb_ledger;
mod error;
mod event;
mod gate;
mod identity;
mod memory;
mod model;
mod traits;
#[cfg(feature = "valkey")]
mod valkey_hot_state;
mod view;

#[cfg(feature = "duckdb")]
pub use duckdb_ledger::DuckDbControlLedger;
pub use error::{ControlError, ControlResult};
pub use event::{ControlEvent, ControlEventKind, ControlEventRecord, RecoveryAttempt};
pub use gate::RequiredEvidenceGate;
pub use identity::{
    ArtifactId, ArtifactKind, EvidenceId, GateName, LeaseId, RunId, StepId, WorkerId,
};
pub use memory::{InMemoryControlLedger, InMemoryHotStateStore};
pub use model::{
    ArtifactRef, Budget, CostObservation, EvidenceRef, GateResult, RecoveryPolicy, RunStatus,
    RunnableStep, StepLease, StepStatus, WaitReason, WorkerHeartbeat, WorkerRef,
};
pub use traits::{ControlLedger, EvidenceGate, HotStateStore};
#[cfg(feature = "valkey")]
pub use valkey_hot_state::{ValkeyHotStateConfig, ValkeyHotStateStore, ValkeyKeyNamespace};
pub use view::{RunView, StepView, replay_run_view};
