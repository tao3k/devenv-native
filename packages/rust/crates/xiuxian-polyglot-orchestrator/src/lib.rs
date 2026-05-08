//! Thin polyglot control-plane contracts for Wendao compute lanes.
//!
//! This crate names shared contracts only. Execution ownership stays in the
//! existing runtime, attachments, analyzer, and Julia packages.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config()
            .with_verification_profile_hint(
                rust_lang_project_harness::RustVerificationProfileHint::new(
                    "src/lib.rs",
                    [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
                )
                .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
                .with_rationale(
                    "crate root owns the shared polyglot control-plane contract exports",
                ),
            )
            .with_verification_profile_hint(
                rust_lang_project_harness::RustVerificationProfileHint::new(
                    "src/docling_schedule/model.rs",
                    [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
                )
                .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
                .with_rationale(
                    "Docling scheduling plans are the reusable policy contract for owner crates",
                ),
            )
            .with_verification_profile_hint(
                rust_lang_project_harness::RustVerificationProfileHint::new(
                    "src/julia_schedule/model.rs",
                    [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
                )
                .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
                .with_rationale(
                    "Julia scheduling plans are the reusable profile-aware policy contract for owner crates",
                ),
            )
    }
);

#[cfg(test)]
#[path = "../tests/unit/lib/mod.rs"]
mod tests;

/// Admission budget and decision contracts.
pub mod admission;
/// Pure scheduling contracts for Python Docling work.
pub mod docling_schedule;
/// Health, readiness, pressure, and fallback evidence contracts.
pub mod evidence;
/// Pure scheduling contracts for Julia compute profiles.
pub mod julia_schedule;
/// Lane identity and capability classification.
pub mod lanes;
/// Worker pressure evidence contracts.
pub mod pressure;
/// Julia readiness evidence contracts.
pub mod readiness;
/// Typed references to external owner contracts.
pub mod refs;
/// Schema benchmark evidence contracts.
pub mod schema_benchmark;
/// Read-only control-plane snapshots.
pub mod snapshot;

pub use admission::{AdmissionBudget, AdmissionDecision, QueueReason, RejectionReason};
pub use docling_schedule::{
    DoclingScheduleAction, DoclingSchedulePlan, DoclingScheduleReason, DoclingSchedulingInput,
    DoclingWorkerPolicy,
};
pub use evidence::{FallbackEvidence, HealthState, LaneEvidence, PressureLevel, ReadinessState};
pub use julia_schedule::{
    JuliaComputeTaskShape, JuliaRuntimeStats, JuliaScheduleAction, JuliaSchedulePlan,
    JuliaScheduleReason, JuliaSchedulingInput, JuliaTaskComplexityClass,
};
pub use lanes::{LaneCapability, PolyglotLane};
pub use pressure::WorkerPressureEvidence;
pub use readiness::{
    BenchmarkState, ContractValidationState, JuliaAcceleratorDiagnostics, JuliaReadinessEvidence,
    JuliaThreadPinningDiagnostics, JuliaThreadPinningState, JuliaThreadTopology,
    ManifestReadinessState, WarmupState,
};
pub use refs::{ContractOwner, RouteProfileRef};
pub use schema_benchmark::{
    SchemaBenchmarkCase, SchemaBenchmarkEvidence, SchemaBenchmarkReport,
    SchemaBenchmarkReportError, SchemaStrategyCandidate, SchemaStrategyPreference,
};
pub use snapshot::{PolyglotControlSnapshot, SnapshotInvariantError};
