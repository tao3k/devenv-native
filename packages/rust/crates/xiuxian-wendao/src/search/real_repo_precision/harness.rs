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
#[cfg(feature = "julia")]
use crate::search::real_repo_precision::semantic_gate::{
    attach_markdown_knowledge_semantic_query_evidence, evaluate_markdown_knowledge_semantic_gate,
};
use crate::search::real_repo_precision::types::{
    RealRepoGoldQuery, RealRepoGoldQueryKind, RealRepoMarkdownKnowledgeSemanticGateReceipt,
    RealRepoPrecisionCatalogEntry, RealRepoPrecisionCorpusPathPrefixReceipt,
    RealRepoPrecisionLinkGraphCorpusReceipt, RealRepoPrecisionQueryReceipt,
    RealRepoPrecisionRepositoryReceipt, RealRepoPrecisionRunOptions, RealRepoPrecisionRunStatus,
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
    let mut markdown_knowledge_semantic_gate =
        evaluate_markdown_knowledge_semantic_gate_if_enabled(
            &checkout_root.join("semantic"),
            &entry.gold_queries,
        )?;
    let query_result = run_repository_queries(entry, link_graph.index.as_ref())?;

    if let Some(gate) = markdown_knowledge_semantic_gate.as_mut() {
        attach_markdown_knowledge_semantic_query_evidence_if_enabled(gate, &query_result.receipts);
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
        query_wall_ms: query_result.wall_ms,
        query_sum_ms: query_result.sum_ms,
        total_ms: elapsed_ms(repository_started_at.elapsed()),
        skip_reason: None,
        query_receipts: query_result.receipts,
    })
}

#[cfg(feature = "julia")]
fn evaluate_markdown_knowledge_semantic_gate_if_enabled(
    semantic_root: &Path,
    gold_queries: &[RealRepoGoldQuery],
) -> Result<Option<RealRepoMarkdownKnowledgeSemanticGateReceipt>, String> {
    Ok(
        evaluate_markdown_knowledge_semantic_gate(semantic_root, gold_queries)?
            .map(|evaluation| evaluation.receipt),
    )
}

#[cfg(not(feature = "julia"))]
fn evaluate_markdown_knowledge_semantic_gate_if_enabled(
    semantic_root: &Path,
    gold_queries: &[RealRepoGoldQuery],
) -> Result<Option<RealRepoMarkdownKnowledgeSemanticGateReceipt>, String> {
    let _ = (semantic_root, gold_queries);
    Ok(None)
}

#[cfg(feature = "julia")]
fn attach_markdown_knowledge_semantic_query_evidence_if_enabled(
    receipt: &mut RealRepoMarkdownKnowledgeSemanticGateReceipt,
    query_receipts: &[RealRepoPrecisionQueryReceipt],
) {
    attach_markdown_knowledge_semantic_query_evidence(receipt, query_receipts);
}

#[cfg(not(feature = "julia"))]
fn attach_markdown_knowledge_semantic_query_evidence_if_enabled(
    receipt: &mut RealRepoMarkdownKnowledgeSemanticGateReceipt,
    query_receipts: &[RealRepoPrecisionQueryReceipt],
) {
    let _ = (receipt, query_receipts);
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

struct QueryRun {
    receipts: Vec<crate::search::real_repo_precision::types::RealRepoPrecisionQueryReceipt>,
    wall_ms: u128,
    sum_ms: u128,
}

fn run_repository_queries(
    entry: &RealRepoPrecisionCatalogEntry,
    link_graph_index: Option<&Arc<LinkGraphIndex>>,
) -> Result<QueryRun, String> {
    let query_started_at = Instant::now();
    let query_receipts = entry
        .gold_queries
        .iter()
        .map(|gold_query| run_gold_query(gold_query, link_graph_index.map(Arc::as_ref)))
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
    let (
        document_count,
        markdown_document_count,
        org_document_count,
        total_word_count,
        path_prefix_counts,
    ) = index.docs_by_id.values().fold(
        (0, 0, 0, 0, BTreeMap::<String, (usize, usize)>::new()),
        |(
            document_count,
            markdown_document_count,
            org_document_count,
            total_word_count,
            mut path_prefix_counts,
        ),
         document| {
            let prefix = corpus_path_prefix(&document.path);
            let (prefix_document_count, prefix_word_count) =
                path_prefix_counts.entry(prefix).or_default();
            *prefix_document_count += 1;
            *prefix_word_count += document.word_count;
            (
                document_count + 1,
                markdown_document_count + usize::from(is_markdown_document(&document.path)),
                org_document_count + usize::from(has_extension(&document.path, "org")),
                total_word_count + document.word_count,
                path_prefix_counts,
            )
        },
    );

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
            let mut observed_paths = hits.1.into_iter().map(|hit| hit.path).collect::<Vec<_>>();
            if !gold_query.language_filters.is_empty() {
                observed_paths.retain(|path| {
                    path_matches_language_filters(path, gold_query.language_filters.as_slice())
                });
            }
            Ok(evaluate_gold_query_paths_with_timing(
                gold_query,
                observed_paths,
                elapsed_ms(query_started_at.elapsed()),
            ))
        }
    }
}

fn path_matches_language_filters(path: &str, language_filters: &[String]) -> bool {
    let Some(extension) = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
    else {
        return false;
    };
    language_filters
        .iter()
        .any(|language| language_matches_extension(language.as_str(), extension))
}

fn language_matches_extension(language: &str, extension: &str) -> bool {
    match language.trim().to_ascii_lowercase().as_str() {
        "rust" => extension.eq_ignore_ascii_case("rs"),
        "python" => extension.eq_ignore_ascii_case("py"),
        "typescript" => {
            extension.eq_ignore_ascii_case("ts") || extension.eq_ignore_ascii_case("tsx")
        }
        "javascript" => {
            extension.eq_ignore_ascii_case("js") || extension.eq_ignore_ascii_case("jsx")
        }
        "julia" => extension.eq_ignore_ascii_case("jl"),
        "modelica" => extension.eq_ignore_ascii_case("mo"),
        "markdown" => {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        }
        other => extension.eq_ignore_ascii_case(other),
    }
}

fn elapsed_ms(duration: Duration) -> u128 {
    duration.as_millis()
}
