use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const REGISTRY_INDEX_KEY: &str = "xiuxian:swarm:registry:index";

pub(in crate::swarm) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|inner| inner.trim().to_string())
        .and_then(|inner| if inner.is_empty() { None } else { Some(inner) })
}

pub(in crate::swarm) fn current_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
