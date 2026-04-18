use xiuxian_daochang::XiuxianConfig;
use xiuxian_daochang::test_support::{ZhenfaRuntimeDeps, ZhenfaToolBridge};

use super::support::build_manifestation_manager;

#[test]
fn from_xiuxian_config_requires_explicit_supported_tool_enablement() {
    let config = XiuxianConfig::default();
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &ZhenfaRuntimeDeps::default());
    assert!(bridge.is_none());
}

#[test]
fn from_xiuxian_config_skips_explicit_wendao_search_request() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec!["wendao.search".to_string()]);
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &ZhenfaRuntimeDeps::default());
    assert!(bridge.is_none());
}

#[test]
fn from_xiuxian_config_filters_unknown_tools() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec!["unknown.tool".to_string()]);

    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &ZhenfaRuntimeDeps::default());
    assert!(bridge.is_none());
}

#[test]
fn from_xiuxian_config_skips_qianhuan_tools_without_runtime_dependency() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec![
        "qianhuan.render".to_string(),
        "qianhuan.reload".to_string(),
    ]);

    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &ZhenfaRuntimeDeps::default());
    assert!(bridge.is_none());
}

#[test]
fn from_xiuxian_config_enables_qianhuan_tools_when_runtime_dependency_is_available() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec![
        "qianhuan.render".to_string(),
        "qianhuan.reload".to_string(),
    ]);
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };

    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));
    assert!(bridge.handles_tool("qianhuan.render"));
    assert!(bridge.handles_tool("qianhuan.reload"));
}

#[test]
fn from_xiuxian_config_enables_valkey_hooks_when_configured() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec!["qianhuan.reload".to_string()]);
    config.zhenfa.valkey.url = Some("redis://127.0.0.1:6379/0".to_string());
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));
    assert!(bridge.valkey_hooks_enabled());
}

#[test]
fn from_xiuxian_config_ignores_wendao_search_when_mixed_with_supported_tools() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec![
        "wendao.search".to_string(),
        "qianhuan.reload".to_string(),
    ]);
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));
    assert!(bridge.handles_tool("qianhuan.reload"));
    assert!(!bridge.handles_tool("wendao.search"));
}

#[tokio::test]
async fn call_tool_rejects_wendao_search_when_bridge_has_other_tools() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec![
        "wendao.search".to_string(),
        "qianhuan.reload".to_string(),
    ]);
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));

    let Err(error) = bridge
        .call_tool(
            Some("telegram:12345"),
            "wendao.search",
            Some(serde_json::json!({
                "request": "show me docs"
            })),
        )
        .await
    else {
        panic!("wendao.search should no longer be bridged through zhenfa");
    };
    assert!(error.to_string().contains("not enabled"));
}

#[tokio::test]
async fn call_tool_dispatches_qianhuan_reload_natively() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec!["qianhuan.reload".to_string()]);
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));

    let output = bridge
        .call_tool(Some("telegram:12345"), "qianhuan.reload", None)
        .await
        .unwrap_or_else(|error| panic!("zhenfa native tool call should succeed: {error}"));
    assert!(output.contains("<qianhuan_reload"));
}
