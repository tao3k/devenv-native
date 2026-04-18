use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, DocRecord, DocTargetRecord, ModuleRecord, PluginAnalysisOutput,
    PluginLinkContext, RegisteredRepository, RelationKind, RelationRecord, RepoIntelligenceError,
    RepoIntelligencePlugin, RepoSourceFile, RepoSymbolKind, RepositoryAnalysisOutput,
    RepositoryRecord, SymbolRecord,
};

use super::capability_manifest::validate_julia_capability_manifest_preflight_for_repository;
use super::discovery::{discover_docs, discover_examples, relative_path_string};
use super::graph_structural::GraphStructuralRouteKind;
use super::graph_structural_transport::build_graph_structural_flight_transport_client;
use super::linking::{build_doc_relations, build_example_relations};
use super::parser_summary::{
    JuliaParserDocAttachment, JuliaParserDocTargetKind, JuliaParserFileSummary, JuliaParserImport,
    JuliaParserSymbol, JuliaParserSymbolKind,
    fetch_julia_parser_file_summary_blocking_for_repository,
    validate_julia_parser_summary_preflight_for_repository,
};
use super::project::{load_project_metadata, locate_root_module_file};
use super::sources::{JuliaAnalyzedFile, collect_julia_sources};
use super::transport::build_julia_flight_transport_client;

const JULIA_PLUGIN_ID: &str = "julia";

/// External Julia analyzer for Repo Intelligence.
#[derive(Debug, Default, Clone, Copy)]
pub struct JuliaRepoIntelligencePlugin;

/// Register the Julia plugin into an existing Repo Intelligence registry.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the registry already contains a
/// plugin with the `julia` identifier.
pub fn register_into(
    registry: &mut xiuxian_wendao_core::repo_intelligence::PluginRegistry,
) -> Result<(), RepoIntelligenceError> {
    registry.register(JuliaRepoIntelligencePlugin)
}

inventory::submit! {
    xiuxian_wendao_core::repo_intelligence::BuiltinPluginRegistrar::new(
        JULIA_PLUGIN_ID,
        register_into,
    )
}

