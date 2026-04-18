use super::support::{
    assert_nested_llm_merge, build_telegram_acl_overrides, load_runtime_settings_from_paths,
    new_temp_settings_paths, require_ok, write_file,
};

#[test]
fn merge_user_overrides_system_with_nested_llm_sections() {
    let (_tmp, system, user) = new_temp_settings_paths();

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

#[test]
fn embedding_timeout_alias_timeout_is_supported() {
    let (_tmp, system, user) = new_temp_settings_paths();

    write_file(
        &system,
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

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.embedding.backend.as_deref(), Some("http"));
    assert_eq!(merged.embedding.timeout_secs, Some(47));
}

#[test]
fn llm_default_provider_populates_inference_defaults_when_missing() {
    let (_tmp, system, user) = new_temp_settings_paths();

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
    let (_tmp, system, user) = new_temp_settings_paths();

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
    let (_tmp, system, user) = new_temp_settings_paths();

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
    let (_tmp, system, user) = new_temp_settings_paths();

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
