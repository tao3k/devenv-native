use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;
use xiuxian_config_core::{
    absolutize_path, load_toml_value_with_imports_and_paths,
    resolve_config_home as resolve_config_home_path, resolve_project_root_or_cwd,
};

use super::{EmbeddingSettings, MistralSettings, RuntimeSettings};

const DEFAULT_SYSTEM_SETTINGS_RELATIVE_PATH: &str =
    "packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml";
const DEFAULT_USER_SETTINGS_RELATIVE_PATH: &str = "xiuxian-artisan-workshop/xiuxian.toml";
const DEFAULT_CONFIG_HOME_RELATIVE_PATH: &str = ".config";
const EMBEDDED_SYSTEM_SETTINGS_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/config/xiuxian.toml"
));

static CONFIG_HOME_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();
static EMBEDDED_SYSTEM_SETTINGS: OnceLock<RuntimeSettings> = OnceLock::new();

/// Load merged runtime settings from embedded defaults and cascading system/user paths.
#[must_use]
pub fn load_runtime_settings() -> RuntimeSettings {
    let project_root = resolve_project_root_or_cwd();
    let config_home = resolve_config_home(project_root.as_path());

    match RuntimeSettingsTomlBridge::load_with_paths(
        Some(project_root.as_path()),
        Some(config_home.as_path()),
    ) {
        Ok(bridge) => bridge.into_runtime_settings(),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "failed to load runtime settings via config-core cascade; using tolerant fallback"
            );
            tolerant_load_runtime_settings(project_root.as_path(), config_home.as_path())
        }
    }
}

/// Resolve effective system/user runtime settings paths.
#[doc(hidden)]
pub fn runtime_settings_paths() -> (PathBuf, PathBuf) {
    let root = resolve_project_root_or_cwd();
    let system_path = root.join(DEFAULT_SYSTEM_SETTINGS_RELATIVE_PATH);
    let user_path = resolve_config_home(&root).join(DEFAULT_USER_SETTINGS_RELATIVE_PATH);
    (system_path, user_path)
}

/// Load and merge runtime settings from explicit system/user paths.
///
/// This path is intentionally tolerant: invalid or unreadable files are ignored.
#[doc(hidden)]
#[must_use]
pub fn load_runtime_settings_from_paths(system: &Path, user: &Path) -> RuntimeSettings {
    let (project_root, config_home) = explicit_path_context(system, user);
    load_one_with_paths(system, project_root.as_deref(), config_home.as_deref()).merge(
        load_one_with_paths(
            user,
            project_root.as_deref(),
            config_home.as_deref(),
        ),
    )
}

fn tolerant_load_runtime_settings(project_root: &Path, config_home: &Path) -> RuntimeSettings {
    let system_path = project_root.join(DEFAULT_SYSTEM_SETTINGS_RELATIVE_PATH);
    let user_path = config_home.join(DEFAULT_USER_SETTINGS_RELATIVE_PATH);

    load_embedded_system_settings()
        .merge(load_one(system_path.as_path()))
        .merge(load_one(user_path.as_path()))
}

fn load_embedded_system_settings() -> RuntimeSettings {
    EMBEDDED_SYSTEM_SETTINGS
        .get_or_init(|| parse_runtime_settings_from_str(EMBEDDED_SYSTEM_SETTINGS_TOML, "embedded"))
        .clone()
}

#[xiuxian_macros::xiuxian_config(
    namespace = "",
    internal_path = "resources/config/xiuxian.toml",
    orphan_file = ""
)]
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RuntimeSettingsTomlBridge {
    #[serde(flatten)]
    runtime: RuntimeSettings,
    #[serde(default)]
    llm: RuntimeSettingsLlmBridge,
    #[serde(flatten)]
    _extra: HashMap<String, toml::Value>,
}

