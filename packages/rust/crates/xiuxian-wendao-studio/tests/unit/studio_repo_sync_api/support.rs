use std::collections::BTreeMap;
use std::fs;
use std::io::Error as IoError;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::UNIX_EPOCH;

use axum::body::{Body, to_bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::util::ServiceExt;

use crate::contracts::UiConfig;
use crate::studio::search::handlers::tests::linked_parser_summary::ensure_linked_modelica_parser_summary_service;
use crate::studio::symbol_index::SymbolIndexCoordinator;
use crate::studio::test_support::{add_git_remote, commit_all, init_git_repository};
use crate::studio::{GatewayState, StudioState};
use xiuxian_git_repo::{SyncMode, discover_checkout_metadata};
use xiuxian_wendao::analyzers::resolve_registered_repository_source;
use xiuxian_wendao::analyzers::{
    DocRecord, ExampleRecord, ModuleRecord, ProjectedPageIndexNode, ProjectionPageKind,
    RelationKind, RelationRecord, RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery,
    RepoSymbolKind, RepositoryAnalysisOutput, RepositoryRecord, SymbolRecord,
    analyze_registered_repository_with_registry, build_repository_analysis_cache_key,
    load_repo_intelligence_config, repo_projected_page_index_trees_from_config,
    repo_projected_pages_from_config, store_cached_repository_analysis,
};
use xiuxian_wendao::repo_index::RepoCodeDocument;
use xiuxian_wendao::repo_index::RepoIndexCoordinator;
use xiuxian_wendao::search::SearchPlaneService;

use super::LocalProjectMetadata;

pub(super) async fn request_json(
    router: axum::Router,
    uri: &str,
) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload = serde_json::from_slice(&body)?;
    Ok((status, payload))
}

pub(super) async fn request_json_post<T: serde::Serialize>(
    router: axum::Router,
    uri: &str,
    payload: &T,
) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(payload)?))?,
        )
        .await?;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload = serde_json::from_slice(&body)?;
    Ok((status, payload))
}

pub(super) fn page_matches_needle(page: &serde_json::Map<String, Value>, needle: &str) -> bool {
    let title = page
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let page_id = page
        .get("page_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    title.contains(needle) || page_id.contains(needle)
}

pub(super) fn node_matches_needle(node: &serde_json::Map<String, Value>, needle: &str) -> bool {
    let node_title = node
        .get("node_title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let page_title = node
        .get("page_title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    node_title.contains(needle) || page_title.contains(needle)
}

pub(super) fn gateway_state_for_project(project_root: &Path) -> Arc<GatewayState> {
    gateway_state_for_project_with_options(project_root, true, true)
}

pub(super) fn gateway_state_for_ui_config(
    project_root: &Path,
    ui_config: UiConfig,
    plugin_registry: Arc<xiuxian_wendao::analyzers::PluginRegistry>,
) -> Arc<GatewayState> {
    let repo_index = Arc::new(RepoIndexCoordinator::new(
        project_root.to_path_buf(),
        Arc::clone(&plugin_registry),
        xiuxian_wendao::search::SearchPlaneService::new(project_root.to_path_buf()),
    ));
    repo_index.start();

    Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState {
            project_root: project_root.to_path_buf(),
            config_root: project_root.to_path_buf(),
            bootstrap_background_indexing: false,
            cold_start_process_started_at: crate::studio::symbol_index::timestamp_now(),
            cold_start_process_started_instant: std::time::Instant::now(),
            cold_start_telemetry: Arc::new(RwLock::new(
                crate::studio::router::StudioSearchColdStartTelemetryState::default(),
            )),
            bootstrap_background_indexing_deferred_activation: Arc::new(RwLock::new(None)),
            configured_owners: Arc::new(RwLock::new(
                StudioState::configured_owners_from_ui_config(ui_config),
            )),
            graph_index: Arc::new(RwLock::new(None)),
            symbol_index: Arc::new(RwLock::new(None)),
            symbol_index_coordinator: Arc::new(SymbolIndexCoordinator::new(
                project_root.to_path_buf(),
                project_root.to_path_buf(),
                SearchPlaneService::new(project_root.to_path_buf()),
            )),
            local_corpus_scan_coalescing: Arc::new(RwLock::new(
                crate::studio::router::LocalCorpusScanCoalescingState::default(),
            )),
            search_plane: SearchPlaneService::new(project_root.to_path_buf()),
            vfs_scan: Arc::new(RwLock::new(None)),
            repo_index,
            plugin_registry,
        }),
    })
}

