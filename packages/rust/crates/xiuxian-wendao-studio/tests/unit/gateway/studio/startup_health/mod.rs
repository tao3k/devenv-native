use super::{
    GatewayStartupDependencyCheck, GatewayStartupDependencyStatus, GatewayStartupHealthReport,
    describe_gateway_startup_health,
};

#[test]
fn startup_health_report_requires_all_dependencies_to_be_connected() {
    let report = GatewayStartupHealthReport::new(vec![
        GatewayStartupDependencyCheck::connected("plugins", "plugins=julia"),
        GatewayStartupDependencyCheck::failed("search_cache_valkey", "connection failed"),
    ]);

    assert!(!report.is_ready());
    assert_eq!(
        report.failure_summary(),
        Some("search_cache_valkey (connection failed)".to_string())
    );
}

#[test]
fn startup_health_report_is_ready_when_all_checks_pass() {
    let report = GatewayStartupHealthReport::new(vec![
        GatewayStartupDependencyCheck::connected("plugins", "plugins=julia,modelica"),
        GatewayStartupDependencyCheck::connected(
            "search_cache_valkey",
            "url=redis://127.0.0.1:6379/0 ping=PONG",
        ),
        GatewayStartupDependencyCheck::connected(
            "link_graph_cache_valkey",
            "url=redis://127.0.0.1:6379/0 ping=PONG",
        ),
    ]);

    assert!(report.is_ready());
    assert!(report.failure_summary().is_none());
}

#[test]
fn describe_gateway_startup_health_includes_status_labels() {
    let report = GatewayStartupHealthReport::new(vec![
        GatewayStartupDependencyCheck {
            dependency: "builtin_plugin_registry",
            status: GatewayStartupDependencyStatus::Connected,
            detail: "plugins=julia".to_string(),
        },
        GatewayStartupDependencyCheck {
            dependency: "search_cache_valkey",
            status: GatewayStartupDependencyStatus::Failed,
            detail: "connection failed".to_string(),
        },
    ]);

    let lines = describe_gateway_startup_health(&report);
    assert_eq!(
        lines,
        vec![
            "builtin_plugin_registry=connected plugins=julia".to_string(),
            "search_cache_valkey=failed connection failed".to_string(),
        ]
    );
}