impl RepoIntelligencePlugin for JuliaRepoIntelligencePlugin {
    fn id(&self) -> &'static str {
        JULIA_PLUGIN_ID
    }

    fn supports_repository(&self, repository: &RegisteredRepository) -> bool {
        repository
            .plugins
            .iter()
            .any(|plugin| plugin.id() == JULIA_PLUGIN_ID)
    }

    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        let summary = parse_julia_file_summary(context, file)?;
        let module_name = summary.module_name.clone().unwrap_or_else(|| {
            load_project_metadata(&context.repository.id, context.repository_root.as_path())
                .map_or_else(|_| context.repository.id.clone(), |metadata| metadata.name)
        });
        let module_id = format!("repo:{}:module:{}", context.repository.id, module_name);
        let symbols = build_symbol_records(
            &context.repository.id,
            &file.path,
            &module_name,
            &summary.exports,
            &summary.symbols,
        );
        Ok(PluginAnalysisOutput {
            modules: summary
                .module_name
                .as_ref()
                .map(|qualified_name| ModuleRecord {
                    repo_id: context.repository.id.clone(),
                    module_id,
                    qualified_name: qualified_name.clone(),
                    path: file.path.clone(),
                })
                .into_iter()
                .collect(),
            symbols: symbols.clone(),
            imports: Vec::new(),
            docs: build_docstring_records(
                &context.repository.id,
                &file.path,
                &module_name,
                &symbols,
                &summary.docstrings,
            ),
            examples: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn preflight_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<(), RepoIntelligenceError> {
        validate_julia_parser_summary_preflight_for_repository(&context.repository)?;
        let _maybe_transport = build_julia_flight_transport_client(&context.repository)?;
        let _maybe_manifest_rows =
            validate_julia_capability_manifest_preflight_for_repository(&context.repository)?;
        let _maybe_graph_structural_rerank_transport =
            build_graph_structural_flight_transport_client(
                &context.repository,
                GraphStructuralRouteKind::StructuralRerank,
            )?;
        let _maybe_graph_structural_filter_transport =
            build_graph_structural_flight_transport_client(
                &context.repository,
                GraphStructuralRouteKind::ConstraintFilter,
            )?;
        let metadata = load_project_metadata(&context.repository.id, repository_root)?;
        let mut diagnostics = Vec::new();
        let _ = locate_root_module_file(
            &context.repository.id,
            repository_root,
            &metadata.name,
            &mut diagnostics,
        )?;
        Ok(())
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        let metadata = load_project_metadata(&context.repository.id, repository_root)?;
        let mut diagnostics = Vec::new();
        let root_file = locate_root_module_file(
            &context.repository.id,
            repository_root,
            &metadata.name,
            &mut diagnostics,
        )?;
        let root_path = relative_path_string(repository_root, &root_file)?;
        let collected = collect_julia_sources(
            &context.repository,
            repository_root,
            &root_file,
            &mut diagnostics,
        )?;
        let module_id = format!(
            "repo:{}:module:{}",
            context.repository.id, collected.root_summary.module_name
        );
        let examples = discover_examples(&context.repository.id, repository_root)?;
        let modules = vec![ModuleRecord {
            repo_id: context.repository.id.clone(),
            module_id: module_id.clone(),
            qualified_name: collected.root_summary.module_name.clone(),
            path: root_path.clone(),
        }];
        let symbols = collect_symbol_records(
            &context.repository.id,
            &collected.root_summary.module_name,
            &collected.files,
        );
        let mut docs = discover_docs(&context.repository.id, repository_root)?;
        for file in &collected.files {
            docs.extend(build_docstring_records(
                &context.repository.id,
                &file.path,
                &collected.root_summary.module_name,
                &symbols,
                &file.summary.docstrings,
            ));
        }
        let mut relations = Vec::new();
        for file in &collected.files {
            relations.extend(build_import_relations(
                &context.repository.id,
                &module_id,
                &file.summary.imports,
            ));
            relations.extend(build_docstring_relations(
                &context.repository.id,
                &module_id,
                &collected.root_summary.module_name,
                &symbols,
                &file.summary.docstrings,
                &file.path,
            ));
        }
        relations.extend(build_structural_relations(
            &context.repository.id,
            &modules,
            &symbols,
            &examples,
            &docs,
        ));

        Ok(RepositoryAnalysisOutput {
            repository: Some(RepositoryRecord {
                repo_id: context.repository.id.clone(),
                name: metadata.name,
                path: repository_root.display().to_string(),
                url: context.repository.url.clone(),
                revision: None,
                version: metadata.version,
                uuid: metadata.uuid,
                dependencies: metadata.dependencies,
            }),
            modules,
            symbols,
            imports: Vec::new(),
            examples,
            docs,
            relations,
            diagnostics,
        })
    }

    fn enrich_relations(
        &self,
        context: &PluginLinkContext,
    ) -> Result<Vec<RelationRecord>, RepoIntelligenceError> {
        let mut relations = build_doc_relations(context)?;
        relations.extend(build_example_relations(context)?);
        Ok(relations)
    }
}

fn parse_julia_file_summary(
    context: &AnalysisContext,
    file: &RepoSourceFile,
) -> Result<JuliaParserFileSummary, RepoIntelligenceError> {
    fetch_julia_parser_file_summary_blocking_for_repository(
        &context.repository,
        &file.path,
        &file.contents,
    )
}

fn collect_symbol_records(
    repo_id: &str,
    module_name: &str,
    files: &[JuliaAnalyzedFile],
) -> Vec<SymbolRecord> {
    let mut export_bindings = Vec::new();
    let mut actual_symbols = Vec::new();

    for file in files {
        export_bindings.extend(
            file.summary
                .exports
                .iter()
                .cloned()
                .map(|export_name| (file.path.clone(), export_name)),
        );
        actual_symbols.extend(collect_pending_symbols(&file.path, &file.summary.symbols));
    }

    materialize_symbol_records(repo_id, module_name, &export_bindings, actual_symbols)
}

fn build_symbol_records(
    repo_id: &str,
    path: &str,
    module_name: &str,
    exports: &[String],
    symbols: &[JuliaParserSymbol],
) -> Vec<SymbolRecord> {
    let export_bindings = exports
        .iter()
        .cloned()
        .map(|export_name| (path.to_string(), export_name))
        .collect::<Vec<_>>();
    let actual_symbols = collect_pending_symbols(path, symbols);
    materialize_symbol_records(repo_id, module_name, &export_bindings, actual_symbols)
}

#[derive(Debug, Clone)]
struct PendingSymbolRecord {
    path: String,
    name: String,
    kind: RepoSymbolKind,
    signature: Option<String>,
    line_start: Option<usize>,
    line_end: Option<usize>,
    attributes: BTreeMap<String, String>,
}

fn collect_pending_symbols(path: &str, symbols: &[JuliaParserSymbol]) -> Vec<PendingSymbolRecord> {
    symbols
        .iter()
        .map(|symbol| PendingSymbolRecord {
            path: path.to_string(),
            name: symbol.name.clone(),
            kind: match symbol.kind {
                JuliaParserSymbolKind::Function => RepoSymbolKind::Function,
                JuliaParserSymbolKind::Type => RepoSymbolKind::Type,
                JuliaParserSymbolKind::Constant => RepoSymbolKind::Constant,
                JuliaParserSymbolKind::Other => RepoSymbolKind::Other,
            },
            signature: symbol.signature.clone(),
            line_start: symbol.line_start,
            line_end: symbol.line_end,
            attributes: symbol.attributes.clone(),
        })
        .collect()
}

fn materialize_symbol_records(
    repo_id: &str,
    module_name: &str,
    export_bindings: &[(String, String)],
    actual_symbols: Vec<PendingSymbolRecord>,
) -> Vec<SymbolRecord> {
    let mut symbol_map = BTreeMap::new();
    let mut actual_counts = BTreeMap::new();
    let actual_names = actual_symbols
        .iter()
        .map(|symbol| symbol.name.clone())
        .collect::<BTreeSet<_>>();

    for symbol in &actual_symbols {
        *actual_counts
            .entry(base_symbol_id(repo_id, module_name, &symbol.name))
            .or_insert(0usize) += 1;
    }

    let mut duplicate_ordinals = BTreeMap::new();
    for symbol in actual_symbols {
        let base_id = base_symbol_id(repo_id, module_name, &symbol.name);
        let duplicate_count = actual_counts.get(&base_id).copied().unwrap_or(0);
        let duplicate_ordinal = if duplicate_count > 1 {
            let ordinal = duplicate_ordinals.entry(base_id.clone()).or_insert(0usize);
            *ordinal += 1;
            Some(*ordinal)
        } else {
            None
        };
        let symbol_id =
            build_materialized_symbol_id(repo_id, module_name, &symbol, duplicate_ordinal);
        let record = build_symbol_record(repo_id, module_name, symbol, symbol_id);
        upsert_symbol(&mut symbol_map, record);
    }

    for (path, export_name) in export_bindings {
        if actual_names.contains(export_name) {
            continue;
        }
        let symbol = build_symbol_record(
            repo_id,
            module_name,
            PendingSymbolRecord {
                path: path.clone(),
                name: export_name.clone(),
                kind: RepoSymbolKind::ModuleExport,
                signature: None,
                line_start: None,
                line_end: None,
                attributes: BTreeMap::new(),
            },
            base_symbol_id(repo_id, module_name, export_name),
        );
        symbol_map.entry(symbol.symbol_id.clone()).or_insert(symbol);
    }

    symbol_map.into_values().collect()
}

fn build_import_relations(
    repo_id: &str,
    module_id: &str,
    imports: &[JuliaParserImport],
) -> Vec<RelationRecord> {
    imports
        .iter()
        .map(|import| RelationRecord {
            repo_id: repo_id.to_string(),
            source_id: module_id.to_string(),
            target_id: format!("external:{}", import.module),
            kind: RelationKind::Uses,
        })
        .collect()
}

fn build_structural_relations(
    repo_id: &str,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
    examples: &[xiuxian_wendao_core::repo_intelligence::ExampleRecord],
    docs: &[DocRecord],
) -> Vec<RelationRecord> {
    let repository_node_id = format!("repo:{repo_id}");
    let mut relations = Vec::new();

    relations.extend(modules.iter().map(|module| RelationRecord {
        repo_id: repo_id.to_string(),
        source_id: repository_node_id.clone(),
        target_id: module.module_id.clone(),
        kind: RelationKind::Contains,
    }));
    relations.extend(symbols.iter().filter_map(|symbol| {
        symbol.module_id.as_ref().map(|module_id| RelationRecord {
            repo_id: repo_id.to_string(),
            source_id: module_id.clone(),
            target_id: symbol.symbol_id.clone(),
            kind: RelationKind::Declares,
        })
    }));
    relations.extend(examples.iter().map(|example| RelationRecord {
        repo_id: repo_id.to_string(),
        source_id: repository_node_id.clone(),
        target_id: example.example_id.clone(),
        kind: RelationKind::Contains,
    }));
    relations.extend(docs.iter().map(|doc| RelationRecord {
        repo_id: repo_id.to_string(),
        source_id: repository_node_id.clone(),
        target_id: doc.doc_id.clone(),
        kind: RelationKind::Contains,
    }));

    relations
}

fn build_docstring_records(
    repo_id: &str,
    path: &str,
    module_name: &str,
    symbols: &[SymbolRecord],
    docstrings: &[JuliaParserDocAttachment],
) -> Vec<DocRecord> {
    docstrings
        .iter()
        .filter_map(|docstring| {
            let anchor = match resolve_docstring_target(module_name, symbols, docstring)? {
                ResolvedDocstringTarget::Module => format!("module:{}", docstring.target_name),
                ResolvedDocstringTarget::Symbol(symbol) => {
                    docstring_symbol_anchor(docstring, symbol)
                }
            };
            Some(DocRecord {
                repo_id: repo_id.to_string(),
                doc_id: format!("repo:{repo_id}:doc:{path}#{anchor}"),
                title: docstring.target_name.clone(),
                path: format!("{path}#{anchor}"),
                format: Some("julia_docstring".to_string()),
                doc_target: Some(build_doc_target_record(docstring)),
            })
        })
        .collect()
}

fn build_docstring_relations(
    repo_id: &str,
    module_id: &str,
    module_name: &str,
    symbols: &[SymbolRecord],
    docstrings: &[JuliaParserDocAttachment],
    path: &str,
) -> Vec<RelationRecord> {
    docstrings
        .iter()
        .filter_map(|docstring| {
            let (anchor, target_id) =
                match resolve_docstring_target(module_name, symbols, docstring)? {
                    ResolvedDocstringTarget::Module => (
                        format!("module:{}", docstring.target_name),
                        module_id.to_string(),
                    ),
                    ResolvedDocstringTarget::Symbol(symbol) => (
                        docstring_symbol_anchor(docstring, symbol),
                        symbol.symbol_id.clone(),
                    ),
                };
            Some(RelationRecord {
                repo_id: repo_id.to_string(),
                source_id: format!("repo:{repo_id}:doc:{path}#{anchor}"),
                target_id,
                kind: RelationKind::Documents,
            })
        })
        .collect()
}

enum ResolvedDocstringTarget<'a> {
    Module,
    Symbol(&'a SymbolRecord),
}

fn resolve_docstring_target<'a>(
    module_name_or_id: &str,
    symbols: &'a [SymbolRecord],
    docstring: &JuliaParserDocAttachment,
) -> Option<ResolvedDocstringTarget<'a>> {
    match docstring.target_kind {
        JuliaParserDocTargetKind::Module if docstring.target_name == module_name_or_id => {
            Some(ResolvedDocstringTarget::Module)
        }
        JuliaParserDocTargetKind::Module => None,
        JuliaParserDocTargetKind::Symbol => {
            resolve_docstring_symbol(symbols, docstring).map(ResolvedDocstringTarget::Symbol)
        }
    }
}