impl RuntimeSettingsTomlBridge {
    fn into_runtime_settings(self) -> RuntimeSettings {
        let mut runtime = self.runtime;
        if let Some(embedding) = self.llm.embedding.clone() {
            runtime.embedding = merge_embedding_settings(runtime.embedding, embedding);
        }
        if let Some(mistral) = self.llm.mistral.clone() {
            runtime.mistral = merge_mistral_settings(runtime.mistral, mistral);
        }
        apply_llm_inference_defaults(&mut runtime, &self.llm);
        runtime
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RuntimeSettingsLlmBridge {
    default_provider: Option<String>,
    default_model: Option<String>,
    wire_api: Option<String>,
    #[serde(default)]
    providers: HashMap<String, RuntimeSettingsLlmProviderBridge>,
    embedding: Option<EmbeddingSettings>,
    mistral: Option<MistralSettings>,
    #[serde(flatten)]
    _extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RuntimeSettingsLlmProviderBridge {
    base_url: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    wire_api: Option<String>,
    #[serde(flatten)]
    _extra: HashMap<String, toml::Value>,
}

fn apply_llm_inference_defaults(runtime: &mut RuntimeSettings, llm: &RuntimeSettingsLlmBridge) {
    if runtime.inference.provider.is_none()
        && let Some(provider) = normalize_non_empty(llm.default_provider.as_deref())
    {
        runtime.inference.provider = Some(provider.to_string());
    }

    let provider_name = runtime
        .inference
        .provider
        .as_deref()
        .and_then(|value| normalize_non_empty(Some(value)));
    let provider_config = provider_name.and_then(|name| find_provider_config(&llm.providers, name));

    if runtime.inference.model.is_none()
        && let Some(model) =
            provider_config.and_then(|cfg| normalize_non_empty(cfg.model.as_deref()))
    {
        runtime.inference.model = Some(model.to_string());
    }

    if runtime.inference.model.is_none()
        && let Some(model) = normalize_non_empty(llm.default_model.as_deref())
    {
        runtime.inference.model = Some(model.to_string());
    }

    let Some(provider_config) = provider_config else {
        if runtime.inference.wire_api.is_none()
            && let Some(wire_api) = normalize_non_empty(llm.wire_api.as_deref())
        {
            runtime.inference.wire_api = Some(wire_api.to_string());
        }
        return;
    };

    if runtime.inference.base_url.is_none()
        && let Some(base_url) = normalize_non_empty(provider_config.base_url.as_deref())
    {
        runtime.inference.base_url = Some(base_url.to_string());
    }

    if runtime.inference.api_key.is_none()
        && let Some(api_key) = normalize_non_empty(provider_config.api_key.as_deref())
    {
        runtime.inference.api_key = Some(api_key.to_string());
    }

    if runtime.inference.wire_api.is_none() {
        if let Some(wire_api) = normalize_non_empty(provider_config.wire_api.as_deref()) {
            runtime.inference.wire_api = Some(wire_api.to_string());
        } else if let Some(wire_api) = normalize_non_empty(llm.wire_api.as_deref()) {
            runtime.inference.wire_api = Some(wire_api.to_string());
        }
    }
}

fn find_provider_config<'a>(
    providers: &'a HashMap<String, RuntimeSettingsLlmProviderBridge>,
    provider_name: &str,
) -> Option<&'a RuntimeSettingsLlmProviderBridge> {
    providers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(provider_name))
        .map(|(_, value)| value)
}

fn normalize_non_empty(value: Option<&str>) -> Option<&str> {
    value
        .map(str::trim)
        .and_then(|text| if text.is_empty() { None } else { Some(text) })
}

fn merge_embedding_settings(
    base: EmbeddingSettings,
    overlay: EmbeddingSettings,
) -> EmbeddingSettings {
    EmbeddingSettings {
        backend: overlay.backend.or(base.backend),
        timeout_secs: overlay.timeout_secs.or(base.timeout_secs),
        max_in_flight: overlay.max_in_flight.or(base.max_in_flight),
        batch_max_size: overlay.batch_max_size.or(base.batch_max_size),
        batch_max_concurrency: overlay.batch_max_concurrency.or(base.batch_max_concurrency),
        model: overlay.model.or(base.model),
        litellm_model: overlay.litellm_model.or(base.litellm_model),
        litellm_api_base: overlay.litellm_api_base.or(base.litellm_api_base),
        dimension: overlay.dimension.or(base.dimension),
        client_url: overlay.client_url.or(base.client_url),
    }
}

