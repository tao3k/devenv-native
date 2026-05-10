use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::analyzers::resolve_registered_repository_source;
use crate::link_graph::LinkGraphIndex;
use crate::link_graph::LinkGraphSearchOptions;
use crate::search::real_repo_precision::catalog::default_real_repo_precision_catalog;
use crate::search::real_repo_precision::evaluate::evaluate_gold_query_paths_with_timing;
use crate::search::real_repo_precision::receipt::{build_run_receipt, write_run_receipt};
use crate::search::real_repo_precision::scenario_matrix::evaluate_knowledge_scenario_matrix;
use crate::search::real_repo_precision::semantic_gate::{
    attach_markdown_knowledge_semantic_query_evidence, evaluate_markdown_knowledge_semantic_gate,
};
use crate::search::real_repo_precision::types::{
    RealRepoGoldQuery, RealRepoGoldQueryKind, RealRepoPrecisionCatalogEntry,
    RealRepoPrecisionCorpusPathPrefixReceipt, RealRepoPrecisionLinkGraphCorpusReceipt,
    RealRepoPrecisionRepositoryReceipt, RealRepoPrecisionRunOptions, RealRepoPrecisionRunStatus,
};
use crate::search::repo_search::{
    RepoAstAnalysisIndex, build_repo_ast_analysis_index_from_checkout,
};

pub(crate) fn run_real_repo_precision_harness() -> Result<RealRepoPrecisionRunStatus, String> {
    let options = RealRepoPrecisionRunOptions::from_env();
    run_real_repo_precision_harness_with_options(&options, default_real_repo_precision_catalog())
}

pub(crate) fn run_real_repo_precision_harness_with_options(
    options: &RealRepoPrecisionRunOptions,
    catalog: Vec<RealRepoPrecisionCatalogEntry>,
) -> Result<RealRepoPrecisionRunStatus, String> {
    run_real_repo_precision_catalog(options, catalog)
}

fn run_real_repo_precision_catalog(
    options: &RealRepoPrecisionRunOptions,
    catalog: Vec<RealRepoPrecisionCatalogEntry>,
) -> Result<RealRepoPrecisionRunStatus, String> {
    if !options.enabled {
        return Ok(RealRepoPrecisionRunStatus::Skipped {
            reason: "set RUN_WENDAO_REAL_REPO_SEARCH_PRECISION_TEST=1 to run".to_string(),
        });
    }

    let mut repository_receipts = Vec::with_capacity(catalog.len());
    for mut entry in catalog {
        if let Some(kind) = options.query_kind_filter {
            entry.gold_queries.retain(|query| query.kind == kind);
            if kind != RealRepoGoldQueryKind::LinkGraph {
                entry.knowledge_scenarios.clear();
            }
        }
        repository_receipts.push(run_repository(&entry, options)?);
    }

    let receipt = build_run_receipt(
        options.sync_mode,
        options.query_kind_filter,
        repository_receipts,
    );
    write_run_receipt(&receipt, options.receipt_path.as_path())?;
    Ok(RealRepoPrecisionRunStatus::Completed(receipt))
}