fn resolve_docstring_symbol<'a>(
    symbols: &'a [SymbolRecord],
    docstring: &JuliaParserDocAttachment,
) -> Option<&'a SymbolRecord> {
    let mut candidates = symbols
        .iter()
        .filter(|symbol| symbol.name == docstring.target_name)
        .collect::<Vec<_>>();

    if let Some(target_path) = docstring.target_path.as_ref() {
        let matched = candidates
            .iter()
            .copied()
            .filter(|symbol| symbol_matches_doc_target_path(symbol, target_path))
            .collect::<Vec<_>>();
        if !matched.is_empty() {
            candidates = matched;
        }
    }

    if let Some(target_line_start) = docstring.target_line_start {
        let matched = candidates
            .iter()
            .copied()
            .filter(|symbol| symbol.line_start == Some(target_line_start))
            .collect::<Vec<_>>();
        if !matched.is_empty() {
            candidates = matched;
        }
    }

    if let Some(target_line_end) = docstring.target_line_end {
        let matched = candidates
            .iter()
            .copied()
            .filter(|symbol| {
                symbol.line_end.unwrap_or(symbol.line_start.unwrap_or(0)) == target_line_end
            })
            .collect::<Vec<_>>();
        if !matched.is_empty() {
            candidates = matched;
        }
    }

    candidates
        .into_iter()
        .min_by(|left, right| left.symbol_id.cmp(&right.symbol_id))
}

