use std::path::Path;

use tempfile::TempDir;
pub(super) use xiuxian_daochang::{build_telegram_acl_overrides, load_runtime_settings_from_paths};

pub(super) fn require_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

pub(super) fn require_some<T>(value: Option<T>, context: &str) -> T {
    value.unwrap_or_else(|| panic!("{context}"))
}

pub(super) fn write_file(path: impl AsRef<Path>, content: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        panic!("create parent directories: {error}");
    }
    if let Err(error) = std::fs::write(path, content) {
        panic!("write file: {error}");
    }
}

pub(super) fn new_temp_settings_paths() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");
    (tmp, system, user)
}

pub(super) fn assert_nested_llm_merge(merged: &xiuxian_daochang::RuntimeSettings) {
    assert_eq!(merged.agent.llm_backend.as_deref(), Some("litellm_rs"));
    assert_eq!(merged.inference.provider.as_deref(), Some("minimax"));
    assert_eq!(merged.inference.api_key.as_deref(), Some("MINIMAX_API_KEY"));
    assert_eq!(merged.inference.model.as_deref(), Some("gpt-4o-mini"));
    assert_eq!(merged.inference.timeout, Some(90));

    assert_eq!(merged.tool_runtime.pool_size, Some(8));
    assert_eq!(merged.tool_runtime.strict_startup, Some(false));
    assert_eq!(merged.tool_runtime.list_tools_cache_ttl_ms, Some(900));

    assert_eq!(merged.session.context_budget_tokens, Some(7000));
    assert_eq!(
        merged.session.context_budget_strategy.as_deref(),
        Some("recent_first")
    );

    assert_eq!(merged.embedding.backend.as_deref(), Some("litellm_rs"));
    assert_eq!(
        merged.embedding.model.as_deref(),
        Some("ollama/qwen3-embedding:0.6b")
    );
    assert_eq!(
        merged.embedding.litellm_api_base.as_deref(),
        Some("http://127.0.0.1:11434")
    );
    assert_eq!(
        merged.embedding.client_url.as_deref(),
        Some("http://127.0.0.1:3002")
    );
    assert_eq!(merged.embedding.timeout_secs, Some(47));
    assert_eq!(merged.embedding.max_in_flight, Some(96));
    assert_eq!(merged.embedding.batch_max_size, Some(256));

    assert_eq!(merged.memory.embedding_timeout_ms, Some(12000));
    assert_eq!(merged.memory.persistence_backend.as_deref(), Some("local"));

    assert_eq!(merged.mistral.enabled, Some(true));
    assert_eq!(merged.mistral.auto_start, Some(false));
    assert_eq!(
        merged.mistral.base_url.as_deref(),
        Some("http://127.0.0.1:11435/v1")
    );
    assert_eq!(
        merged.mistral.sdk_hf_cache_path.as_deref(),
        Some(".data/models/hf-cache")
    );
    assert_eq!(merged.mistral.sdk_hf_revision.as_deref(), Some("v2"));
    assert_eq!(merged.mistral.sdk_embedding_max_num_seqs, Some(128));
}

pub(super) fn assert_telegram_group_merge(merged: &xiuxian_daochang::RuntimeSettings) {
    assert_eq!(merged.telegram.group_policy.as_deref(), Some("open"));
    assert_eq!(merged.telegram.group_allow_from.as_deref(), Some("ops"));
    assert_eq!(merged.telegram.session_admin_persist, Some(false));
    assert_eq!(merged.telegram.session_partition_persist, Some(false));
    assert_eq!(merged.telegram.require_mention, Some(false));

    let groups = require_some(merged.telegram.groups.as_ref(), "merged groups");
    let wildcard = require_some(groups.get("*"), "wildcard group");
    assert_eq!(
        wildcard
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["9090".to_string()])
    );
    assert_eq!(wildcard.require_mention, Some(true));

    let group_100 = require_some(groups.get("-100"), "group -100");
    assert_eq!(group_100.group_policy.as_deref(), Some("disabled"));
    assert_eq!(
        group_100
            .allow_from
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["admin2".to_string()])
    );
    assert_eq!(
        group_100
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["3002".to_string()])
    );

    let topics_100 = require_some(group_100.topics.as_ref(), "group -100 topics");
    let topic_10 = require_some(topics_100.get("10"), "topic 10");
    assert_eq!(
        topic_10
            .allow_from
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["ops1".to_string()])
    );
    assert_eq!(
        topic_10
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["7002".to_string()])
    );
    assert_eq!(topic_10.require_mention, Some(true));

    let topic_11 = require_some(topics_100.get("11"), "topic 11");
    assert_eq!(
        topic_11
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["8001".to_string()])
    );
    assert_eq!(topic_11.enabled, Some(true));

    let group_200 = require_some(groups.get("-200"), "group -200");
    assert_eq!(
        group_200
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["4001".to_string()])
    );
    assert_eq!(group_200.enabled, Some(true));
}