pub(super) async fn publish_repo_entity_search_plane(
    state: &GatewayState,
    project_root: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = project_root.join("wendao.toml");
    let repo_config = load_repo_intelligence_config(Some(config_path.as_path()), project_root)?;
    let repository = repo_config
        .repos
        .iter()
        .find(|repository| repository.id == repo_id)
        .ok_or_else(|| format!("missing repository `{repo_id}`"))?;
    let analysis = analyze_registered_repository_with_registry(
        repository,
        project_root,
        &state.studio.plugin_registry,
    )?;
    let repository_root = repository
        .path
        .as_ref()
        .ok_or_else(|| format!("repo `{repo_id}` missing path"))?;
    let mut relative_paths = Vec::new();
    if repository
        .plugins
        .iter()
        .any(|plugin| plugin.id() == "julia-code-parser")
    {
        relative_paths.extend(collect_relative_files_under(
            repository_root.as_path(),
            "src",
            "jl",
        )?);
        relative_paths.extend(collect_relative_files_under(
            repository_root.as_path(),
            "examples",
            "jl",
        )?);
    }
    if repository
        .plugins
        .iter()
        .any(|plugin| plugin.id() == "modelica")
    {
        relative_paths.extend(collect_relative_files_under(
            repository_root.as_path(),
            "",
            "mo",
        )?);
    }
    relative_paths.sort();
    relative_paths.dedup();
    let documents = repo_code_documents(repository_root.as_path(), relative_paths.as_slice())?;
    state
        .studio
        .search_plane
        .publish_repo_entities_with_revision(
            repo_id,
            &analysis,
            documents.as_slice(),
            Some("test-rev"),
        )
        .await?;
    Ok(())
}

pub(super) fn repo_code_documents(
    repo_root: &Path,
    relative_paths: &[String],
) -> Result<Vec<RepoCodeDocument>, Box<dyn std::error::Error>> {
    let mut documents = Vec::new();
    for relative_path in relative_paths {
        let absolute_path = repo_root.join(relative_path);
        if !absolute_path.exists() {
            continue;
        }
        let metadata = fs::metadata(&absolute_path)?;
        let modified_unix_ms = u64::try_from(
            metadata.modified()?.duration_since(UNIX_EPOCH)?.as_millis(),
        )
        .map_err(|error| {
            std::io::Error::other(format!(
                "repo document modified time overflow for {}: {error}",
                absolute_path.display()
            ))
        })?;
        documents.push(RepoCodeDocument {
            path: relative_path.clone(),
            language: repo_code_document_language(relative_path.as_str()).map(str::to_string),
            contents: Arc::<str>::from(fs::read_to_string(&absolute_path)?),
            size_bytes: metadata.len(),
            modified_unix_ms,
        });
    }
    Ok(documents)
}

pub(super) fn repo_code_document_language(relative_path: &str) -> Option<&'static str> {
    match Path::new(relative_path)
        .extension()
        .and_then(|value| value.to_str())
    {
        Some("jl") => Some("julia"),
        Some("mo") => Some("modelica"),
        _ => None,
    }
}

