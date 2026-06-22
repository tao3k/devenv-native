use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use xiuxian_wendao_parsers::EpistemeSourceManifest;

use crate::load_episteme_runtime_config;

use super::model::EpistemeExtensionValidationRequest;

pub(super) fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))
}

pub(super) fn resolve_corpus_root(
    request: &EpistemeExtensionValidationRequest,
    source_manifest: &EpistemeSourceManifest,
) -> Result<PathBuf> {
    if let Some(corpus_root) = request.corpus_root() {
        return Ok(corpus_root.to_path_buf());
    }
    if let Ok(value) = env::var(source_manifest.corpus_root_env.as_str())
        && !value.trim().is_empty()
    {
        return Ok(PathBuf::from(value));
    }
    let Some(config) = load_episteme_runtime_config(request.episteme_root())
        .with_context(|| "failed to load Episteme runtime config")?
    else {
        bail!(
            "runtime.corpus_root is required in episteme.toml when --corpus-root and {} are unset",
            source_manifest.corpus_root_env
        );
    };
    config.corpus.with_context(|| {
        format!(
            "runtime.corpus_root is required in episteme.toml when --corpus-root and {} are unset",
            source_manifest.corpus_root_env
        )
    })
}

pub(super) fn resolve_beside(parent_file: &Path, raw: &str, field: &str) -> Result<PathBuf> {
    let Some(parent) = parent_file.parent() else {
        bail!("`{}` has no parent directory", parent_file.display());
    };
    Ok(parent.join(safe_relative_path(raw, field)?))
}

pub(super) fn safe_relative_path(raw: &str, field: &str) -> Result<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("{field} must not be blank");
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        bail!("{field} must be a safe relative path: {trimmed}");
    }
    Ok(path.to_path_buf())
}

pub(super) fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let digest = Sha256::digest(&bytes);
    Ok(format!("{digest:x}"))
}

pub(super) fn has_cjk(value: &str) -> bool {
    value.chars().any(|ch| {
        matches!(
            ch as u32,
            0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
        )
    })
}