fn symbol_matches_doc_target_path(symbol: &SymbolRecord, target_path: &str) -> bool {
    if symbol.qualified_name == target_path {
        return true;
    }
    if let Some(owner_path) = symbol.attributes.get("owner_path")
        && format!("{owner_path}.{}", symbol.name) == target_path
    {
        return true;
    }
    if let Some(module_path) = symbol.attributes.get("module_path")
        && format!("{module_path}.{}", symbol.name) == target_path
    {
        return true;
    }
    false
}

fn docstring_symbol_anchor(docstring: &JuliaParserDocAttachment, symbol: &SymbolRecord) -> String {
    if docstring.target_line_start.is_some()
        || docstring.target_line_end.is_some()
        || docstring.target_path.is_some()
        || symbol.symbol_id.contains('@')
    {
        return format!("symbol-id:{}", symbol.symbol_id);
    }
    format!("symbol:{}", docstring.target_name)
}

fn build_doc_target_record(docstring: &JuliaParserDocAttachment) -> DocTargetRecord {
    DocTargetRecord {
        kind: doc_target_kind_label(docstring.target_kind).to_string(),
        name: docstring.target_name.clone(),
        path: docstring.target_path.clone(),
        line_start: docstring.target_line_start,
        line_end: docstring.target_line_end,
    }
}