fn merge_mistral_settings(base: MistralSettings, overlay: MistralSettings) -> MistralSettings {
    MistralSettings {
        enabled: overlay.enabled.or(base.enabled),
        auto_start: overlay.auto_start.or(base.auto_start),
        command: overlay.command.or(base.command),
        args: overlay.args.or(base.args),
        base_url: overlay.base_url.or(base.base_url),
        startup_timeout_secs: overlay.startup_timeout_secs.or(base.startup_timeout_secs),
        probe_timeout_ms: overlay.probe_timeout_ms.or(base.probe_timeout_ms),
        probe_interval_ms: overlay.probe_interval_ms.or(base.probe_interval_ms),
        sdk_hf_cache_path: overlay.sdk_hf_cache_path.or(base.sdk_hf_cache_path),
        sdk_hf_revision: overlay.sdk_hf_revision.or(base.sdk_hf_revision),
        sdk_embedding_max_num_seqs: overlay
            .sdk_embedding_max_num_seqs
            .or(base.sdk_embedding_max_num_seqs),
    }
}

fn load_one(path: &Path) -> RuntimeSettings {
    load_one_with_paths(path, None, None)
}

fn load_one_with_paths(
    path: &Path,
    project_root: Option<&Path>,
    config_home: Option<&Path>,
) -> RuntimeSettings {
    if !path.is_file() {
        return RuntimeSettings::default();
    }
    let merged = match load_toml_value_with_imports_and_paths(path, project_root, config_home) {
        Ok(merged) => merged,
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                error = %error,
                "failed to load settings file via config-core; ignoring"
            );
            return RuntimeSettings::default();
        }
    };
    match merged.try_into::<RuntimeSettingsTomlBridge>() {
        Ok(bridge) => bridge.into_runtime_settings(),
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                error = %error,
                "failed to parse merged settings toml; ignoring file"
            );
            RuntimeSettings::default()
        }
    }
}

fn parse_runtime_settings_from_str(raw: &str, context: &str) -> RuntimeSettings {
    match toml::from_str::<RuntimeSettingsTomlBridge>(raw) {
        Ok(bridge) => bridge.into_runtime_settings(),
        Err(error) => {
            tracing::warn!(
                error = %error,
                %context,
                "failed to parse runtime settings bridge; falling back to defaults"
            );
            RuntimeSettings::default()
        }
    }
}

/// Set config-home override (used by CLI `--conf`).
///
/// The path can be absolute, or relative to `PRJ_ROOT`/cwd.
pub fn set_config_home_override(path: impl Into<PathBuf>) {
    let path = path.into();
    if path.as_os_str().is_empty() {
        return;
    }
    if CONFIG_HOME_OVERRIDE.set(path.clone()).is_err()
        && let Some(current) = CONFIG_HOME_OVERRIDE.get()
        && current != &path
    {
        tracing::warn!(
            current = %current.display(),
            ignored = %path.display(),
            "config home override already set; ignoring subsequent value"
        );
    }
}

fn resolve_config_home(project_root: &Path) -> PathBuf {
    if let Some(path) = CONFIG_HOME_OVERRIDE.get() {
        return absolutize_path(project_root, path.as_path());
    }

    resolve_config_home_path(Some(project_root))
        .unwrap_or_else(|| project_root.join(DEFAULT_CONFIG_HOME_RELATIVE_PATH))
}

fn explicit_path_context(system: &Path, user: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    let base_root = resolve_project_root_or_cwd();
    let system_path = absolutize_path(base_root.as_path(), system);
    let user_path = absolutize_path(base_root.as_path(), user);
    let project_root =
        common_ancestor(system_path.as_path(), user_path.as_path()).or(Some(base_root));
    let config_home = infer_config_home(user_path.as_path(), project_root.as_deref());
    (project_root, config_home)
}

fn infer_config_home(user_path: &Path, project_root: Option<&Path>) -> Option<PathBuf> {
    for ancestor in user_path.ancestors() {
        if ancestor
            .file_name()
            .is_some_and(|name| name == DEFAULT_CONFIG_HOME_RELATIVE_PATH)
        {
            return Some(ancestor.to_path_buf());
        }
    }

    project_root.map(|root| root.join(DEFAULT_CONFIG_HOME_RELATIVE_PATH))
}

fn common_ancestor(left: &Path, right: &Path) -> Option<PathBuf> {
    let left_components = left.components().collect::<Vec<_>>();
    let right_components = right.components().collect::<Vec<_>>();
    let shared_len = left_components
        .iter()
        .zip(right_components.iter())
        .take_while(|(lhs, rhs)| lhs == rhs)
        .count();
    if shared_len == 0 {
        return None;
    }

    let mut shared = PathBuf::new();
    for component in left_components.into_iter().take(shared_len) {
        shared.push(component.as_os_str());
    }
    Some(shared)
}
