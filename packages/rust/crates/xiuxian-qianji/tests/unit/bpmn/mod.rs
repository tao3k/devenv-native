#[cfg(feature = "valkey")]
use crate::qianji_test_valkey_support as valkey_support;

mod adapter;
#[cfg(feature = "valkey")]
mod control;
mod flowhub_activity_adapter;
mod http;
mod llm_activity_adapter;
#[cfg(feature = "run-console-flight")]
mod run_console_flight;
#[cfg(feature = "run-console-flight")]
mod run_console_read_model;
mod runtime;
mod runtime_identity;
#[cfg(feature = "valkey")]
mod runtime_lease;
mod runtime_selector;

#[cfg(feature = "valkey")]
pub(super) fn unique_instance_id(base: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{base}_{}_{}", std::process::id(), nanos)
}
