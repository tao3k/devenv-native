use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use xiuxian_git_repo::SyncMode;
use xiuxian_wendao::episteme::{
    EpistemeRegistryEntry, EpistemeRuntimeConfig, configured_episteme_corpus_root_env,
    load_episteme_registry_entries_with_mode, load_episteme_runtime_config,
    validate_episteme_registry_reference_graph,
};

use crate::bin_support::wendao::types::Cli;
use crate::studio::router::{
    load_episteme_registry_from_wendao_toml, load_episteme_registry_from_wendao_toml_path,
};

pub(super) fn resolve_episteme_root(
    cli: &Cli,
    episteme_root: &Path,
    episteme_registry_id: Option<&str>,
) -> Result<PathBuf> {
    let Some(registry_id) = episteme_registry_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(episteme_root.to_path_buf());
    };
    let entries = load_episteme_registry_entries(cli)?;
    let entry = entries
        .iter()
        .find(|entry| entry.id == registry_id)
        .with_context(|| format!("episteme registry `{registry_id}` is not configured"))?;
    if !entry.enabled {
        anyhow::bail!("episteme registry `{registry_id}` is disabled");
    }
    let receipt = load_episteme_registry_entries_with_mode(&entries, &cli.root, SyncMode::Ensure)
        .with_context(|| format!("failed to load episteme registry `{registry_id}`"))?;
    validate_episteme_registry_reference_graph(&receipt)
        .with_context(|| format!("failed to validate episteme registry `{registry_id}` graph"))?;
    receipt
        .entries
        .into_iter()
        .find(|entry| entry.id == registry_id)
        .map(|entry| entry.episteme_root)
        .with_context(|| format!("episteme registry `{registry_id}` did not load an episteme root"))
}

pub(super) fn load_runtime_config(episteme_root: &Path) -> Result<Option<EpistemeRuntimeConfig>> {
    load_episteme_runtime_config(episteme_root).with_context(|| {
        format!(
            "failed to load episteme runtime config from `{}`",
            episteme_root.join("episteme.toml").display()
        )
    })
}

pub(super) fn resolve_corpus_root(
    corpus_root: Option<&PathBuf>,
    episteme_root: &Path,
    config: Option<&EpistemeRuntimeConfig>,
) -> Result<PathBuf> {
    if let Some(path) = corpus_root {
        return Ok(path.clone());
    }
    if let Some(path) = config.and_then(|config| config.corpus.clone()) {
        return Ok(path);
    }
    let corpus_root_env = configured_episteme_corpus_root_env(episteme_root)
        .context("failed to read episteme-configured corpus root env")?;
    env::var_os(corpus_root_env.as_str())
        .map(PathBuf::from)
        .with_context(|| {
            format!(
                "--corpus-root is required when episteme.toml has no runtime.corpus_root and {corpus_root_env} is not set"
            )
        })
}

pub(super) fn resolve_run_root(
    explicit: Option<&PathBuf>,
    configured: Option<&PathBuf>,
    fallback: impl FnOnce() -> PathBuf,
) -> PathBuf {
    explicit
        .cloned()
        .or_else(|| configured.cloned())
        .unwrap_or_else(fallback)
}

pub(super) fn resolve_legacy_office_converter(
    explicit: Option<&PathBuf>,
    config: Option<&EpistemeRuntimeConfig>,
    dry_run: bool,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.clone());
    }
    if let Some(path) = config.and_then(|config| config.legacy_office_converter.clone()) {
        return Ok(path);
    }
    if dry_run {
        return Ok(PathBuf::from("legacy-office-converter"));
    }
    anyhow::bail!(
        "--converter-command or runtime.legacy_office_converter in episteme.toml is required for legacy Office conversion"
    )
}

pub(super) fn absolute_runtime_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    env::current_dir()
        .map(|current_dir| current_dir.join(path))
        .context("failed to resolve current directory for episteme command paths")
}

pub(super) fn path_display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn load_episteme_registry_entries(cli: &Cli) -> Result<Vec<EpistemeRegistryEntry>> {
    if let Some(config_file) = &cli.config_file {
        return load_episteme_registry_from_wendao_toml_path(config_file.as_path()).map_err(
            |error| {
                anyhow::anyhow!(
                    "failed to load episteme registry from `{}`: {error}",
                    config_file.display()
                )
            },
        );
    }
    load_episteme_registry_from_wendao_toml(cli.root.as_path()).map_err(|error| {
        anyhow::anyhow!(
            "failed to load episteme registry from `{}`: {error}",
            cli.root.display()
        )
    })
}
