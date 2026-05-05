//! Incremental-safety helpers for Modelica repository analysis.

use std::path::{Path, PathBuf};

use serde::Serialize;
use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, DiagnosticRecord, DocRecord, ImportRecord, ModuleRecord, PluginAnalysisOutput,
    RegisteredRepository, RepoIntelligenceError, RepoSourceFile, SymbolRecord,
};

use super::analysis::{
    load_modelica_repository_context_for_source, modelica_root_relative_source_path,
};
use super::discovery::{is_api_surface_path, safe_package_overlay_metadata_for_relative_path};
use super::parser_summary::fetch_modelica_parser_file_summary_blocking_for_repository;
use super::parsing::{
    RootPackageOverlayMetadata, contains_documentation_annotation,
    parse_package_name_for_repository, parse_safe_root_package_overlay_metadata,
};
use super::pathing::{containing_module_name, qualified_module_name};
use super::relations::annotation_doc_title;
use super::types::{ModelicaSourceId, ParsedDeclaration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepoOwnedModelicaIncrementalKind {
    LeafApi,
    PackageFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepoOwnedModelicaIncrementalContext {
    kind: RepoOwnedModelicaIncrementalKind,
    module_qualified_name: String,
    module_id: String,
    package_root: PathBuf,
    relative_within_root: String,
    root_package_name: String,
}

/// Return whether one Modelica source file can use the bounded leaf-only
/// incremental overlay path for repository analysis.
///
/// Safe incremental files must stay on the API surface, avoid `package.mo`,
/// and avoid documentation annotations so the overlay can replace only
/// leaf-local symbol and import rows without rebuilding repository-wide
/// structure.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository root discovery or the
/// linked parser-summary request fails.
pub fn modelica_parser_summary_allows_safe_incremental_file_for_repository(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: ModelicaSourceId<'_>,
    source_text: &str,
) -> Result<bool, RepoIntelligenceError> {
    let source_id = source_id.as_str();
    Ok(resolve_safe_modelica_incremental_context(
        repository,
        repository_root,
        source_id,
        source_text,
    )?
    .is_some())
}

/// Return whether one Modelica source file can use the bounded root
/// `package.mo` incremental overlay path for repository analysis.
///
/// This owner is intentionally narrower than general package-file support: it
/// only admits the repository root `package.mo`, which can update root-local
/// module/symbol/import/doc rows without forcing a repository-wide rebuild as
/// long as the root package identity itself stays stable.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository root discovery or the
/// linked parser-summary request fails.
pub fn modelica_parser_summary_allows_safe_root_package_incremental_file_for_repository(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: ModelicaSourceId<'_>,
    source_text: &str,
) -> Result<bool, RepoIntelligenceError> {
    let source_id = source_id.as_str();
    Ok(resolve_safe_root_package_modelica_incremental_context(
        repository,
        repository_root,
        source_id,
        source_text,
    )?
    .is_some())
}

/// Return whether one Modelica `package.mo` file can use the bounded package
/// incremental overlay path for repository analysis.
///
/// This owner admits API-surface package files whose package identity stays
/// aligned with the repository path and whose contents remain inside the
/// bounded import/doc-only contract.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository root discovery fails.
pub fn modelica_parser_summary_allows_safe_package_incremental_file_for_repository(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: ModelicaSourceId<'_>,
    source_text: &str,
) -> Result<bool, RepoIntelligenceError> {
    let source_id = source_id.as_str();
    Ok(resolve_safe_package_file_modelica_incremental_context(
        repository,
        repository_root,
        source_id,
        source_text,
    )?
    .is_some())
}

/// Return whether the parsed package name of a root `package.mo` still matches
/// the currently resolved repository root package identity.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository root discovery or the
/// linked parser-summary request fails.
pub fn modelica_parser_summary_root_package_name_matches_repository_context(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: ModelicaSourceId<'_>,
    source_text: &str,
) -> Result<bool, RepoIntelligenceError> {
    let source_id = source_id.as_str();
    let Some(context) = resolve_safe_root_package_modelica_incremental_context(
        repository,
        repository_root,
        source_id,
        source_text,
    )?
    else {
        return Ok(false);
    };
    let metadata = parse_safe_root_package_overlay_metadata(source_text).or_else(|| {
        parse_package_name_for_repository(repository, source_id, source_text)
            .ok()
            .flatten()
            .map(|package_name| RootPackageOverlayMetadata {
                package_name,
                imports: Vec::new(),
                has_documentation_annotation: contains_documentation_annotation(source_text),
            })
    });
    Ok(metadata.is_some_and(|metadata| metadata.package_name == context.root_package_name))
}

/// Return a stable bounded semantic fingerprint for a safe root `package.mo`
/// incremental overlay.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository root discovery fails.
pub fn modelica_root_package_incremental_semantic_fingerprint_for_repository(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: ModelicaSourceId<'_>,
    source_text: &str,
) -> Result<Option<String>, RepoIntelligenceError> {
    let source_id = source_id.as_str();
    let Some(context) = resolve_safe_root_package_modelica_incremental_context(
        repository,
        repository_root,
        source_id,
        source_text,
    )?
    else {
        return Ok(None);
    };
    let Some(metadata) = parse_safe_root_package_overlay_metadata(source_text) else {
        return Ok(None);
    };
    Ok(Some(stable_payload_fingerprint(
        "modelica_root_package_incremental_overlay",
        &PackageOverlayFingerprint {
            package_name: metadata.package_name,
            imports: package_overlay_fingerprint_imports(metadata.imports.as_slice()),
            has_documentation_annotation: metadata.has_documentation_annotation,
            module_qualified_name: context.module_qualified_name,
        },
    )))
}

/// Return a stable bounded semantic fingerprint for a safe Modelica
/// `package.mo` incremental overlay.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository root discovery fails.
pub fn modelica_package_incremental_semantic_fingerprint_for_repository(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: ModelicaSourceId<'_>,
    source_text: &str,
) -> Result<Option<String>, RepoIntelligenceError> {
    let source_id = source_id.as_str();
    let Some(context) = resolve_safe_package_file_modelica_incremental_context(
        repository,
        repository_root,
        source_id,
        source_text,
    )?
    else {
        return Ok(None);
    };
    let Some(metadata) = safe_package_overlay_metadata_for_relative_path(
        context.relative_within_root.as_str(),
        source_text,
        context.root_package_name.as_str(),
    ) else {
        return Ok(None);
    };
    Ok(Some(stable_payload_fingerprint(
        "modelica_package_incremental_overlay",
        &PackageOverlayFingerprint {
            package_name: metadata.package_name,
            imports: package_overlay_fingerprint_imports(metadata.imports.as_slice()),
            has_documentation_annotation: metadata.has_documentation_annotation,
            module_qualified_name: context.module_qualified_name,
        },
    )))
}

pub(crate) fn analyze_repo_owned_modelica_file_for_repository(
    context: &AnalysisContext,
    file: &RepoSourceFile,
) -> Result<Option<PluginAnalysisOutput>, RepoIntelligenceError> {
    let Some(incremental_context) = resolve_repo_owned_modelica_incremental_context(
        &context.repository,
        context.repository_root.as_path(),
        file.path.as_str(),
        file.contents.as_str(),
    )?
    else {
        return Ok(None);
    };

    let output = match incremental_context.kind {
        RepoOwnedModelicaIncrementalKind::LeafApi => {
            build_repo_owned_leaf_overlay(context, file, &incremental_context)?
        }
        RepoOwnedModelicaIncrementalKind::PackageFile => {
            build_repo_owned_package_overlay(context, file, &incremental_context)?
        }
    };
    Ok(Some(output))
}

fn build_repo_owned_leaf_overlay(
    context: &AnalysisContext,
    file: &RepoSourceFile,
    incremental_context: &RepoOwnedModelicaIncrementalContext,
) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
    let summary = fetch_modelica_parser_file_summary_blocking_for_repository(
        &context.repository,
        file.path.as_str(),
        file.contents.as_str(),
    )?;
    let imports = if summary.imports.is_empty() {
        Vec::new()
    } else {
        build_import_records_without_resolution(
            &context.repository.id,
            file.path.as_str(),
            &incremental_context.module_id,
            summary.imports.as_slice(),
        )
    };
    Ok(PluginAnalysisOutput {
        modules: Vec::new(),
        symbols: summary
            .declarations
            .into_iter()
            .map(|declaration| {
                build_repo_owned_modelica_symbol_record(
                    &context.repository.id,
                    &file.path,
                    &incremental_context.module_qualified_name,
                    &incremental_context.module_id,
                    declaration,
                )
            })
            .collect::<Vec<_>>(),
        imports,
        examples: Vec::new(),
        docs: Vec::new(),
        diagnostics: Vec::new(),
    })
}

fn build_repo_owned_package_overlay(
    context: &AnalysisContext,
    file: &RepoSourceFile,
    incremental_context: &RepoOwnedModelicaIncrementalContext,
) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
    let metadata = safe_package_overlay_metadata_for_relative_path(
        incremental_context.relative_within_root.as_str(),
        file.contents.as_str(),
        incremental_context.root_package_name.as_str(),
    )
    .ok_or_else(|| RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Modelica package `{}` no longer qualifies for bounded incremental overlay",
            file.path
        ),
    })?;
    Ok(PluginAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: context.repository.id.clone(),
            module_id: incremental_context.module_id.clone(),
            qualified_name: incremental_context.module_qualified_name.clone(),
            path: file.path.clone(),
        }],
        symbols: Vec::new(),
        imports: build_import_records_without_resolution(
            &context.repository.id,
            &file.path,
            &incremental_context.module_id,
            &metadata.imports,
        ),
        examples: Vec::new(),
        docs: build_package_doc_records(
            &context.repository.id,
            &file.path,
            metadata.has_documentation_annotation,
        ),
        diagnostics: vec![DiagnosticRecord {
            repo_id: context.repository.id.clone(),
            path: file.path.clone(),
            line: 1,
            message:
                "Modelica analysis is conservative and currently based on package layout plus lightweight declaration scanning."
                    .to_string(),
            severity: "info".to_string(),
        }],
    })
}