pub(super) fn gateway_state_for_project_with_options(
    project_root: &Path,
    start_repo_index: bool,
    prewarm_repo_analysis_cache: bool,
) -> Arc<GatewayState> {
    let config_root = project_root.to_path_buf();
    let ui_config =
        crate::studio::router::load_ui_config_from_wendao_toml(&config_root).unwrap_or_default();
    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap builtin plugin registry: {error}")),
    );
    let repo_index = Arc::new(RepoIndexCoordinator::new(
        project_root.to_path_buf(),
        Arc::clone(&plugin_registry),
        xiuxian_wendao::search::SearchPlaneService::new(project_root.to_path_buf()),
    ));
    if start_repo_index {
        repo_index.start();
    }
    let config_path = config_root.join("wendao.toml");
    if prewarm_repo_analysis_cache && config_path.exists() {
        let repo_config = load_repo_intelligence_config(Some(config_path.as_path()), &config_root)
            .unwrap_or_else(|error| {
                panic!("load repo intelligence config for gateway tests: {error}")
            });
        for repository in &repo_config.repos {
            analyze_registered_repository_with_registry(
                repository,
                config_root.as_path(),
                &plugin_registry,
            )
            .unwrap_or_else(|error| {
                panic!("prewarm repository analysis cache for gateway tests: {error}")
            });
        }
    }

    Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState {
            project_root: project_root.to_path_buf(),
            config_root,
            bootstrap_background_indexing: false,
            cold_start_process_started_at: crate::studio::symbol_index::timestamp_now(),
            cold_start_process_started_instant: std::time::Instant::now(),
            cold_start_telemetry: Arc::new(RwLock::new(
                crate::studio::router::StudioSearchColdStartTelemetryState::default(),
            )),
            bootstrap_background_indexing_deferred_activation: Arc::new(RwLock::new(None)),
            configured_owners: Arc::new(RwLock::new(
                StudioState::configured_owners_from_ui_config(ui_config),
            )),
            graph_index: Arc::new(RwLock::new(None)),
            symbol_index: Arc::new(RwLock::new(None)),
            symbol_index_coordinator: Arc::new(SymbolIndexCoordinator::new(
                project_root.to_path_buf(),
                project_root.to_path_buf(),
                SearchPlaneService::new(project_root.to_path_buf()),
            )),
            local_corpus_scan_coalescing: Arc::new(RwLock::new(
                crate::studio::router::LocalCorpusScanCoalescingState::default(),
            )),
            search_plane: SearchPlaneService::new(project_root.to_path_buf()),
            vfs_scan: Arc::new(RwLock::new(None)),
            repo_index,
            plugin_registry,
        }),
    })
}

pub(super) fn write_default_repo_config(
    base: &Path,
    repo_dir: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_default_repo_config_without_priming(base, repo_dir, repo_id)?;
    prime_local_julia_fixture_analysis_cache(base, repo_id)?;
    Ok(())
}

pub(super) fn write_default_repo_config_without_priming(
    base: &Path,
    repo_dir: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        base.join("wendao.toml"),
        format!(
            r#"[sources.projects.{repo_id}]
root = "{}"
plugins = ["julia-code-parser"]
"#,
            repo_dir.display()
        ),
    )?;
    Ok(())
}

pub(crate) fn prime_local_julia_fixture_analysis_cache(
    project_root: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = project_root.join("wendao.toml");
    let repo_config = load_repo_intelligence_config(Some(config_path.as_path()), project_root)?;
    let repository = repo_config
        .repos
        .iter()
        .find(|repository| repository.id == repo_id)
        .ok_or_else(|| IoError::other(format!("missing repository `{repo_id}`")))?;
    let repository_source =
        resolve_registered_repository_source(repository, project_root, SyncMode::Status)?;
    let checkout_metadata = discover_checkout_metadata(repository_source.checkout_root.as_path());
    let cache_key = build_repository_analysis_cache_key(
        repository,
        &repository_source,
        checkout_metadata.as_ref(),
    );
    let analysis = build_local_julia_fixture_analysis(
        repo_id,
        repository_source.checkout_root.as_path(),
        checkout_metadata
            .as_ref()
            .and_then(|metadata| metadata.revision.clone()),
    )?;
    store_cached_repository_analysis(cache_key, &analysis)?;
    Ok(())
}