fn doc_target_kind_label(target_kind: JuliaParserDocTargetKind) -> &'static str {
    match target_kind {
        JuliaParserDocTargetKind::Module => "module",
        JuliaParserDocTargetKind::Symbol => "symbol",
    }
}

fn build_symbol_record(
    repo_id: &str,
    module_name: &str,
    symbol: PendingSymbolRecord,
    symbol_id: String,
) -> SymbolRecord {
    let qualified_name = qualified_symbol_name(module_name, &symbol.name);
    SymbolRecord {
        repo_id: repo_id.to_string(),
        symbol_id,
        module_id: Some(format!("repo:{repo_id}:module:{module_name}")),
        name: symbol.name,
        qualified_name,
        kind: symbol.kind,
        path: symbol.path,
        line_start: symbol.line_start,
        line_end: symbol.line_end,
        signature: symbol.signature,
        audit_status: Some("unreviewed".to_string()),
        verification_state: None,
        attributes: symbol.attributes,
    }
}

fn build_materialized_symbol_id(
    repo_id: &str,
    module_name: &str,
    symbol: &PendingSymbolRecord,
    duplicate_ordinal: Option<usize>,
) -> String {
    let base_id = base_symbol_id(repo_id, module_name, &symbol.name);
    let Some(ordinal) = duplicate_ordinal else {
        return base_id;
    };

    let mut segments = vec![normalize_symbol_id_segment(&symbol.path)];
    if let Some(start) = symbol.line_start {
        let end = symbol.line_end.unwrap_or(start);
        segments.push(format!("l{start}-{end}"));
    }
    if let Some(signature) = &symbol.signature {
        segments.push(format!("sig_{}", normalize_symbol_id_segment(signature)));
    }
    segments.push(format!("dup{ordinal}"));
    format!("{base_id}@{}", segments.join("__"))
}

fn base_symbol_id(repo_id: &str, module_name: &str, symbol_name: &str) -> String {
    format!(
        "repo:{repo_id}:symbol:{}",
        qualified_symbol_name(module_name, symbol_name)
    )
}

fn qualified_symbol_name(module_name: &str, symbol_name: &str) -> String {
    format!("{module_name}.{symbol_name}")
}

fn normalize_symbol_id_segment(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' => ch,
            _ => '_',
        })
        .collect()
}

fn upsert_symbol(symbols: &mut BTreeMap<String, SymbolRecord>, symbol: SymbolRecord) {
    match symbols.get_mut(&symbol.symbol_id) {
        Some(existing) if existing.kind == RepoSymbolKind::ModuleExport => {
            *existing = symbol;
        }
        Some(_) => {}
        None => {
            symbols.insert(symbol.symbol_id.clone(), symbol);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/entry.rs"]
mod tests;
