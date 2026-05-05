use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, DiagnosticRecord, RegisteredRepository, RepoIntelligenceError,
    RepositoryAnalysisOutput, RepositoryRecord,
};

use super::discovery::{
    RepositorySnapshot, collect_doc_records, collect_example_records, collect_import_records,
    collect_module_records, collect_symbol_records,
};
use super::parser_summary::validate_modelica_parser_summary_preflight_for_repository;
use super::parsing::{parse_package_name_for_repository, parse_package_name_lexical};
use super::pathing::modules_by_qualified_name;
use super::relations::collect_relation_records;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelicaRepositoryContext {
    pub(crate) package_root: PathBuf,
    pub(crate) root_package_name: String,
    pub(crate) path_prefix: Option<String>,
}

pub(crate) fn analyze_repository(
    context: &AnalysisContext,
    repository_root: &Path,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let repository_context =
        load_modelica_repository_context(&context.repository, repository_root)?;
    let snapshot = RepositorySnapshot::load(&repository_context.package_root)?;
    let package_files = snapshot.package_files()?;
    let modules = collect_module_records(
        &context.repository.id,
        repository_context.root_package_name.as_str(),
        package_files.as_slice(),
        snapshot.package_orders(),
    );
    let module_lookup = modules_by_qualified_name(&modules);
    let symbols = collect_symbol_records(
        &context.repository,
        &context.repository.id,
        &snapshot,
        repository_context.root_package_name.as_str(),
        &module_lookup,
    )?;
    let imports = collect_import_records(
        &context.repository,
        &context.repository.id,
        &snapshot,
        repository_context.root_package_name.as_str(),
        &module_lookup,
    )?;
    let examples =
        collect_example_records(&context.repository.id, &snapshot, snapshot.package_orders());
    let collected_docs = collect_doc_records(
        &context.repository.id,
        &snapshot,
        repository_context.root_package_name.as_str(),
        &module_lookup,
        &symbols,
        snapshot.package_orders(),
    );
    let docs = collected_docs
        .iter()
        .map(|doc| doc.record.clone())
        .collect::<Vec<_>>();
    let relations = collect_relation_records(
        &context.repository.id,
        repository_context.root_package_name.as_str(),
        &modules,
        &module_lookup,
        &symbols,
        &examples,
        &collected_docs,
    );

    let mut output = RepositoryAnalysisOutput {
        repository: Some(RepositoryRecord {
            repo_id: context.repository.id.clone(),
            name: repository_context.root_package_name,
            path: repository_root.display().to_string(),
            url: context.repository.url.clone(),
            revision: None,
            version: None,
            uuid: None,
            dependencies: Vec::new(),
        }),
        modules,
        symbols,
        imports,
        examples,
        docs,
        relations,
        diagnostics: vec![DiagnosticRecord {
            repo_id: context.repository.id.clone(),
            path: "package.mo".to_string(),
            line: 1,
            message: "Modelica analysis is conservative and currently based on package layout plus lightweight declaration scanning.".to_string(),
            severity: "info".to_string(),
        }],
    };
    prefix_output_paths(&mut output, repository_context.path_prefix.as_deref());
    Ok(output)
}

pub(crate) fn load_modelica_repository_context(
    repository: &RegisteredRepository,
    repository_root: &Path,
) -> Result<ModelicaRepositoryContext, RepoIntelligenceError> {
    load_modelica_repository_context_with_source_hint(repository, repository_root, None)
}

pub(crate) fn load_modelica_repository_context_for_source(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id: &str,
) -> Result<ModelicaRepositoryContext, RepoIntelligenceError> {
    load_modelica_repository_context_with_source_hint(repository, repository_root, Some(source_id))
}

fn load_modelica_repository_context_with_source_hint(
    repository: &RegisteredRepository,
    repository_root: &Path,
    source_id_hint: Option<&str>,
) -> Result<ModelicaRepositoryContext, RepoIntelligenceError> {
    let context = AnalysisContext {
        repository: repository.clone(),
        repository_root: repository_root.to_path_buf(),
    };
    let resolved_root = resolve_modelica_root(&context, repository_root, source_id_hint)?;
    let root_package_path = resolved_root.package_root.join("package.mo");
    let root_package_contents = std::fs::read_to_string(&root_package_path).map_err(|error| {
        RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "failed to read Modelica root package `{}`: {error}",
                root_package_path.display()
            ),
        }
    })?;
    let root_package_source_id = relative_path(repository_root, &root_package_path)
        .unwrap_or_else(|| "package.mo".to_string());
    let root_package_name = parse_package_name_lexical(&root_package_contents)
        .or(parse_package_name_for_repository(
            repository,
            root_package_source_id.as_str(),
            &root_package_contents,
        )?)
        .ok_or_else(|| RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: repository.id.clone(),
            message: "failed to parse root Modelica package name".to_string(),
        })?;

    Ok(ModelicaRepositoryContext {
        package_root: resolved_root.package_root,
        root_package_name,
        path_prefix: resolved_root.path_prefix,
    })
}