pub(super) fn build_local_julia_fixture_analysis(
    repo_id: &str,
    repo_root: &Path,
    revision: Option<String>,
) -> Result<RepositoryAnalysisOutput, Box<dyn std::error::Error>> {
    let (project_name, version, uuid) = local_project_metadata(repo_root)?;
    let mut modules = Vec::new();
    let mut symbols = Vec::new();
    let mut docs = Vec::new();
    let mut relations = Vec::new();

    for relative_path in collect_relative_files_under(repo_root, "src", "jl")? {
        let contents = fs::read_to_string(repo_root.join(&relative_path))?;
        parse_local_julia_source(
            repo_id,
            &relative_path,
            &contents,
            &mut modules,
            &mut symbols,
            &mut docs,
            &mut relations,
        );
    }

    append_readme_doc_record(repo_id, repo_root, &modules, &mut docs, &mut relations);
    append_markdown_doc_records(
        repo_id,
        repo_root,
        &modules,
        &symbols,
        &mut docs,
        &mut relations,
    )?;
    let mut examples =
        collect_example_records(repo_id, repo_root, &modules, &symbols, &mut relations)?;
    sort_local_fixture_analysis_records(&mut modules, &mut symbols, &mut docs, &mut examples);
    Ok(RepositoryAnalysisOutput {
        repository: Some(build_local_fixture_repository_record(
            repo_id,
            repo_root,
            revision,
            project_name,
            version,
            uuid,
            &modules,
        )),
        modules,
        symbols,
        imports: Vec::new(),
        examples,
        docs,
        relations,
        diagnostics: Vec::new(),
    })
}

pub(super) fn local_project_metadata(
    repo_root: &Path,
) -> Result<LocalProjectMetadata, Box<dyn std::error::Error>> {
    let project_toml = repo_root.join("Project.toml");
    if !project_toml.exists() {
        return Ok((None, None, None));
    }
    let contents = fs::read_to_string(project_toml)?;
    let mut name = None;
    let mut version = None;
    let mut uuid = None;
    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(value) = toml_string_value(trimmed, "name") {
            name = Some(value);
        } else if let Some(value) = toml_string_value(trimmed, "version") {
            version = Some(value);
        } else if let Some(value) = toml_string_value(trimmed, "uuid") {
            uuid = Some(value);
        }
    }
    Ok((name, version, uuid))
}

pub(super) fn toml_string_value(line: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    line.strip_prefix(prefix.as_str())
        .and_then(|value| value.trim().strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .map(ToString::to_string)
}

pub(super) fn collect_relative_files_under(
    repo_root: &Path,
    relative_root: &str,
    extension: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let root = repo_root.join(relative_root);
    let mut relative_paths = Vec::new();
    if !root.exists() {
        return Ok(relative_paths);
    }
    collect_relative_files_recursive(repo_root, root.as_path(), extension, &mut relative_paths)?;
    relative_paths.sort();
    Ok(relative_paths)
}

pub(super) fn collect_relative_files_recursive(
    repo_root: &Path,
    current_dir: &Path,
    extension: &str,
    relative_paths: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_relative_files_recursive(repo_root, path.as_path(), extension, relative_paths)?;
            continue;
        }
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value == extension)
        {
            relative_paths.push(repo_relative_path(repo_root, path.as_path())?);
        }
    }
    Ok(())
}

pub(super) fn repo_relative_path(
    repo_root: &Path,
    path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(path
        .strip_prefix(repo_root)?
        .to_string_lossy()
        .replace('\\', "/"))
}

