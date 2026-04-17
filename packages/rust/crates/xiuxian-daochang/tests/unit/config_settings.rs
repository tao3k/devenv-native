//! Test coverage for xiuxian-daochang behavior.

use std::path::Path;

use tempfile::TempDir;
use xiuxian_daochang::{build_telegram_acl_overrides, load_runtime_settings_from_paths};

fn require_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn require_some<T>(value: Option<T>, context: &str) -> T {
    value.unwrap_or_else(|| panic!("{context}"))
}

fn write_file(path: impl AsRef<Path>, content: &str) {
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

#[test]
fn merge_channel_foreground_queue_mode_uses_user_override() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r#"
[telegram]
foreground_queue_mode = "interrupt"

[discord]
foreground_queue_mode = "interrupt"
"#,
    );
    write_file(
        &user,
        r#"
[telegram]
foreground_queue_mode = "queue"

[discord]
foreground_queue_mode = "queue"
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(
        merged.telegram.foreground_queue_mode.as_deref(),
        Some("queue")
    );
    assert_eq!(
        merged.discord.foreground_queue_mode.as_deref(),
        Some("queue")
    );
}

#[test]
fn merge_user_overrides_system_with_nested_llm_sections() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r#"
[agent]
llm_backend = "http"

[inference]
provider = "openai"
api_key = "OPENAI_API_KEY"
model = "gpt-4o-mini"
timeout = 45

[tool_runtime]
pool_size = 4
strict_startup = true
list_tools_cache_ttl_ms = 900

[session]
context_budget_tokens = 5000
context_budget_strategy = "recent_first"

[embedding]
backend = "http"
timeout_secs = 11

[llm.embedding]
backend = "litellm_rs"
model = "ollama/qwen3-embedding:0.6b"
litellm_api_base = "http://127.0.0.1:11434"
client_url = "http://127.0.0.1:3002"
timeout = 31

[memory]
embedding_timeout_ms = 7000
persistence_backend = "local"

[llm.mistral]
enabled = true
auto_start = true
base_url = "http://127.0.0.1:11435/v1"
sdk_hf_cache_path = ".data/models/hf-cache"
sdk_hf_revision = "main"
sdk_embedding_max_num_seqs = 64
"#,
    );
    write_file(
        &user,
        r#"
[agent]
llm_backend = "litellm_rs"

[inference]
provider = "minimax"
api_key = "MINIMAX_API_KEY"
timeout = 90

[tool_runtime]
pool_size = 8
strict_startup = false

[session]
context_budget_tokens = 7000

[llm.embedding]
timeout_secs = 47
max_in_flight = 96
batch_max_size = 256

[memory]
embedding_timeout_ms = 12000

[llm.mistral]
auto_start = false
sdk_hf_revision = "v2"
sdk_embedding_max_num_seqs = 128
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_nested_llm_merge(&merged);
}

fn assert_nested_llm_merge(merged: &xiuxian_daochang::RuntimeSettings) {
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

#[test]
fn merge_telegram_group_policy_overrides_deeply() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r#"
[telegram]
group_policy = "allowlist"
group_allow_from = "ops"
session_admin_persist = true
session_partition_persist = true
require_mention = true

[telegram.groups."*"]
require_mention = true

[telegram.groups."*".admin_users]
users = ["9090"]

[telegram.groups."*".topics."42"]
enabled = false

[telegram.groups."-100"]
group_policy = "disabled"

[telegram.groups."-100".allow_from]
users = ["root"]

[telegram.groups."-100".admin_users]
users = ["3001"]

[telegram.groups."-100".topics."10".allow_from]
users = ["ops1"]

[telegram.groups."-100".topics."10".admin_users]
users = ["7001"]
"#,
    );

    write_file(
        &user,
        r#"
[telegram]
group_policy = "open"
session_admin_persist = false
session_partition_persist = false
require_mention = false

[telegram.groups."-100".allow_from]
users = ["admin2"]

[telegram.groups."-100".admin_users]
users = ["3002"]

[telegram.groups."-100".topics."10"]
require_mention = true

[telegram.groups."-100".topics."10".admin_users]
users = ["7002"]

[telegram.groups."-100".topics."11"]
enabled = true

[telegram.groups."-100".topics."11".admin_users]
users = ["8001"]

[telegram.groups."-200"]
enabled = true

[telegram.groups."-200".admin_users]
users = ["4001"]
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_telegram_group_merge(&merged);
}

fn assert_telegram_group_merge(merged: &xiuxian_daochang::RuntimeSettings) {
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

#[test]
fn merge_discord_session_partition_persist_uses_user_override() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r"
[discord]
session_partition_persist = true
",
    );
    write_file(
        &user,
        r"
[discord]
session_partition_persist = false
",
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.discord.session_partition_persist, Some(false));
}

#[test]
fn merge_discord_mention_policy_overrides_deeply() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r#"
[discord]
require_mention = true
require_mention_persist = false

[discord.channels."*"]
require_mention = true

[discord.channels."2001"]
require_mention = false
"#,
    );
    write_file(
        &user,
        r#"
[discord]
require_mention = false
require_mention_persist = true

[discord.channels."2001"]
require_mention = true

[discord.channels."2002"]
require_mention = false
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.discord.require_mention, Some(false));
    assert_eq!(merged.discord.require_mention_persist, Some(true));
    let channels = require_some(merged.discord.channels.as_ref(), "discord channels");
    assert_eq!(
        channels.get("*").and_then(|value| value.require_mention),
        Some(true)
    );
    assert_eq!(
        channels.get("2001").and_then(|value| value.require_mention),
        Some(true)
    );
    assert_eq!(
        channels.get("2002").and_then(|value| value.require_mention),
        Some(false)
    );
}