fn run_repository(
    entry: &RealRepoPrecisionCatalogEntry,
    options: &RealRepoPrecisionRunOptions,
) -> Result<RealRepoPrecisionRepositoryReceipt, String> {
    let repository_started_at = Instant::now();
    let repo_id = entry.repository.id.clone();
    let materialization = materialize_repository(entry, options, &repo_id, repository_started_at);
    let Some(materialized) = materialization.materialized else {
        return Ok(materialization.receipt);
    };
    let materialize_ms = materialization.materialize_ms;
    let checkout_root = materialized.checkout_root;

    if !checkout_root.is_dir() {
        return Ok(missing_checkout_receipt(
            &repo_id,
            &checkout_root,
            format!("{:?}", materialized.checkout_state),
            materialize_ms,
            repository_started_at,
        ));
    }

    let link_graph = build_link_graph_index(entry, options, &checkout_root, &repo_id)?;
    let repo_ast_language_filters = merged_repo_ast_language_filters(entry.gold_queries.as_slice());
    let repo_ast =
        build_repo_ast_index(entry, &checkout_root, repo_ast_language_filters.as_slice());
    let mut markdown_knowledge_semantic_gate = evaluate_markdown_knowledge_semantic_gate(
        &checkout_root.join("semantic"),
        &entry.gold_queries,
    )?
    .map(|evaluation| evaluation.receipt);
    let query_result =
        run_repository_queries(entry, link_graph.index.as_ref(), repo_ast.index.as_ref())?;

    if let Some(gate) = markdown_knowledge_semantic_gate.as_mut() {
        attach_markdown_knowledge_semantic_query_evidence(gate, &query_result.receipts);
    }
    let knowledge_scenarios = evaluate_knowledge_scenario_matrix(
        entry.knowledge_scenarios.as_slice(),
        &query_result.receipts,
        markdown_knowledge_semantic_gate.as_ref(),
    );

    Ok(RealRepoPrecisionRepositoryReceipt {
        repo_id,
        checkout_root: checkout_root.display().to_string(),
        lifecycle: format!("{:?}", materialized.checkout_state),
        indexed: true,
        materialize_ms,
        link_graph_index_ms: link_graph.index_ms,
        link_graph_cache_backend: link_graph.cache_backend,
        link_graph_cache_status: link_graph.cache_status,
        link_graph_cache_miss_reason: link_graph.cache_miss_reason,
        link_graph_corpus: link_graph.corpus,
        markdown_knowledge_semantic_gate,
        knowledge_scenarios,
        repo_ast_index_ms: repo_ast.index_ms,
        repo_ast_index_file_count: repo_ast.file_count,
        repo_ast_index_symbol_count: repo_ast.symbol_count,
        query_wall_ms: query_result.wall_ms,
        query_sum_ms: query_result.sum_ms,
        total_ms: elapsed_ms(repository_started_at.elapsed()),
        skip_reason: None,
        query_receipts: query_result.receipts,
    })
}

struct RepositoryMaterialization {
    materialized: Option<xiuxian_git_repo::MaterializedRepo>,
    materialize_ms: u128,
    receipt: RealRepoPrecisionRepositoryReceipt,
}

fn materialize_repository(
    entry: &RealRepoPrecisionCatalogEntry,
    options: &RealRepoPrecisionRunOptions,
    repo_id: &str,
    repository_started_at: Instant,
) -> RepositoryMaterialization {
    let materialize_started_at = Instant::now();
    match resolve_registered_repository_source(
        &entry.repository,
        options.project_root.as_path(),
        options.sync_mode.as_git_sync_mode(),
    ) {
        Ok(materialized) => RepositoryMaterialization {
            materialized: Some(materialized),
            materialize_ms: elapsed_ms(materialize_started_at.elapsed()),
            receipt: empty_repository_receipt(
                repo_id,
                String::new(),
                String::new(),
                0,
                None,
                repository_started_at,
            ),
        },
        Err(error) => {
            let materialize_ms = elapsed_ms(materialize_started_at.elapsed());
            RepositoryMaterialization {
                materialized: None,
                materialize_ms,
                receipt: empty_repository_receipt(
                    repo_id,
                    String::new(),
                    "MaterializationFailed".to_string(),
                    materialize_ms,
                    Some(format!("materialization failed: {error}")),
                    repository_started_at,
                ),
            }
        }
    }
}

fn missing_checkout_receipt(
    repo_id: &str,
    checkout_root: &Path,
    lifecycle: String,
    materialize_ms: u128,
    repository_started_at: Instant,
) -> RealRepoPrecisionRepositoryReceipt {
    empty_repository_receipt(
        repo_id,
        checkout_root.display().to_string(),
        lifecycle,
        materialize_ms,
        Some(format!(
            "checkout is missing; rerun with {}=ensure or refresh",
            crate::search::real_repo_precision::types::SYNC_MODE_ENV
        )),
        repository_started_at,
    )
}

fn empty_repository_receipt(
    repo_id: &str,
    checkout_root: String,
    lifecycle: String,
    materialize_ms: u128,
    skip_reason: Option<String>,
    repository_started_at: Instant,
) -> RealRepoPrecisionRepositoryReceipt {
    RealRepoPrecisionRepositoryReceipt {
        repo_id: repo_id.to_string(),
        checkout_root,
        lifecycle,
        indexed: false,
        materialize_ms,
        link_graph_index_ms: None,
        link_graph_cache_backend: None,
        link_graph_cache_status: None,
        link_graph_cache_miss_reason: None,
        link_graph_corpus: None,
        markdown_knowledge_semantic_gate: None,
        knowledge_scenarios: Vec::new(),
        repo_ast_index_ms: None,
        repo_ast_index_file_count: 0,
        repo_ast_index_symbol_count: 0,
        query_wall_ms: 0,
        query_sum_ms: 0,
        total_ms: elapsed_ms(repository_started_at.elapsed()),
        skip_reason,
        query_receipts: Vec::new(),
    }
}

