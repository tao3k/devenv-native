use crate::qianji_test_valkey_support as valkey_support;

mod adapter;
mod control;
mod http;
mod runtime;
mod runtime_identity;
mod runtime_lease;
mod runtime_selector;

pub(super) fn unique_instance_id(base: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{base}_{}_{}", std::process::id(), nanos)
}
