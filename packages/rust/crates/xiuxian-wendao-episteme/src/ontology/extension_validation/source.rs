use std::{collections::BTreeSet, fs};

use anyhow::{Context, Result, bail};
use xiuxian_wendao_parsers::{
    EpistemeExtractionQueueRow, EpistemeFileRow, EpistemeSourceManifest,
    parse_episteme_extraction_queue_tsv, parse_episteme_files_tsv,
    parse_episteme_source_manifest_toml,
};

use crate::source_contract::EpistemeSourceContractPaths;

use super::{
    model::{EpistemeExtensionValidationMode, EpistemeExtensionValidationRequest},
    pathing::{
        read_to_string, resolve_beside, resolve_corpus_root, safe_relative_path, sha256_file,
    },
};

pub(super) struct ExtensionSourceValidationReport {
    pub(super) source_manifests: usize,
    pub(super) source_files: usize,
    pub(super) extraction_queue_rows: usize,
}

pub(super) fn validate_extension_sources(
    request: &EpistemeExtensionValidationRequest,
    paths: &EpistemeSourceContractPaths,
) -> Result<ExtensionSourceValidationReport> {
    let source_manifest_path = paths.source_manifest_path(request.episteme_root());
    let manifest_raw = read_to_string(source_manifest_path.as_path())?;
    let manifest = parse_episteme_source_manifest_toml(manifest_raw.as_str())
        .with_context(|| format!("failed to parse `{}`", source_manifest_path.display()))?;
    validate_source_manifest(paths.domain_id(), &manifest)?;

    let files_path = resolve_beside(
        source_manifest_path.as_path(),
        manifest.files.as_str(),
        "files",
    )?;
    let queue_path = resolve_beside(
        source_manifest_path.as_path(),
        manifest.extraction_queue.as_str(),
        "extraction_queue",
    )?;
    let files = parse_episteme_files_tsv(read_to_string(files_path.as_path())?.as_str())
        .with_context(|| format!("failed to parse `{}`", files_path.display()))?;
    let queue = parse_episteme_extraction_queue_tsv(read_to_string(queue_path.as_path())?.as_str())
        .with_context(|| format!("failed to parse `{}`", queue_path.display()))?;
    let corpus_root = resolve_corpus_root(request, &manifest)?;
    validate_files(&manifest, &files, &corpus_root, request.validation_mode())?;
    validate_queue(&files, &queue)?;

    Ok(ExtensionSourceValidationReport {
        source_manifests: 1,
        source_files: files.len(),
        extraction_queue_rows: queue.len(),
    })
}

fn validate_source_manifest(domain_id: &str, manifest: &EpistemeSourceManifest) -> Result<()> {
    if manifest.schema_version != 1 {
        bail!(
            "unsupported source manifest schema_version: {}",
            manifest.schema_version
        );
    }
    if manifest.domain != domain_id {
        bail!(
            "source manifest domain `{}` does not match selected domain `{domain_id}`",
            manifest.domain
        );
    }
    if manifest.primary_language.trim().is_empty() {
        bail!("extension source manifests must declare primary_language");
    }
    if manifest.copy_raw_files {
        bail!("extension source manifests must not request raw file copies");
    }
    if manifest.raw_to_rdf_promotion_allowed {
        bail!("extension source manifests must not allow raw-to-RDF promotion");
    }
    safe_relative_path(manifest.files.as_str(), "files")?;
    safe_relative_path(manifest.extraction_queue.as_str(), "extraction_queue")?;
    if manifest.routes.is_empty() {
        bail!("extension source manifests must declare at least one extraction route");
    }
    Ok(())
}

