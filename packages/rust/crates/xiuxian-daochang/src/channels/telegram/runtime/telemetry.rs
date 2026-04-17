use std::time::Duration;

use tokio::sync::mpsc;

use crate::agent::DownstreamAdmissionRuntimeSnapshot;
use crate::channels::runtime_snapshot::resolve_runtime_snapshot_interval_secs;
use crate::channels::telegram::runtime_config::TelegramRuntimeConfig;
use crate::channels::traits::ChannelMessage;

const RUNTIME_SNAPSHOT_INTERVAL_ENV: &str =
    "XIUXIAN_DAOCHANG_TELEGRAM_RUNTIME_SNAPSHOT_INTERVAL_SECS";
const DEFAULT_RUNTIME_SNAPSHOT_INTERVAL_SECS: u64 = 30;

pub(in crate::channels::telegram::runtime) fn snapshot_interval_from_env() -> Option<Duration> {
    resolve_snapshot_interval_secs(|name| std::env::var(name).ok()).map(Duration::from_secs)
}

pub(in crate::channels::telegram::runtime) fn emit_runtime_snapshot(
    mode: &str,
    inbound_tx: &mpsc::Sender<ChannelMessage>,
    foreground_tx: &mpsc::Sender<ChannelMessage>,
    runtime_config: TelegramRuntimeConfig,
    admission: DownstreamAdmissionRuntimeSnapshot,
) {
    let inbound_queue_capacity = runtime_config.inbound_queue_capacity;
    let inbound_queue_available = inbound_tx.capacity();
    let inbound_queue_depth = inbound_queue_capacity.saturating_sub(inbound_queue_available);

    let foreground_queue_capacity = runtime_config.foreground_queue_capacity;
    let foreground_queue_available = foreground_tx.capacity();
    let foreground_queue_depth =
        foreground_queue_capacity.saturating_sub(foreground_queue_available);

    tracing::info!(
        event = "telegram.runtime.snapshot",
        mode,
        inbound_queue_capacity,
        inbound_queue_depth,
        inbound_queue_available,
        foreground_queue_capacity,
        foreground_queue_depth,
        foreground_queue_available,
        foreground_queue_mode = runtime_config.foreground_queue_mode.as_str(),
        foreground_max_in_flight = runtime_config.foreground_max_in_flight_messages,
        foreground_turn_timeout_secs = runtime_config.foreground_turn_timeout_secs,
        admission_enabled = admission.enabled,
        admission_llm_reject_threshold_pct = admission.llm_reject_threshold_pct,
        admission_embedding_reject_threshold_pct = admission.embedding_reject_threshold_pct,
        admission_total_evaluations = admission.metrics.total,
        admission_admitted_total = admission.metrics.admitted,
        admission_rejected_total = admission.metrics.rejected,
        admission_rejected_llm_total = admission.metrics.rejected_llm_saturated,
        admission_rejected_embedding_total = admission.metrics.rejected_embedding_saturated,
        admission_reject_rate_pct = admission.metrics.reject_rate_pct,
        "telegram runtime snapshot"
    );
}

pub(in crate::channels::telegram::runtime) fn resolve_snapshot_interval_secs<F>(
    lookup: F,
) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    resolve_runtime_snapshot_interval_secs(
        lookup,
        RUNTIME_SNAPSHOT_INTERVAL_ENV,
        DEFAULT_RUNTIME_SNAPSHOT_INTERVAL_SECS,
    )
}