fn resolve_repo_owned_modelica_incremental_context(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: &str,
    source_text: &str,
) -> Result<Option<RepoOwnedModelicaIncrementalContext>, RepoIntelligenceError> {
    if let Some(context) = resolve_safe_modelica_incremental_context(
        repository,
        repository_root,
        source_id,
        source_text,
    )? {
        return Ok(Some(context));
    }
    resolve_safe_package_file_modelica_incremental_context(
        repository,
        repository_root,
        source_id,
        source_text,
    )
}

fn resolve_safe_modelica_incremental_context(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: &str,
    source_text: &str,
) -> Result<Option<RepoOwnedModelicaIncrementalContext>, RepoIntelligenceError> {
    if !has_modelica_file_extension(source_id)
        || Path::new(source_id)
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            == Some("package.mo")
        || !is_api_surface_path(source_id)
        || contains_documentation_annotation(source_text)
    {
        return Ok(None);
    }

    let repository_context =
        match load_modelica_repository_context_for_source(repository, repository_root, source_id) {
            Ok(context) => context,
            Err(RepoIntelligenceError::UnsupportedRepositoryLayout { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
    let Some(relative_within_root) =
        modelica_root_relative_source_path(source_id, repository_context.path_prefix.as_deref())
    else {
        return Ok(None);
    };
    let Some(module_qualified_name) = containing_module_name(
        repository_context.root_package_name.as_str(),
        relative_within_root.as_str(),
    ) else {
        return Ok(None);
    };

    Ok(Some(RepoOwnedModelicaIncrementalContext {
        kind: RepoOwnedModelicaIncrementalKind::LeafApi,
        module_id: format!("repo:{}:module:{module_qualified_name}", repository.id),
        module_qualified_name,
        package_root: repository_context.package_root,
        relative_within_root,
        root_package_name: repository_context.root_package_name,
    }))
}

fn has_modelica_file_extension(source_id: &str) -> bool {
    Path::new(source_id)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mo"))
}

fn resolve_safe_root_package_modelica_incremental_context(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: &str,
    source_text: &str,
) -> Result<Option<RepoOwnedModelicaIncrementalContext>, RepoIntelligenceError> {
    if !source_id.ends_with("package.mo") {
        return Ok(None);
    }
    if parse_safe_root_package_overlay_metadata(source_text).is_none() {
        return Ok(None);
    }

    let repository_context =
        match load_modelica_repository_context_for_source(repository, repository_root, source_id) {
            Ok(context) => context,
            Err(RepoIntelligenceError::UnsupportedRepositoryLayout { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
    let Some(relative_within_root) =
        modelica_root_relative_source_path(source_id, repository_context.path_prefix.as_deref())
    else {
        return Ok(None);
    };
    if relative_within_root != "package.mo" {
        return Ok(None);
    }
    let Some(module_qualified_name) = qualified_module_name(
        repository_context.root_package_name.as_str(),
        relative_within_root.as_str(),
    ) else {
        return Ok(None);
    };

    Ok(Some(RepoOwnedModelicaIncrementalContext {
        kind: RepoOwnedModelicaIncrementalKind::PackageFile,
        module_id: format!("repo:{}:module:{module_qualified_name}", repository.id),
        module_qualified_name,
        package_root: repository_context.package_root,
        relative_within_root,
        root_package_name: repository_context.root_package_name,
    }))
}

fn resolve_safe_package_file_modelica_incremental_context(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: &str,
    source_text: &str,
) -> Result<Option<RepoOwnedModelicaIncrementalContext>, RepoIntelligenceError> {
    if !source_id.ends_with("package.mo") {
        return Ok(None);
    }

    let repository_context =
        match load_modelica_repository_context_for_source(repository, repository_root, source_id) {
            Ok(context) => context,
            Err(RepoIntelligenceError::UnsupportedRepositoryLayout { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
    let Some(relative_within_root) =
        modelica_root_relative_source_path(source_id, repository_context.path_prefix.as_deref())
    else {
        return Ok(None);
    };
    if !is_api_surface_path(relative_within_root.as_str()) {
        return Ok(None);
    }
    if safe_package_overlay_metadata_for_relative_path(
        relative_within_root.as_str(),
        source_text,
        repository_context.root_package_name.as_str(),
    )
    .is_none()
    {
        return Ok(None);
    }
    let Some(module_qualified_name) = qualified_module_name(
        repository_context.root_package_name.as_str(),
        relative_within_root.as_str(),
    ) else {
        return Ok(None);
    };

    Ok(Some(RepoOwnedModelicaIncrementalContext {
        kind: RepoOwnedModelicaIncrementalKind::PackageFile,
        module_id: format!("repo:{}:module:{module_qualified_name}", repository.id),
        module_qualified_name,
        package_root: repository_context.package_root,
        relative_within_root,
        root_package_name: repository_context.root_package_name,
    }))
}

fn build_repo_owned_modelica_symbol_record(
    repo_id: &str,
    path: &str,
    module_qualified_name: &str,
    module_id: &str,
    declaration: ParsedDeclaration,
) -> SymbolRecord {
    let qualified_name = format!("{module_qualified_name}.{}", declaration.name);
    SymbolRecord {
        repo_id: repo_id.to_string(),
        symbol_id: format!("repo:{repo_id}:symbol:{qualified_name}"),
        module_id: Some(module_id.to_string()),
        name: declaration.name,
        qualified_name,
        kind: declaration.kind,
        path: path.to_string(),
        line_start: declaration.line_start,
        line_end: declaration.line_end,
        signature: Some(declaration.signature),
        audit_status: None,
        verification_state: None,
        attributes: declaration.attributes,
    }
}

fn build_package_doc_records(
    repo_id: &str,
    record_path: &str,
    has_documentation_annotation: bool,
) -> Vec<DocRecord> {
    if !has_documentation_annotation {
        return Vec::new();
    }

    vec![DocRecord {
        repo_id: repo_id.to_string(),
        doc_id: format!("repo:{repo_id}:doc:{record_path}#annotation.documentation"),
        title: annotation_doc_title(record_path, &[]),
        path: format!("{record_path}#annotation.documentation"),
        format: Some("modelica_annotation".to_string()),
        doc_target: None,
    }]
}

fn build_import_records_without_resolution(
    repo_id: &str,
    record_path: &str,
    module_id: &str,
    imports: &[super::types::ParsedImport],
) -> Vec<ImportRecord> {
    let module_lookup = std::collections::BTreeMap::<String, ModuleRecord>::new();
    build_import_records_from_parsed_imports(
        repo_id,
        record_path,
        module_id,
        imports,
        &module_lookup,
    )
}

fn build_import_records_from_parsed_imports(
    repo_id: &str,
    record_path: &str,
    module_id: &str,
    imports: &[super::types::ParsedImport],
    module_lookup: &std::collections::BTreeMap<String, ModuleRecord>,
) -> Vec<ImportRecord> {
    let mut records = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for parsed_import in imports {
        let source_module = parsed_import.name.clone();
        let import_name = parsed_import
            .alias
            .clone()
            .unwrap_or_else(|| import_leaf_name(source_module.as_str()));
        let target_package = source_module
            .split('.')
            .next()
            .unwrap_or(source_module.as_str())
            .to_string();
        let kind_key = match parsed_import.kind {
            xiuxian_wendao_core::repo_intelligence::ImportKind::Symbol => "symbol",
            xiuxian_wendao_core::repo_intelligence::ImportKind::Module => "module",
            xiuxian_wendao_core::repo_intelligence::ImportKind::Reexport => "reexport",
        };
        let import_key = (
            record_path.to_string(),
            source_module.clone(),
            import_name.clone(),
            kind_key,
        );
        if !seen.insert(import_key) {
            continue;
        }
        records.push(ImportRecord {
            repo_id: repo_id.to_string(),
            module_id: module_id.to_string(),
            path: record_path.to_string(),
            import_name,
            target_package,
            source_module: source_module.clone(),
            kind: parsed_import.kind,
            line_start: parsed_import.line_start,
            resolved_id: module_lookup
                .get(source_module.as_str())
                .map(|module| module.module_id.clone()),
            attributes: parsed_import.attributes.clone(),
        });
    }

    records
}

fn import_leaf_name(import_path: &str) -> String {
    import_path
        .rsplit('.')
        .next()
        .unwrap_or(import_path)
        .trim()
        .to_string()
}

#[derive(Serialize)]
struct PackageOverlayFingerprint {
    package_name: String,
    imports: Vec<PackageOverlayFingerprintImport>,
    has_documentation_annotation: bool,
    module_qualified_name: String,
}

#[derive(Serialize)]
struct PackageOverlayFingerprintImport {
    name: String,
    alias: Option<String>,
    kind: xiuxian_wendao_core::repo_intelligence::ImportKind,
    attributes: std::collections::BTreeMap<String, String>,
}

fn package_overlay_fingerprint_imports(
    imports: &[super::types::ParsedImport],
) -> Vec<PackageOverlayFingerprintImport> {
    imports
        .iter()
        .map(|import| PackageOverlayFingerprintImport {
            name: import.name.clone(),
            alias: import.alias.clone(),
            kind: import.kind,
            attributes: import.attributes.clone(),
        })
        .collect()
}

fn stable_payload_fingerprint<T: Serialize + ?Sized>(kind: &str, value: &T) -> String {
    let payload = serde_json::to_vec(value).unwrap_or_else(|error| {
        panic!("Modelica incremental overlay payload should serialize: {error}");
    });
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.as_bytes());
    hasher.update(&payload);
    hasher.finalize().to_hex().to_string()
}
