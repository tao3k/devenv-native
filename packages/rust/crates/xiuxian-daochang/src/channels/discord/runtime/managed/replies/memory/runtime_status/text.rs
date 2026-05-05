use crate::agent::{DownstreamAdmissionRuntimeSnapshot, MemoryRuntimeStatusSnapshot};

use super::readiness::{
    format_optional_bool, format_optional_str, format_optional_string, is_backend_ready,
};

pub(in crate::channels::discord::runtime::managed::replies::memory) fn format_memory_runtime_status_lines(
    status: MemoryRuntimeStatusSnapshot,
) -> Vec<String> {
    let backend_ready = is_backend_ready(
        status.enabled,
        status.active_backend.is_some(),
        status.startup_load_status,
    );
    vec![
        format!("- `memory_enabled={}`", status.enabled),
        format!(
            "- `configured_backend={}` / `active_backend={}`",
            format_optional_string(status.configured_backend),
            format_optional_str(status.active_backend)
        ),
        format!(
            "- `strict_startup={}` / `startup_load_status={}` / `backend_ready={}`",
            format_optional_bool(status.strict_startup),
            status.startup_load_status,
            backend_ready
        ),
        format!(
            "- `store_path={}` / `table_name={}`",
            format_optional_string(status.store_path),
            format_optional_string(status.table_name)
        ),
        format!(
            "- `episodes_total={}` / `q_values_total={}`",
            status
                .episodes_total
                .map_or_else(|| "-".to_string(), |value| value.to_string()),
            status
                .q_values_total
                .map_or_else(|| "-".to_string(), |value| value.to_string())
        ),
    ]
}

pub(in crate::channels::discord::runtime::managed::replies::memory) fn format_downstream_admission_status_lines(
    status: DownstreamAdmissionRuntimeSnapshot,
) -> Vec<String> {
    vec![
        format!("- `enabled={}`", status.enabled),
        format!(
            "- `llm_reject_threshold_pct={}` / `embedding_reject_threshold_pct={}`",
            status.llm_reject_threshold_pct, status.embedding_reject_threshold_pct
        ),
        format!(
            "- `total={}` / `admitted={}` / `rejected={}` / `reject_rate_pct={}`",
            status.metrics.total,
            status.metrics.admitted,
            status.metrics.rejected,
            status.metrics.reject_rate_pct
        ),
        format!(
            "- `rejected_llm_saturated={}` / `rejected_embedding_saturated={}`",
            status.metrics.rejected_llm_saturated, status.metrics.rejected_embedding_saturated
        ),
    ]
}