fn validate_files(
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
    corpus_root: &std::path::Path,
    validation_mode: EpistemeExtensionValidationMode,
) -> Result<()> {
    let mut file_ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for file in files {
        validate_file_row(manifest, file)?;
        if !file_ids.insert(file.file_id.as_str()) {
            bail!("duplicate file_id in files.tsv: {}", file.file_id);
        }
        if !paths.insert(file.relative_path.as_str()) {
            bail!(
                "duplicate relative_path in files.tsv: {}",
                file.relative_path
            );
        }
        let source_path = corpus_root.join(file.relative_path.as_str());
        let metadata = fs::metadata(source_path.as_path())
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
        if validation_mode == EpistemeExtensionValidationMode::FullHash {
            let actual = sha256_file(source_path.as_path())?;
            if actual != file.sha256 {
                bail!(
                    "sha256 drift for `{}`: expected {}, found {}",
                    file.relative_path,
                    file.sha256,
                    actual
                );
            }
        }
    }
    Ok(())
}

fn validate_file_row(manifest: &EpistemeSourceManifest, file: &EpistemeFileRow) -> Result<()> {
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
    safe_relative_path(file.relative_path.as_str(), "relative_path")?;
    if file.extension != file.extension.to_ascii_lowercase() {
        bail!(
            "files.tsv row for `{}` extension must be lowercase",
            file.file_id
        );
    }
    if file.sha256.len() != 64 || !file.sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
        bail!("files.tsv row for `{}` has invalid sha256", file.file_id);
    }
    if file.language != manifest.primary_language {
        bail!(
            "files.tsv row for `{}` language `{}` does not match source primary_language `{}`",
            file.file_id,
            file.language,
            manifest.primary_language
        );
    }
    let Some(allowed_extensions) = manifest.routes.get(file.extraction_route.as_str()) else {
        bail!(
            "files.tsv row for `{}` uses unknown extraction_route `{}`",
            file.file_id,
            file.extraction_route
        );
    };
    if !allowed_extensions
        .iter()
        .any(|extension| extension == &file.extension)
    {
        bail!(
            "files.tsv row for `{}` extension `{}` is not allowed by route `{}`",
            file.file_id,
            file.extension,
            file.extraction_route
        );
    }
    Ok(())
}

fn validate_queue(files: &[EpistemeFileRow], queue: &[EpistemeExtractionQueueRow]) -> Result<()> {
    let files_by_id = files
        .iter()
        .map(|file| (file.file_id.as_str(), file))
        .collect::<std::collections::BTreeMap<_, _>>();
    let file_ids = files_by_id.keys().copied().collect::<BTreeSet<_>>();
    let mut queued_file_ids = BTreeSet::new();
    let mut queue_ids = BTreeSet::new();
    for row in queue {
        if !queue_ids.insert(row.queue_id.as_str()) {
            bail!(
                "duplicate queue_id in extraction_queue.tsv: {}",
                row.queue_id
            );
        }
        if !file_ids.contains(row.file_id.as_str()) {
            bail!(
                "extraction_queue.tsv row `{}` references unknown file_id `{}`",
                row.queue_id,
                row.file_id
            );
        }
        let Some(file) = files_by_id.get(row.file_id.as_str()) else {
            bail!(
                "extraction_queue.tsv row `{}` references unknown file_id `{}`",
                row.queue_id,
                row.file_id
            );
        };
        if row.relative_path != file.relative_path
            || row.category != file.category
            || row.language != file.language
            || row.extraction_route != file.extraction_route
        {
            bail!(
                "extraction_queue.tsv row `{}` does not match files.tsv metadata for `{}`",
                row.queue_id,
                row.file_id
            );
        }
        if !row.output_contract.contains("no_rdf_promotion") {
            bail!(
                "extraction_queue.tsv row `{}` output_contract must preserve no-RDF-promotion policy",
                row.queue_id
            );
        }
        if row.status.trim().is_empty() {
            bail!(
                "extraction_queue.tsv row `{}` status must not be blank",
                row.queue_id
            );
        }
        queued_file_ids.insert(row.file_id.as_str());
    }
    if queued_file_ids.len() != file_ids.len() {
        bail!(
            "extraction_queue.tsv must include one row for every files.tsv file: queued {}, files {}",
            queued_file_ids.len(),
            file_ids.len()
        );
    }
    Ok(())
}