pub(super) fn parse_local_julia_source(
    repo_id: &str,
    relative_path: &str,
    contents: &str,
    modules: &mut Vec<ModuleRecord>,
    symbols: &mut Vec<SymbolRecord>,
    docs: &mut Vec<DocRecord>,
    relations: &mut Vec<RelationRecord>,
) {
    let Some(module_name) = contents.lines().find_map(parse_module_name) else {
        return;
    };
    let module_id = format!("repo:{repo_id}:module:{module_name}");
    modules.push(ModuleRecord {
        repo_id: repo_id.to_string().into(),
        module_id: module_id.clone().into(),
        qualified_name: module_name.clone(),
        path: relative_path.to_string().into(),
    });

    let mut pending_docstring = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if is_single_line_docstring(trimmed) {
            pending_docstring = true;
            continue;
        }
        if let Some((symbol_name, signature, kind)) = parse_source_symbol(trimmed) {
            let symbol_id = format!("repo:{repo_id}:symbol:{module_name}.{symbol_name}");
            symbols.push(SymbolRecord {
                repo_id: repo_id.to_string().into(),
                symbol_id: symbol_id.clone().into(),
                module_id: Some(module_id.clone().into()),
                name: symbol_name.clone(),
                qualified_name: format!("{module_name}.{symbol_name}"),
                kind,
                path: relative_path.to_string().into(),
                line_start: None,
                line_end: None,
                signature,
                audit_status: None,
                verification_state: Some("unknown".to_string().into()),
                attributes: BTreeMap::new(),
            });
            if pending_docstring {
                let doc_path = format!("{relative_path}#symbol-id:{symbol_id}");
                let doc_id = format!("repo:{repo_id}:doc:{doc_path}");
                docs.push(DocRecord {
                    repo_id: repo_id.to_string().into(),
                    doc_id: doc_id.clone().into(),
                    title: symbol_name.clone(),
                    path: doc_path.into(),
                    format: Some("julia_docstring".to_string()),
                    doc_target: None,
                });
                relations.push(RelationRecord {
                    repo_id: repo_id.to_string().into(),
                    source_id: doc_id,
                    target_id: symbol_id,
                    kind: RelationKind::Documents,
                });
            }
            pending_docstring = false;
            continue;
        }
        if pending_docstring && !trimmed.is_empty() && !trimmed.starts_with('#') {
            pending_docstring = false;
        }
    }
}

pub(super) fn parse_module_name(line: &str) -> Option<String> {
    let module_name = line.strip_prefix("module ")?;
    valid_identifier(module_name).then(|| module_name.to_string())
}

pub(super) fn parse_source_symbol(line: &str) -> Option<(String, Option<String>, RepoSymbolKind)> {
    if let Some(name) = line.strip_prefix("struct ") {
        let name = name.split_whitespace().next()?;
        return valid_identifier(name).then(|| {
            (
                name.to_string(),
                Some(format!("struct {name}")),
                RepoSymbolKind::Type,
            )
        });
    }
    let (head, _) = line.split_once('(')?;
    let name = head.trim();
    if !valid_identifier(name) {
        return None;
    }
    let (_, tail) = line.split_once(')')?;
    tail.trim_start().starts_with('=').then(|| {
        (
            name.to_string(),
            Some(line.to_string()),
            RepoSymbolKind::Function,
        )
    })
}

pub(super) fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '!')
}

pub(super) fn is_single_line_docstring(line: &str) -> bool {
    line.starts_with("\"\"\"") && line.ends_with("\"\"\"") && line.len() >= 6
}

pub(super) fn markdown_title(relative_path: &str, contents: &str) -> String {
    if relative_path == "README.md" {
        return "README.md".to_string();
    }
    contents
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix('#')
                .map(|rest| rest.trim_start_matches('#').trim())
                .filter(|title| !title.is_empty())
        })
        .map_or_else(|| example_title(relative_path), ToString::to_string)
}

pub(super) fn append_readme_doc_record(
    repo_id: &str,
    repo_root: &Path,
    modules: &[ModuleRecord],
    docs: &mut Vec<DocRecord>,
    relations: &mut Vec<RelationRecord>,
) {
    if !repo_root.join("README.md").exists() {
        return;
    }
    docs.push(DocRecord {
        repo_id: repo_id.to_string().into(),
        doc_id: format!("repo:{repo_id}:doc:README.md").into(),
        title: "README.md".to_string(),
        path: "README.md".to_string().into(),
        format: Some("md".to_string()),
        doc_target: None,
    });
    if let Some(module) = modules.first() {
        relations.push(RelationRecord {
            repo_id: repo_id.to_string().into(),
            source_id: format!("repo:{repo_id}:doc:README.md"),
            target_id: module.module_id.to_string(),
            kind: RelationKind::Documents,
        });
    }
}

