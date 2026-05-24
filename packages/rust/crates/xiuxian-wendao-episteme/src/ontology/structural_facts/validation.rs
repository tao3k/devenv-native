use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use xiuxian_wendao_parsers::{EpistemeFileRow, EpistemeSourceManifest};

use super::types::EpistemeOntologyStructuralFactsValidationMode;

pub(super) fn validate_source_manifest(
    domain_id: &str,
    manifest: &EpistemeSourceManifest,
    source_manifest_path: &str,
) -> Result<()> {
    if manifest.domain != domain_id {
        bail!(
            "source manifest `{source_manifest_path}` domain `{}` does not match ontology domain `{domain_id}`",
            manifest.domain
        );
    }
    if manifest.raw_to_rdf_promotion_allowed {
        bail!("source manifest `{source_manifest_path}` cannot allow raw-to-RDF promotion");
    }
    if manifest.copy_raw_files {
        bail!("source manifest `{source_manifest_path}` cannot request raw file copies");
    }
    validate_safe_relative_path(manifest.files.as_str(), "files")?;
    Ok(())
}

pub(super) fn validate_file_row(file: &EpistemeFileRow) -> Result<()> {
    for (field, value) in [
        ("file_id", file.file_id.as_str()),
        ("relative_path", file.relative_path.as_str()),
        ("extension", file.extension.as_str()),
        ("sha256", file.sha256.as_str()),
        ("category", file.category.as_str()),
        ("language", file.language.as_str()),
        ("extraction_route", file.extraction_route.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("files.tsv row for `{}` has blank {field}", file.file_id);
        }
    }
    validate_safe_relative_path(file.relative_path.as_str(), "relative_path")?;
    if file.sha256.len() != 64 || !file.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("files.tsv row for `{}` has invalid sha256", file.file_id);
    }
    Ok(())
}

pub(super) fn validate_source_file(
    source_path: &Path,
    file: &EpistemeFileRow,
    validation_mode: EpistemeOntologyStructuralFactsValidationMode,
) -> Result<()> {
    let metadata = fs::metadata(source_path)
        .with_context(|| format!("failed to access source file `{}`", source_path.display()))?;
    if !metadata.is_file() {
        bail!("source path is not a file: {}", source_path.display());
    }
    if metadata.len() != file.byte_size {
        bail!(
            "byte_size drift for `{}`: expected {}, found {}",
            file.relative_path,
            file.byte_size,
            metadata.len()
        );
    }
    if validation_mode == EpistemeOntologyStructuralFactsValidationMode::FullHash {
        let actual = sha256_file(source_path)?;
        if actual != file.sha256 {
            bail!(
                "sha256 drift for `{}`: expected {}, found {}",
                file.relative_path,
                file.sha256,
                actual
            );
        }
    }
    Ok(())
}

pub(super) fn source_files_path(
    source_manifest_path: &Path,
    files_path: &str,
    source_manifest_label: &str,
) -> Result<PathBuf> {
    validate_safe_relative_path(files_path, "files")?;
    let Some(parent) = source_manifest_path.parent() else {
        bail!("source manifest `{source_manifest_label}` has no parent directory");
    };
    Ok(parent.join(files_path))
}

pub(super) fn parent_components(relative_path: &str) -> Result<Vec<String>> {
    let path = Path::new(relative_path);
    let Some(parent) = path.parent() else {
        return Ok(Vec::new());
    };
    let mut components = Vec::new();
    for component in parent.components() {
        match component {
            Component::Normal(value) => components.push(value.to_string_lossy().to_string()),
            Component::CurDir => {}
            _ => bail!("unsafe parent component in relative_path: {relative_path}"),
        }
    }
    Ok(components)
}

pub(super) fn path_depth(relative_path: &str) -> usize {
    Path::new(relative_path).components().count()
}

pub(super) fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        bail!("invalid run id `{run_id}`; use ASCII letters, digits, '.', '_', or '-'");
    }
    Ok(())
}

fn validate_safe_relative_path(raw: &str, field: &str) -> Result<()> {
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
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let digest = Sha256::digest(&bytes);
    Ok(format!("{digest:x}"))
}