struct LinkGraphBuild {
    index: Option<Arc<LinkGraphIndex>>,
    index_ms: Option<u128>,
    cache_backend: Option<String>,
    cache_status: Option<String>,
    cache_miss_reason: Option<String>,
    corpus: Option<RealRepoPrecisionLinkGraphCorpusReceipt>,
}

fn build_link_graph_index(
    entry: &RealRepoPrecisionCatalogEntry,
    options: &RealRepoPrecisionRunOptions,
    checkout_root: &Path,
    repo_id: &str,
) -> Result<LinkGraphBuild, String> {
    let link_graph_started_at = Instant::now();
    let link_graph_build = entry
        .gold_queries
        .iter()
        .any(|query| matches!(query.kind, RealRepoGoldQueryKind::LinkGraph))
        .then(|| {
            if options.prewarmed_resident_only {
                LinkGraphIndex::lookup_prewarmed_resident_local_cache_path_with_meta(
                    checkout_root,
                    entry.include_dirs.as_slice(),
                    entry.excluded_dirs.as_slice(),
                    options.link_graph_cache_path.as_path(),
                )
            } else {
                LinkGraphIndex::build_with_resident_local_cache_path_with_meta(
                    checkout_root,
                    entry.include_dirs.as_slice(),
                    entry.excluded_dirs.as_slice(),
                    options.link_graph_cache_path.as_path(),
                )
            }
            .map_err(|error| format!("failed to build LinkGraph index for `{repo_id}`: {error}"))
        })
        .transpose()?;
    let link_graph_index_ms = link_graph_build
        .as_ref()
        .map(|_| elapsed_ms(link_graph_started_at.elapsed()));
    let (
        link_graph_index,
        link_graph_cache_backend,
        link_graph_cache_status,
        link_graph_cache_miss_reason,
    ) = link_graph_build.map_or((None, None, None, None), |(index, meta)| {
        (
            Some(index),
            Some(meta.backend),
            Some(meta.status),
            meta.miss_reason,
        )
    });
    Ok(LinkGraphBuild {
        corpus: link_graph_index
            .as_ref()
            .map(|index| link_graph_corpus_receipt(index.as_ref())),
        index: link_graph_index,
        index_ms: link_graph_index_ms,
        cache_backend: link_graph_cache_backend,
        cache_status: link_graph_cache_status,
        cache_miss_reason: link_graph_cache_miss_reason,
    })
}

struct RepoAstBuild {
    index: Option<RepoAstAnalysisIndex>,
    index_ms: Option<u128>,
    file_count: usize,
    symbol_count: usize,
}

fn build_repo_ast_index(
    entry: &RealRepoPrecisionCatalogEntry,
    checkout_root: &Path,
    repo_ast_language_filters: &[String],
) -> RepoAstBuild {
    let repo_ast_started_at = Instant::now();
    let repo_ast_index = entry
        .gold_queries
        .iter()
        .any(|query| matches!(query.kind, RealRepoGoldQueryKind::RepoAst))
        .then(|| {
            build_repo_ast_analysis_index_from_checkout(
                checkout_root,
                &entry.repository,
                repo_ast_language_filters,
                entry.include_dirs.as_slice(),
                entry.excluded_dirs.as_slice(),
            )
        });
    let repo_ast_index_ms = repo_ast_index
        .as_ref()
        .map(|_| elapsed_ms(repo_ast_started_at.elapsed()));
    let repo_ast_index_file_count = repo_ast_index
        .as_ref()
        .map_or(0, RepoAstAnalysisIndex::file_count);
    let repo_ast_index_symbol_count = repo_ast_index
        .as_ref()
        .map_or(0, RepoAstAnalysisIndex::symbol_count);
    RepoAstBuild {
        index: repo_ast_index,
        index_ms: repo_ast_index_ms,
        file_count: repo_ast_index_file_count,
        symbol_count: repo_ast_index_symbol_count,
    }
}

struct QueryRun {
    receipts: Vec<crate::search::real_repo_precision::types::RealRepoPrecisionQueryReceipt>,
    wall_ms: u128,
    sum_ms: u128,
}