pub(super) fn append_markdown_doc_records(
    repo_id: &str,
    repo_root: &Path,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
    docs: &mut Vec<DocRecord>,
    relations: &mut Vec<RelationRecord>,
) -> Result<(), Box<dyn std::error::Error>> {
    for relative_path in collect_relative_files_under(repo_root, "docs", "md")? {
        let contents = fs::read_to_string(repo_root.join(&relative_path))?;
        let title = markdown_title(&relative_path, &contents);
        let doc_id = format!("repo:{repo_id}:doc:{relative_path}");
        docs.push(DocRecord {
            repo_id: repo_id.to_string().into(),
            doc_id: doc_id.clone().into(),
            title: title.clone(),
            path: relative_path.clone().into(),
            format: Some("md".to_string()),
            doc_target: None,
        });
        if let Some(target_id) = matching_doc_target_id(&title, modules, symbols) {
            relations.push(RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: doc_id,
                target_id,
                kind: RelationKind::Documents,
            });
        }
    }
    Ok(())
}

pub(super) fn collect_example_records(
    repo_id: &str,
    repo_root: &Path,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
    relations: &mut Vec<RelationRecord>,
) -> Result<Vec<ExampleRecord>, Box<dyn std::error::Error>> {
    let mut examples = Vec::new();
    for relative_path in collect_relative_files_under(repo_root, "examples", "jl")? {
        let contents = fs::read_to_string(repo_root.join(&relative_path))?;
        let example_id = format!("repo:{repo_id}:example:{relative_path}");
        examples.push(ExampleRecord {
            repo_id: repo_id.to_string().into(),
            example_id: example_id.clone().into(),
            title: example_title(&relative_path),
            path: relative_path.clone().into(),
            summary: None,
        });
        for target_id in example_target_ids(&contents, modules, symbols) {
            relations.push(RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: example_id.clone(),
                target_id,
                kind: RelationKind::ExampleOf,
            });
        }
    }
    Ok(examples)
}

pub(super) fn sort_local_fixture_analysis_records(
    modules: &mut [ModuleRecord],
    symbols: &mut [SymbolRecord],
    docs: &mut [DocRecord],
    examples: &mut [ExampleRecord],
) {
    modules.sort_by(|left, right| {
        left.qualified_name
            .cmp(&right.qualified_name)
            .then_with(|| left.path.cmp(&right.path))
    });
    symbols.sort_by(|left, right| {
        left.qualified_name
            .cmp(&right.qualified_name)
            .then_with(|| left.path.cmp(&right.path))
    });
    docs.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.doc_id.cmp(&right.doc_id))
    });
    examples.sort_by(|left, right| left.path.cmp(&right.path));
}

pub(super) fn build_local_fixture_repository_record(
    repo_id: &str,
    repo_root: &Path,
    revision: Option<String>,
    project_name: Option<String>,
    version: Option<String>,
    uuid: Option<String>,
    modules: &[ModuleRecord],
) -> RepositoryRecord {
    RepositoryRecord {
        repo_id: repo_id.to_string().into(),
        name: project_name
            .or_else(|| modules.first().map(|module| module.qualified_name.clone()))
            .unwrap_or_else(|| repo_id.to_string()),
        path: repo_root.display().to_string().into(),
        url: None,
        revision,
        version,
        uuid,
        dependencies: Vec::new(),
    }
}

pub(super) fn example_title(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(relative_path)
        .to_string()
}

pub(super) fn matching_doc_target_id(
    title: &str,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
) -> Option<String> {
    symbols
        .iter()
        .find(|symbol| symbol.name == title || symbol.qualified_name == title)
        .map(|symbol| symbol.symbol_id.to_string())
        .or_else(|| {
            modules
                .iter()
                .find(|module| {
                    module.qualified_name == title
                        || module
                            .qualified_name
                            .rsplit('.')
                            .next()
                            .is_some_and(|name| name == title)
                })
                .map(|module| module.module_id.to_string())
        })
}

