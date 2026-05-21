//! Read-only projections from attachment-owned facts into polyglot contracts.

#[cfg(feature = "pdf-source-range")]
use xiuxian_polyglot_orchestrator::{
    DoclingSchedulePlan, DoclingSchedulingInput, DoclingWorkerPolicy, PolyglotControlSnapshot,
    RouteProfileRef, SnapshotInvariantError, WorkerPressureEvidence,
};

#[cfg(feature = "pdf-source-range")]
use crate::pdf::ocr::{
    PDF_OCR_DEFAULT_PROFILE, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
};

/// Returns a typed reference to the attachment-owned PDF OCR shard input contract.
///
/// The route must be supplied by the transport owner. This helper only attaches
/// the profile and schema facts owned by `xiuxian-wendao-attachments`.
#[cfg(feature = "pdf-source-range")]
#[must_use]
pub fn pdf_ocr_shard_input_ref(route: impl Into<String>) -> RouteProfileRef {
    let mut reference = RouteProfileRef::ocr_shards(route, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION);
    reference.profile = Some(PDF_OCR_DEFAULT_PROFILE.to_string());
    reference
}

/// Returns a typed reference to the attachment-owned PDF OCR shard result contract.
///
/// The route must be supplied by the transport owner. This helper only attaches
/// the profile and schema facts owned by `xiuxian-wendao-attachments`.
#[cfg(feature = "pdf-source-range")]
#[must_use]
pub fn pdf_ocr_shard_result_ref(route: impl Into<String>) -> RouteProfileRef {
    let mut reference = RouteProfileRef::ocr_shards(route, PDF_OCR_SHARD_RESULT_SCHEMA_VERSION);
    reference.profile = Some(PDF_OCR_DEFAULT_PROFILE.to_string());
    reference
}

/// Builds a read-only snapshot for attachment-owned PDF OCR shard contracts.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
#[cfg(feature = "pdf-source-range")]
pub fn pdf_ocr_shard_contract_snapshot(
    route: impl Into<String>,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    let route = route.into();
    PolyglotControlSnapshot::from_parts(
        vec![
            pdf_ocr_shard_input_ref(route.clone()),
            pdf_ocr_shard_result_ref(route),
        ],
        Vec::new(),
        Vec::new(),
    )
}

/// Positional boundary for OCR shard pressure counters.
///
/// The helper mirrors the orchestrator evidence constructor and keeps caller
/// sites explicit about each counter until the transport contract is promoted
/// to a shared request type.
#[cfg(feature = "pdf-source-range")]
#[must_use]
pub fn pdf_ocr_shard_pressure_evidence(
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queued_items: u32,
    failed_items: u32,
    retryable_failures: u32,
    ordering_backlog: u32,
    fallback_available: bool,
) -> WorkerPressureEvidence {
    WorkerPressureEvidence::ocr_shard_extraction()
        .with_worker_budget(max_in_flight, active_in_flight)
        .with_queue_depth(queued_items)
        .with_failures(failed_items, retryable_failures)
        .with_ordering_backlog(ordering_backlog)
        .with_fallback_available(fallback_available)
}

/// Builds a read-only OCR shard pressure snapshot from supplied counters.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
#[cfg(feature = "pdf-source-range")]
pub fn pdf_ocr_shard_pressure_snapshot(
    route: impl Into<String>,
    pressure: WorkerPressureEvidence,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    let route = route.into();
    PolyglotControlSnapshot::from_parts(
        vec![
            pdf_ocr_shard_input_ref(route.clone()),
            pdf_ocr_shard_result_ref(route),
        ],
        vec![pressure.to_admission_budget()],
        vec![pressure.to_lane_evidence()],
    )
}

/// Returns an inert OCR shard scheduling plan from attachment-owned pressure
/// facts and caller-local worker bounds.
#[cfg(feature = "pdf-source-range")]
#[must_use]
pub fn pdf_ocr_shard_schedule_plan(
    pressure: WorkerPressureEvidence,
    requested_workers: Option<u32>,
    max_worker_cap: Option<u32>,
    shard_count: u32,
) -> DoclingSchedulePlan {
    DoclingSchedulingInput::ocr_shards(pressure)
        .with_worker_request(requested_workers, max_worker_cap)
        .with_shard_count(shard_count)
        .plan()
}

/// Returns an inert source-PDF page-range OCR scheduling plan.
///
/// The optional `diagnostic_worker_override` is for benchmark sweeps only.
/// When it is `None`, the orchestrator crate computes the worker request from
/// the owner-supplied adaptive budget, maximum worker cap, and shard count.
///
/// Positional boundary: the arguments mirror the orchestrator scheduling
/// constructor while this crate remains a read-only owner bridge.
#[cfg(feature = "pdf-source-range")]
#[must_use]
pub fn pdf_ocr_source_range_shard_schedule_plan(
    pressure: WorkerPressureEvidence,
    adaptive_worker_budget: Option<u32>,
    diagnostic_worker_override: Option<u32>,
    max_worker_cap: Option<u32>,
    shard_count: u32,
) -> DoclingSchedulePlan {
    DoclingSchedulingInput::ocr_shards(pressure)
        .with_worker_policy(DoclingWorkerPolicy::SourcePdfPageRange)
        .with_adaptive_worker_budget(adaptive_worker_budget)
        .with_worker_request(diagnostic_worker_override, max_worker_cap)
        .with_shard_count(shard_count)
        .plan()
}

#[cfg(all(test, feature = "pdf-source-range"))]
#[path = "../tests/unit/polyglot.rs"]
mod tests;