pub(crate) fn modelica_root_relative_source_path(
    source_id: &str,
    path_prefix: Option<&str>,
) -> Option<String> {
    let normalized = source_id.replace('\\', "/");
    let Some(prefix) = path_prefix.filter(|prefix| !prefix.is_empty()) else {
        return Some(normalized);
    };
    let relative = normalized.strip_prefix(prefix)?;
    let relative = relative.strip_prefix('/').unwrap_or(relative);
    if relative.is_empty() {
        None
    } else {
        Some(relative.to_string())
    }
}

pub(crate) fn preflight_repository(
    context: &AnalysisContext,
    repository_root: &Path,
) -> Result<(), RepoIntelligenceError> {
    validate_modelica_parser_summary_preflight_for_repository(&context.repository)?;
    resolve_modelica_root(context, repository_root, None).map(|_| ())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedModelicaRoot {
    package_root: PathBuf,
    path_prefix: Option<String>,
}

fn resolve_modelica_root(
    context: &AnalysisContext,
    repository_root: &Path,
    source_id_hint: Option<&str>,
) -> Result<ResolvedModelicaRoot, RepoIntelligenceError> {
    let root_package_path = repository_root.join("package.mo");
    if root_package_path.is_file() {
        return Ok(ResolvedModelicaRoot {
            package_root: repository_root.to_path_buf(),
            path_prefix: None,
        });
    }

    if let Some(hinted_root) = hinted_nested_package_root(repository_root, source_id_hint) {
        return Ok(hinted_root);
    }

    let nested_candidates = nested_package_root_candidates(context, repository_root)?;
    match nested_candidates.as_slice() {
        [] => Err(RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: context.repository.id.clone(),
            message: "expected a Modelica repository root package.mo or a dominant top-level package directory".to_string(),
        }),
        [candidate] => Ok(ResolvedModelicaRoot {
            package_root: candidate.package_root.clone(),
            path_prefix: Some(candidate.path_prefix.clone()),
        }),
        [first, second, ..] if first.modelica_file_count > second.modelica_file_count => Ok(
            ResolvedModelicaRoot {
                package_root: first.package_root.clone(),
                path_prefix: Some(first.path_prefix.clone()),
            },
        ),
        _ => Err(RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: context.repository.id.clone(),
            message: "found multiple top-level Modelica package roots without a dominant package".to_string(),
        }),
    }
}

fn hinted_nested_package_root(
    repository_root: &Path,
    source_id_hint: Option<&str>,
) -> Option<ResolvedModelicaRoot> {
    let source_id = source_id_hint?;
    let normalized = source_id.trim().replace('\\', "/");
    let prefix = normalized.split('/').next()?.trim();
    if prefix.is_empty() || prefix == "package.mo" {
        return None;
    }

    let package_root = repository_root.join(prefix);
    package_root
        .join("package.mo")
        .is_file()
        .then(|| ResolvedModelicaRoot {
            package_root,
            path_prefix: Some(prefix.to_string()),
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NestedPackageCandidate {
    package_root: PathBuf,
    path_prefix: String,
    modelica_file_count: usize,
}

fn nested_package_root_candidates(
    context: &AnalysisContext,
    repository_root: &Path,
) -> Result<Vec<NestedPackageCandidate>, RepoIntelligenceError> {
    let mut candidates = fs::read_dir(repository_root)
        .map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "failed to enumerate Modelica repository root `{}`: {error}",
                repository_root.display()
            ),
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("package.mo").is_file())
        .filter_map(|package_root| {
            let path_prefix = package_root.file_name()?.to_str()?.to_string();
            Some(NestedPackageCandidate {
                modelica_file_count: count_modelica_files(package_root.as_path()),
                package_root,
                path_prefix,
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .modelica_file_count
            .cmp(&left.modelica_file_count)
            .then_with(|| left.path_prefix.cmp(&right.path_prefix))
    });

    if candidates.is_empty() {
        return Err(RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: context.repository.id.clone(),
            message: "expected a Modelica repository root package.mo or a dominant top-level package directory".to_string(),
        });
    }

    Ok(candidates)
}

fn count_modelica_files(root: &Path) -> usize {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(std::ffi::OsStr::to_str) == Some("mo"))
        .count()
}

fn relative_path(repository_root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(repository_root)
        .ok()
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
}

fn prefix_output_paths(output: &mut RepositoryAnalysisOutput, prefix: Option<&str>) {
    let Some(prefix) = prefix.filter(|prefix| !prefix.is_empty()) else {
        return;
    };

    for module in &mut output.modules {
        module.path = prefixed_relative_path(prefix, module.path.as_str());
    }
    for symbol in &mut output.symbols {
        symbol.path = prefixed_relative_path(prefix, symbol.path.as_str());
    }
    for example in &mut output.examples {
        example.path = prefixed_relative_path(prefix, example.path.as_str());
    }
    for import in &mut output.imports {
        import.path = prefixed_relative_path(prefix, import.path.as_str());
    }
    for doc in &mut output.docs {
        doc.path = prefixed_relative_path(prefix, doc.path.as_str());
    }
    for diagnostic in &mut output.diagnostics {
        diagnostic.path = prefixed_relative_path(prefix, diagnostic.path.as_str());
    }
}

fn prefixed_relative_path(prefix: &str, path: &str) -> String {
    if path.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}/{path}")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/analysis.rs"]
mod tests;