pub(super) fn example_target_ids(
    contents: &str,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
) -> Vec<String> {
    let mut target_ids = Vec::new();
    for symbol in symbols {
        if symbol_mentioned_in_example(contents, &symbol.name) {
            push_unique_string(&mut target_ids, symbol.symbol_id.to_string());
            if let Some(module_id) = symbol.module_id.as_ref() {
                push_unique_string(&mut target_ids, module_id.to_string());
            }
        }
    }
    if !target_ids.is_empty() {
        return target_ids;
    }
    for module in modules {
        if module_mentioned_in_example(contents, &module.qualified_name) {
            push_unique_string(&mut target_ids, module.module_id.to_string());
        }
    }
    target_ids
}

pub(super) fn symbol_mentioned_in_example(contents: &str, symbol_name: &str) -> bool {
    contents.contains(&format!("{symbol_name}(")) || contents.contains(&format!(".{symbol_name}("))
}

pub(super) fn module_mentioned_in_example(contents: &str, module_name: &str) -> bool {
    contents.contains(&format!("using {module_name}"))
        || contents.contains(&format!("import {module_name}"))
}

pub(super) fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

pub(super) fn create_local_git_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(repo_dir.join("README.md"), "# Gateway Repo\n")?;
    fs::write(
        repo_dir.join("Project.toml"),
        format!(
            r#"name = "{package_name}"
uuid = "12345678-1234-1234-1234-123456789abc"
version = "0.1.0"
"#
        ),
    )?;
    fs::write(
        repo_dir.join("src").join(format!("{package_name}.jl")),
        format!("module {package_name}\nend\n"),
    )?;

    init_git_repository(&repo_dir);
    add_git_remote(
        &repo_dir,
        "origin",
        &format!(
            "https://example.invalid/xiuxian-wendao/{}.git",
            package_name.to_ascii_lowercase()
        ),
    );
    commit_all(&repo_dir, "initial import");
    Ok(repo_dir)
}

pub(super) fn create_local_modelica_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("Controllers").join("Examples"))?;
    fs::create_dir_all(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("Tutorial"),
    )?;
    fs::write(repo_dir.join("README.md"), format!("# {package_name}\n"))?;
    fs::write(repo_dir.join("package.order"), "Controllers\n")?;
    fs::write(
        repo_dir.join("package.mo"),
        format!(
            "within;\npackage {package_name}\n  annotation(Documentation(info = \"<html>{package_name} package docs.</html>\"));\nend {package_name};\n",
        ),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("package.mo"),
        format!("within {package_name};\npackage Controllers\nend Controllers;\n"),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("PI.mo"),
        format!(
            "within {package_name}.Controllers;\nmodel PI\n  annotation(Documentation(info = \"<html>PI controller docs.</html>\"));\nend PI;\n",
        ),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("Examples")
            .join("package.order"),
        "Step\n",
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("Examples")
            .join("Step.mo"),
        format!("within {package_name}.Controllers.Examples;\nmodel Step\nend Step;\n"),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("package.order"),
        "Tutorial\n",
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("package.mo"),
        format!("within {package_name}.Controllers;\npackage UsersGuide\nend UsersGuide;\n"),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("Tutorial")
            .join("FirstSteps.mo"),
        format!(
            "within {package_name}.Controllers.UsersGuide.Tutorial;\nmodel FirstSteps\n  annotation(Documentation(info = \"<html>First steps guide.</html>\"));\nend FirstSteps;\n",
        ),
    )?;

    init_git_repository(&repo_dir);
    add_git_remote(
        &repo_dir,
        "origin",
        &format!(
            "https://example.invalid/xiuxian-wendao/{}.git",
            package_name.to_ascii_lowercase()
        ),
    );
    commit_all(&repo_dir, "initial import");
    Ok(repo_dir)
}

