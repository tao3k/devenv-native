use std::collections::HashMap;

use xiuxian_daochang::test_support::resolve_telegram_snapshot_interval_secs as resolve_snapshot_interval_secs;

#[test]
fn telegram_runtime_snapshot_interval_defaults_when_unset() {
    let resolved = resolve_snapshot_interval_secs(|_| None);
    assert_eq!(resolved, Some(30));
}

#[test]
fn telegram_runtime_snapshot_interval_uses_positive_override() {
    let values = HashMap::from([(
        "XIUXIAN_DAOCHANG_TELEGRAM_RUNTIME_SNAPSHOT_INTERVAL_SECS".to_string(),
        "12".to_string(),
    )]);
    let resolved = resolve_snapshot_interval_secs(|name| values.get(name).cloned());
    assert_eq!(resolved, Some(12));
}

#[test]
fn telegram_runtime_snapshot_interval_zero_disables_snapshots() {
    let values = HashMap::from([(
        "XIUXIAN_DAOCHANG_TELEGRAM_RUNTIME_SNAPSHOT_INTERVAL_SECS".to_string(),
        "0".to_string(),
    )]);
    let resolved = resolve_snapshot_interval_secs(|name| values.get(name).cloned());
    assert_eq!(resolved, None);
}

#[test]
fn telegram_runtime_snapshot_interval_invalid_falls_back_to_default() {
    let values = HashMap::from([(
        "XIUXIAN_DAOCHANG_TELEGRAM_RUNTIME_SNAPSHOT_INTERVAL_SECS".to_string(),
        "invalid".to_string(),
    )]);
    let resolved = resolve_snapshot_interval_secs(|name| values.get(name).cloned());
    assert_eq!(resolved, Some(30));
}
