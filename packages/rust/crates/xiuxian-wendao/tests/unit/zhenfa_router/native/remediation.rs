use super::{RemediationAction, RemediationConfig, RemediationResult};

#[test]
fn remediation_config_default() {
    let config = RemediationConfig::default();
    assert!(config.auto_refresh_enabled);
    assert!(config.symbol_cache_sync_enabled);
    assert_eq!(config.max_concurrency, 4);
}

#[test]
fn remediation_action_debug() {
    let action = RemediationAction::RefreshSymbolCache {
        doc_id: "docs/api".to_string().into(),
    };
    let debug_str = format!("{action:?}");
    assert!(debug_str.contains("RefreshSymbolCache"));
    assert!(debug_str.contains("docs/api"));
}

#[test]
fn remediation_result_success() {
    let result = RemediationResult {
        action: RemediationAction::NoOp,
        success: true,
        error: None,
        duration_ms: 42,
    };
    assert!(result.success);
    assert!(result.error.is_none());
}

#[test]
fn remediation_result_failure() {
    let result = RemediationResult {
        action: RemediationAction::NoOp,
        success: false,
        error: Some("Test error".to_string()),
        duration_ms: 0,
    };
    assert!(!result.success);
    assert_eq!(result.error, Some("Test error".to_string()));
}

#[test]
fn remediation_action_clone() {
    let action = RemediationAction::IncrementalRebuild {
        source_path: "src/lib.rs".to_string(),
        affected_docs: vec!["docs/a".to_string()],
    };
    let cloned = action.clone();
    match cloned {
        RemediationAction::IncrementalRebuild { source_path, .. } => {
            assert_eq!(source_path, "src/lib.rs");
        }
        _ => panic!("Expected IncrementalRebuild"),
    }
}

#[test]
fn remediation_config_clone() {
    let config = RemediationConfig {
        auto_refresh_enabled: false,
        symbol_cache_sync_enabled: true,
        max_concurrency: 8,
        emit_refresh_signals: false,
    };
    let cloned = config.clone();
    assert!(!cloned.auto_refresh_enabled);
    assert_eq!(cloned.max_concurrency, 8);
    assert!(!cloned.emit_refresh_signals);
}