pub(super) fn write_modelica_repo_config(
    base: &Path,
    repo_dir: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        base.join("wendao.toml"),
        format!(
            r#"[sources.projects.{repo_id}]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    Ok(())
}

pub(super) fn projected_page_id_for_title(
    base: &Path,
    repo_id: &str,
    kind: ProjectionPageKind,
    title: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: repo_id.to_string(),
        },
        None,
        base,
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| page.kind == kind && page.title == title)
        .unwrap_or_else(|| panic!("expected a projected `{title}` page in repo `{repo_id}`"));
    Ok(page.page_id.clone())
}

pub(super) fn projected_page_and_node_id_for_title(
    base: &Path,
    repo_id: &str,
    title: &str,
    node_title: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let page_id = projected_page_id_for_title(base, repo_id, ProjectionPageKind::Reference, title)?;
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: repo_id.to_string(),
        },
        None,
        base,
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for `{title}`"));
    let node_id = find_node_id(tree.roots.as_slice(), node_title)
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `{node_title}`"));
    Ok((page_id, node_id))
}

pub(super) fn redact_repo_sync_payload(value: &mut Value) {
    if let Some(path) = value.pointer_mut("/checkout_path") {
        *path = Value::String("[checkout-path]".to_string());
    }
    if let Some(path) = value.pointer_mut("/mirror_path") {
        *path = Value::String("[mirror-path]".to_string());
    }
    if let Some(url) = value.pointer_mut("/upstream_url") {
        *url = Value::String("[upstream-url]".to_string());
    }
    if let Some(path) = value.pointer_mut("/checked_at") {
        *path = Value::String("[checked-at]".to_string());
    }
    if let Some(path) = value.pointer_mut("/last_fetched_at") {
        *path = match path {
            Value::Null => Value::Null,
            _ => Value::String("[last-fetched-at]".to_string()),
        };
    }
    if let Some(path) = value.pointer_mut("/status_summary/freshness/checked_at") {
        *path = Value::String("[checked-at]".to_string());
    }
    if let Some(path) = value.pointer_mut("/status_summary/freshness/last_fetched_at") {
        *path = match path {
            Value::Null => Value::Null,
            _ => Value::String("[last-fetched-at]".to_string()),
        };
    }
    redact_revision_pointers(
        value,
        &[
            "/revision",
            "/mirror_revision",
            "/tracking_revision",
            "/status_summary/revisions/checkout_revision",
            "/status_summary/revisions/mirror_revision",
            "/status_summary/revisions/tracking_revision",
        ],
    );
}

pub(super) fn redact_repo_overview_payload(value: &mut Value) {
    redact_revision_pointers(value, &["/revision"]);
}

pub(super) fn redact_revision_pointers(value: &mut Value, pointers: &[&str]) {
    for pointer in pointers {
        if let Some(revision) = value.pointer_mut(pointer)
            && !revision.is_null()
        {
            *revision = Value::String("[revision]".to_string());
        }
    }
}

pub(super) fn redact_repo_index_payload(value: &mut Value) {
    if let Some(max_concurrency) = value.get_mut("maxConcurrency") {
        *max_concurrency = Value::Number(1.into());
    }
    if let Some(sync_concurrency_limit) = value.get_mut("syncConcurrencyLimit") {
        *sync_concurrency_limit = Value::Number(1.into());
    }
    if let Some(repos) = value.get_mut("repos").and_then(Value::as_array_mut) {
        for repo in repos {
            if let Some(updated_at) = repo.get_mut("updatedAt") {
                *updated_at = Value::String("[updated-at]".to_string());
            }
        }
    }
    if let Some(activation_at) =
        value.get_mut("studioBootstrapBackgroundIndexingDeferredActivationAt")
    {
        *activation_at = match activation_at {
            Value::Null => Value::Null,
            _ => Value::String("[activation-at]".to_string()),
        };
    }
}

pub(super) fn find_node_id(nodes: &[ProjectedPageIndexNode], title: &str) -> Option<String> {
    for node in nodes {
        if node.title == title {
            return Some(node.node_id.clone());
        }
        if let Some(node_id) = find_node_id(node.children.as_slice(), title) {
            return Some(node_id);
        }
    }
    None
}