fn run_repository_queries(
    entry: &RealRepoPrecisionCatalogEntry,
    link_graph_index: Option<&Arc<LinkGraphIndex>>,
    repo_ast_index: Option<&RepoAstAnalysisIndex>,
) -> Result<QueryRun, String> {
    let query_started_at = Instant::now();
    let query_receipts = entry
        .gold_queries
        .iter()
        .map(|gold_query| {
            run_gold_query(
                gold_query,
                link_graph_index.map(Arc::as_ref),
                repo_ast_index,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let query_wall_ms = elapsed_ms(query_started_at.elapsed());
    let query_sum_ms = query_receipts
        .iter()
        .map(|receipt| receipt.query_ms)
        .sum::<u128>();
    Ok(QueryRun {
        receipts: query_receipts,
        wall_ms: query_wall_ms,
        sum_ms: query_sum_ms,
    })
}

fn link_graph_corpus_receipt(index: &LinkGraphIndex) -> RealRepoPrecisionLinkGraphCorpusReceipt {
    let mut document_count = 0;
    let mut markdown_document_count = 0;
    let mut org_document_count = 0;
    let mut total_word_count = 0;
    let mut path_prefix_counts = BTreeMap::<String, (usize, usize)>::new();

    for document in index.docs_by_id.values() {
        document_count += 1;
        total_word_count += document.word_count;
        if is_markdown_document(&document.path) {
            markdown_document_count += 1;
        }
        if has_extension(&document.path, "org") {
            org_document_count += 1;
        }
        let prefix = corpus_path_prefix(&document.path);
        let (prefix_document_count, prefix_word_count) =
            path_prefix_counts.entry(prefix).or_default();
        *prefix_document_count += 1;
        *prefix_word_count += document.word_count;
    }

    RealRepoPrecisionLinkGraphCorpusReceipt {
        document_count,
        markdown_document_count,
        org_document_count,
        total_word_count,
        path_prefix_counts: path_prefix_counts
            .into_iter()
            .map(|(prefix, (document_count, word_count))| {
                RealRepoPrecisionCorpusPathPrefixReceipt {
                    prefix,
                    document_count,
                    word_count,
                }
            })
            .collect(),
    }
}

fn is_markdown_document(path: &str) -> bool {
    has_extension(path, "md") || has_extension(path, "markdown")
}

fn has_extension(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(extension))
}

fn corpus_path_prefix(path: &str) -> String {
    let mut parts = path.split('/');
    let Some(first) = parts.next().filter(|part| !part.is_empty()) else {
        return "root".to_string();
    };
    if first == "docs"
        && let Some(second) = parts.next().filter(|part| !part.is_empty())
    {
        return format!("docs/{second}");
    }
    first.to_string()
}

fn run_gold_query(
    gold_query: &RealRepoGoldQuery,
    link_graph_index: Option<&LinkGraphIndex>,
    repo_ast_index: Option<&RepoAstAnalysisIndex>,
) -> Result<crate::search::real_repo_precision::types::RealRepoPrecisionQueryReceipt, String> {
    let query_started_at = Instant::now();
    match gold_query.kind {
        RealRepoGoldQueryKind::LinkGraph => {
            let index = link_graph_index
                .ok_or_else(|| format!("LinkGraph index is missing for `{}`", gold_query.id))?;
            let hits = index.search_planned(
                &gold_query.query,
                gold_query.limit,
                LinkGraphSearchOptions::default(),
            );
            Ok(evaluate_gold_query_paths_with_timing(
                gold_query,
                hits.1.into_iter().map(|hit| hit.path).collect(),
                elapsed_ms(query_started_at.elapsed()),
            ))
        }
        RealRepoGoldQueryKind::RepoAst => {
            let index = repo_ast_index
                .ok_or_else(|| format!("repo AST index is missing for `{}`", gold_query.id))?;
            let hits = index.search(Some(gold_query.query.as_str()), gold_query.limit);
            Ok(evaluate_gold_query_paths_with_timing(
                gold_query,
                hits.into_iter().map(|hit| hit.path).collect(),
                elapsed_ms(query_started_at.elapsed()),
            ))
        }
    }
}

fn merged_repo_ast_language_filters(gold_queries: &[RealRepoGoldQuery]) -> Vec<String> {
    let mut filters = gold_queries
        .iter()
        .filter(|query| matches!(query.kind, RealRepoGoldQueryKind::RepoAst))
        .flat_map(|query| query.language_filters.iter())
        .map(|filter| filter.trim().to_ascii_lowercase())
        .filter(|filter| !filter.is_empty())
        .collect::<Vec<_>>();
    filters.sort();
    filters.dedup();
    filters
}

fn elapsed_ms(duration: Duration) -> u128 {
    duration.as_millis()
}