#[test]
fn missing_files_fallback_to_defaults() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let merged = load_runtime_settings_from_paths(
        &tmp.path().join("missing-system.toml"),
        &tmp.path().join("missing-user.toml"),
    );
    let telegram_overrides = require_ok(
        build_telegram_acl_overrides(&merged),
        "telegram acl overrides",
    );
    assert!(telegram_overrides.allowed_users.is_empty());
    assert!(telegram_overrides.allowed_groups.is_empty());
    assert!(merged.telegram.group_policy.is_none());
    assert!(merged.tool_runtime.pool_size.is_none());
    assert!(merged.embedding.backend.is_none());
    assert!(merged.memory.embedding_timeout_ms.is_none());
}

#[test]
fn invalid_toml_is_ignored() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(&system, "[telegram");
    write_file(
        user,
        r#"
[telegram.acl.allow]
users = ["1001"]
"#,
    );

    let merged = load_runtime_settings_from_paths(
        &tmp.path()
            .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml"),
        &tmp.path()
            .join(".config/xiuxian-artisan-workshop/xiuxian.toml"),
    );
    let telegram_overrides = require_ok(
        build_telegram_acl_overrides(&merged),
        "telegram acl overrides",
    );
    assert_eq!(telegram_overrides.allowed_users, vec!["1001"]);
}

#[test]
fn embedding_timeout_alias_timeout_is_supported() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        system,
        r#"
[llm.embedding]
backend = "http"
timeout = 31
"#,
    );
    write_file(
        &user,
        r"
[llm.embedding]
timeout_secs = 47
",
    );

    let merged = load_runtime_settings_from_paths(
        &tmp.path()
            .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml"),
        &tmp.path()
            .join(".config/xiuxian-artisan-workshop/xiuxian.toml"),
    );
    assert_eq!(merged.embedding.backend.as_deref(), Some("http"));
    assert_eq!(merged.embedding.timeout_secs, Some(47));
}

#[test]
fn llm_default_provider_populates_inference_defaults_when_missing() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r#"
[llm]
default_provider = "minimax"
default_model = "MiniMax-M2.5"

[llm.providers.minimax]
base_url = "https://api.minimax.io/v1"
api_key = "MINIMAX_API_KEY"
"#,
    );
    write_file(&user, "");

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.inference.provider.as_deref(), Some("minimax"));
    assert_eq!(merged.inference.model.as_deref(), Some("MiniMax-M2.5"));
    assert_eq!(
        merged.inference.base_url.as_deref(),
        Some("https://api.minimax.io/v1")
    );
    assert_eq!(merged.inference.api_key.as_deref(), Some("MINIMAX_API_KEY"));
}

#[test]
fn llm_provider_model_overrides_global_default_model_for_selected_provider() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r#"
[llm]
default_provider = "anthropic"
default_model = "glm-5"

[llm.providers.anthropic]
base_url = "https://aiproxy.xin/api"
api_key = "ANTHROPIC_API_KEY"
model = "claude-3-5-sonnet-20241022"
"#,
    );
    write_file(&user, "");

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.inference.provider.as_deref(), Some("anthropic"));
    assert_eq!(
        merged.inference.model.as_deref(),
        Some("claude-3-5-sonnet-20241022")
    );
    assert_eq!(
        merged.inference.base_url.as_deref(),
        Some("https://aiproxy.xin/api")
    );
    assert_eq!(
        merged.inference.api_key.as_deref(),
        Some("ANTHROPIC_API_KEY")
    );
}

#[test]
fn llm_provider_wire_api_overrides_global_wire_api_for_selected_provider() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r#"
[llm]
default_provider = "openai"
wire_api = "chat_completions"

[llm.providers.openai]
base_url = "https://aiproxy.xin/openai"
api_key = "OPENAI_API_KEY"
model = "gpt-5-codex"
wire_api = "responses"
"#,
    );
    write_file(&user, "");

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.inference.provider.as_deref(), Some("openai"));
    assert_eq!(merged.inference.wire_api.as_deref(), Some("responses"));
}

#[test]
fn llm_provider_extra_fields_keep_inference_bridge_active() {
    let tmp = require_ok(TempDir::new(), "tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        &system,
        r#"
[llm]
default_provider = "minimax"
default_model = "MiniMax-M2.5"

[llm.providers.minimax]
base_url = "https://api.minimax.io/v1"
api_key = "MINIMAX_API_KEY"

[llm.providers.minimax.model_aliases]
"minimax-m2.1-highspeed" = "MiniMax-M2.1-lightning"
"#,
    );
    write_file(
        &user,
        r#"
[wendao.link_graph.index.delta]
full_rebuild_threshold = 256
stats_persistent_cache_ttl_sec = 120.0

[telegram.acl.allow]
users = ["1304799691"]
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.inference.provider.as_deref(), Some("minimax"));
    assert_eq!(merged.inference.model.as_deref(), Some("MiniMax-M2.5"));
    assert_eq!(
        merged.inference.base_url.as_deref(),
        Some("https://api.minimax.io/v1")
    );
    assert_eq!(merged.inference.api_key.as_deref(), Some("MINIMAX_API_KEY"));
    let telegram_overrides = require_ok(
        build_telegram_acl_overrides(&merged),
        "telegram acl overrides",
    );
    assert_eq!(telegram_overrides.allowed_users, vec!["1304799691"]);
}
