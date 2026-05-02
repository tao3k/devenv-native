use serde_json::json;

use crate::agent::{DownstreamAdmissionRuntimeSnapshot, MemoryRuntimeStatusSnapshot};

use super::readiness::is_backend_ready;

pub(in crate::channels::discord::runtime::managed::replies::memory) fn format_memory_runtime_status_json(
    status: &MemoryRuntimeStatusSnapshot,
) -> serde_json::Value {
    let backend_ready = is_backend_ready(
        status.enabled,
        status.active_backend.is_some(),
        status.startup_load_status,
    );
    json!({
        "memory_enabled": status.enabled,
        "configured_backend": status.configured_backend,
        "active_backend": status.active_backend,
        "strict_startup": status.strict_startup,
        "startup_load_status": status.startup_load_status,
        "backend_ready": backend_ready,
        "store_path": status.store_path,
        "table_name": status.table_name,
        "gate_promote_threshold": status.gate_promote_threshold,
        "gate_obsolete_threshold": status.gate_obsolete_threshold,
        "gate_promote_min_usage": status.gate_promote_min_usage,
        "gate_obsolete_min_usage": status.gate_obsolete_min_usage,
        "gate_promote_failure_rate_ceiling": status.gate_promote_failure_rate_ceiling,
        "gate_obsolete_failure_rate_floor": status.gate_obsolete_failure_rate_floor,
        "gate_promote_min_ttl_score": status.gate_promote_min_ttl_score,
        "gate_obsolete_max_ttl_score": status.gate_obsolete_max_ttl_score,
        "episodes_total": status.episodes_total,
        "q_values_total": status.q_values_total,
    })
}

pub(in crate::channels::discord::runtime::managed::replies::memory) fn format_downstream_admission_status_json(
    status: &DownstreamAdmissionRuntimeSnapshot,
) -> serde_json::Value {
    json!({
        "enabled": status.enabled,
        "llm_reject_threshold_pct": status.llm_reject_threshold_pct,
        "embedding_reject_threshold_pct": status.embedding_reject_threshold_pct,
        "metrics": {
            "total": status.metrics.total,
            "admitted": status.metrics.admitted,
            "rejected": status.metrics.rejected,
            "rejected_llm_saturated": status.metrics.rejected_llm_saturated,
            "rejected_embedding_saturated": status.metrics.rejected_embedding_saturated,
            "reject_rate_pct": status.metrics.reject_rate_pct,
        },
    })
}
