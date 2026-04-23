use super::env_vars::env_var_or_override;
use super::model::QianjiRuntimeEnv;
use super::toml_config::{
    QianjiToml, apply_checkpoint_overlay, apply_llm_overlay, apply_memory_promotion_overlay,
    apply_server_overlay,
};
use std::io;
use std::path::{Path, PathBuf};
use xiuxian_config_core::{ConfigCoreError, load_toml_value_with_imports};

pub(super) fn load_qianji_toml(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
    config_home: &Path,
) -> io::Result<QianjiToml> {
    let mut merged = QianjiToml::default();

    for path in qianji_toml_candidates(runtime_env, project_root, config_home) {
        if !path.exists() {
            continue;
        }
        let parsed = read_qianji_toml_file(&path)?;
        apply_llm_overlay(&mut merged.llm, parsed.llm);
        apply_memory_promotion_overlay(&mut merged.memory_promotion, parsed.memory_promotion);
        apply_checkpoint_overlay(&mut merged.checkpoint, parsed.checkpoint);
        apply_server_overlay(&mut merged.server, parsed.server);
    }

    Ok(merged)
}

fn qianji_toml_candidates(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
    config_home: &Path,
) -> Vec<PathBuf> {
    let mut candidates =
        vec![project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml")];

    let user_qianji_toml = config_home.join("xiuxian-artisan-workshop/qianji.toml");
    if user_qianji_toml.exists() {
        candidates.push(user_qianji_toml);
    } else {
        candidates.push(config_home.join("xiuxian-artisan-workshop/xiuxian.toml"));
    }

    if let Some(explicit) = resolve_explicit_qianji_config_path(runtime_env) {
        candidates.push(explicit);
    }

    candidates
}

fn resolve_explicit_qianji_config_path(runtime_env: &QianjiRuntimeEnv) -> Option<PathBuf> {
    runtime_env.qianji_config_path.clone().or_else(|| {
        env_var_or_override(runtime_env, "QIANJI_CONFIG_PATH")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    })
}

fn read_qianji_toml_file(path: &Path) -> io::Result<QianjiToml> {
    let value = load_toml_value_with_imports(path).map_err(|error| {
        let kind = match &error {
            ConfigCoreError::ReadFile { source, .. } => source.kind(),
            _ => io::ErrorKind::InvalidData,
        };
        io::Error::new(
            kind,
            format!("failed to load qianji config {}: {error}", path.display()),
        )
    })?;
    value.try_into::<QianjiToml>().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse qianji config {}: {e}", path.display()),
        )
    })
}
