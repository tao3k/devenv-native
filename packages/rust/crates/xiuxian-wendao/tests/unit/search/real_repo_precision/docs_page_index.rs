use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serial_test::serial;
use xiuxian_wendao_julia::integration_support::probe_wendaograph_page_index_host_request_with_fixture;

use crate::analyzers::resolve_registered_repository_source;
use crate::link_graph::{LinkGraphIndex, LinkGraphSearchOptions, PageIndexNode};
use crate::search::real_repo_precision::{
    RealRepoGoldQuery, RealRepoGoldQueryKind, RealRepoPrecisionCatalogEntry,
    RealRepoPrecisionRunOptions, default_real_repo_precision_catalog, evaluate_gold_query_paths,
};

const RUN_WENDAOGRAPH_DOCS_CORPUS_PAGE_INDEX_LIVE_PROOF_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_DOCS_CORPUS_PAGE_INDEX_LIVE_PROOF_TEST";

#[derive(Debug)]
struct DocsPageIndexFixtureReceipt {
    queries: usize,
    selected_documents: usize,
    nodes: usize,
    edges: usize,
    seeds: usize,
}

#[test]
fn docs_query_hits_seed_page_index_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_root = temp.path().join("repo");
    fs::create_dir_all(repo_root.join("docs/03_features"))?;
    fs::write(
        repo_root.join("docs/03_features/wendao-agentic-retrieval.md"),
        "# Wendao Agentic Retrieval\n\nAutonomous Query Planning.\n\n## Saliency-Aware Priority Scoring\n\nAgentic retrieval expands query plans.\n",
    )?;
    let index = LinkGraphIndex::build_with_filters(&repo_root, &["docs".to_string()], &[])?;
    let queries = vec![RealRepoGoldQuery {
        id: "docs-wendao-agentic-retrieval".to_string(),
        kind: RealRepoGoldQueryKind::LinkGraph,
        query: "Wendao Agentic Retrieval Autonomous Query Planning".to_string(),
        limit: 10,
        must_hit_paths: vec!["docs/03_features/wendao-agentic-retrieval.md".to_string()],
        required_top_path: None,
        language_filters: Vec::new(),
    }];
    let fixture_dir = temp.path().join("page_index_fixture");

    let receipt =
        write_docs_page_index_fixture_from_queries(&index, queries.as_slice(), &fixture_dir)
            .map_err(|error| format!("write docs PageIndex fixture: {error}"))?;

    assert_eq!(receipt.queries, 1);
    assert_eq!(receipt.selected_documents, 1);
    assert!(receipt.nodes >= 1);
    assert!(receipt.seeds >= 1);
    assert!(fixture_dir.join("page_index_nodes.tsv").exists());
    assert!(fixture_dir.join("page_index_edges.tsv").exists());
    assert!(fixture_dir.join("page_index_seeds.tsv").exists());
    Ok(())
}

#[test]
#[serial]
fn docs_corpus_page_index_live_proof_runs_real_wendaograph_when_enabled() -> Result<(), String> {
    if std::env::var_os(RUN_WENDAOGRAPH_DOCS_CORPUS_PAGE_INDEX_LIVE_PROOF_TEST_ENV).is_none() {
        eprintln!(
            "skipping docs-corpus PageIndex live proof; set {RUN_WENDAOGRAPH_DOCS_CORPUS_PAGE_INDEX_LIVE_PROOF_TEST_ENV}=1 and WENDAOGRAPH_PACKAGE_DIR"
        );
        return Ok(());
    }

    let options = RealRepoPrecisionRunOptions::from_env();
    let mut catalog = default_real_repo_precision_catalog();
    let entry = catalog
        .pop()
        .unwrap_or_else(|| panic!("default real-repo precision catalog should have one entry"));
    let index = build_real_repo_link_graph_index(&entry, &options)?;
    let docs_queries = entry
        .gold_queries
        .iter()
        .filter(|query| {
            query.id.starts_with("docs-") && matches!(query.kind, RealRepoGoldQueryKind::LinkGraph)
        })
        .cloned()
        .collect::<Vec<_>>();
    assert!(docs_queries.len() >= 9);

    let temp = tempfile::tempdir()
        .map_err(|error| format!("create docs PageIndex fixture temp dir: {error}"))?;
    let fixture = write_docs_page_index_fixture_from_queries(
        index.as_ref(),
        docs_queries.as_slice(),
        temp.path(),
    )?;
    assert!(fixture.selected_documents >= 9);
    assert!(fixture.seeds >= 9);
    assert!(fixture.nodes >= fixture.seeds);

    let report = probe_wendaograph_page_index_host_request_with_fixture(temp.path(), 2)
        .map_err(|error| format!("run docs-corpus PageIndex live proof: {error}"))?;

    assert_eq!(report.sample_count, 2);
    assert!(report.frontier_rows > 0);
    assert!(report.trace_rows > 0);
    assert!(report.warm_median_ms >= report.warm_min_ms);
    assert!(report.warm_p95_ms >= report.warm_median_ms);
    eprintln!(
        "wendaograph_docs_corpus_page_index_live_proof_summary docs_queries={} selected_documents={} page_index_nodes={} page_index_edges={} page_index_seeds={} frontier_rows={} trace_rows={} first_ms={:.3} warm_median_ms={:.3} warm_p95_ms={:.3} warm_max_ms={:.3}",
        fixture.queries,
        fixture.selected_documents,
        fixture.nodes,
        fixture.edges,
        fixture.seeds,
        report.frontier_rows,
        report.trace_rows,
        report.first_ms,
        report.warm_median_ms,
        report.warm_p95_ms,
        report.warm_max_ms
    );
    Ok(())
}

fn build_real_repo_link_graph_index(
    entry: &RealRepoPrecisionCatalogEntry,
    options: &RealRepoPrecisionRunOptions,
) -> Result<std::sync::Arc<LinkGraphIndex>, String> {
    let materialized = resolve_registered_repository_source(
        &entry.repository,
        options.project_root.as_path(),
        options.sync_mode.as_git_sync_mode(),
    )
    .map_err(|error| format!("materialize real repo `{}`: {error}", entry.repository.id))?;
    if !materialized.checkout_root.is_dir() {
        return Err(format!(
            "checkout `{}` is missing for docs PageIndex live proof",
            materialized.checkout_root.display()
        ));
    }
    LinkGraphIndex::build_with_resident_local_cache_path_with_meta(
        materialized.checkout_root.as_path(),
        entry.include_dirs.as_slice(),
        entry.excluded_dirs.as_slice(),
        options.link_graph_cache_path.as_path(),
    )
    .map(|(index, _)| index)
    .map_err(|error| format!("build real docs LinkGraph index: {error}"))
}

fn write_docs_page_index_fixture_from_queries(
    index: &LinkGraphIndex,
    queries: &[RealRepoGoldQuery],
    fixture_dir: &Path,
) -> Result<DocsPageIndexFixtureReceipt, String> {
    let selected_doc_ids = selected_doc_ids_from_queries(index, queries)?;
    let mut node_rows = Vec::new();
    let mut edge_rows = Vec::new();
    let mut seed_rows = Vec::new();
    let mut node_ids = BTreeSet::new();

    for doc_id in &selected_doc_ids {
        let roots = index
            .trees_by_doc
            .get(doc_id)
            .ok_or_else(|| format!("PageIndex tree missing for selected doc `{doc_id}`"))?;
        let seed_node_id = roots
            .first()
            .map(|root| root.node_id.clone())
            .ok_or_else(|| format!("PageIndex tree is empty for selected doc `{doc_id}`"))?;
        seed_rows.push(vec![
            seed_node_id,
            "1".to_string(),
            "docs_gold_query".to_string(),
        ]);
        for root in roots {
            collect_page_index_rows(
                doc_id,
                None,
                0,
                root,
                &mut node_rows,
                &mut edge_rows,
                &mut node_ids,
            );
        }
    }

    for seed in &seed_rows {
        if !node_ids.contains(&seed[0]) {
            return Err(format!(
                "seed node `{}` missing from PageIndex fixture",
                seed[0]
            ));
        }
    }

    fs::create_dir_all(fixture_dir).map_err(|error| {
        format!(
            "create docs PageIndex fixture dir `{}`: {error}",
            fixture_dir.display()
        )
    })?;
    write_tsv_file(
        &fixture_dir.join("page_index_nodes.tsv"),
        &[
            "node_id",
            "page_id",
            "parent_id",
            "depth",
            "rank",
            "title",
            "summary",
            "line_start",
            "line_end",
            "token_count",
        ],
        node_rows.as_slice(),
    )?;
    write_tsv_file(
        &fixture_dir.join("page_index_edges.tsv"),
        &["source_id", "target_id", "edge_kind", "weight"],
        edge_rows.as_slice(),
    )?;
    write_tsv_file(
        &fixture_dir.join("page_index_seeds.tsv"),
        &["node_id", "weight", "seed_kind"],
        seed_rows.as_slice(),
    )?;

    Ok(DocsPageIndexFixtureReceipt {
        queries: queries.len(),
        selected_documents: selected_doc_ids.len(),
        nodes: node_rows.len(),
        edges: edge_rows.len(),
        seeds: seed_rows.len(),
    })
}

fn selected_doc_ids_from_queries(
    index: &LinkGraphIndex,
    queries: &[RealRepoGoldQuery],
) -> Result<BTreeSet<String>, String> {
    let docs_by_path = index
        .docs_by_id
        .values()
        .map(|doc| (doc.path.as_str(), doc.id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut selected = BTreeSet::new();

    for query in queries {
        let hits =
            index.search_planned(&query.query, query.limit, LinkGraphSearchOptions::default());
        let observed_paths = hits.1.into_iter().map(|hit| hit.path).collect::<Vec<_>>();
        let receipt = evaluate_gold_query_paths(query, observed_paths);
        if !receipt.passed {
            return Err(format!(
                "docs PageIndex seed query `{}` failed: missing={:?} observed_top={:?}",
                receipt.query_id, receipt.missing_paths, receipt.observed_top_path
            ));
        }
        for path in &query.must_hit_paths {
            let doc_id = docs_by_path.get(path.as_str()).ok_or_else(|| {
                format!("docs PageIndex seed path `{path}` was not indexed as a document")
            })?;
            selected.insert((*doc_id).to_string());
        }
    }

    Ok(selected)
}

fn collect_page_index_rows(
    page_id: &str,
    parent_id: Option<&str>,
    depth: usize,
    node: &PageIndexNode,
    node_rows: &mut Vec<Vec<String>>,
    edge_rows: &mut Vec<Vec<String>>,
    node_ids: &mut BTreeSet<String>,
) {
    let parent_id = node.parent_id.as_deref().or(parent_id).unwrap_or_default();
    let rank = node_rows.len();
    let (line_start, line_end) = node.metadata.line_range;
    node_ids.insert(node.node_id.clone());
    if !parent_id.is_empty() {
        edge_rows.push(vec![
            parent_id.to_string(),
            node.node_id.clone(),
            "hierarchy".to_string(),
            "1".to_string(),
        ]);
    }
    node_rows.push(vec![
        node.node_id.clone(),
        page_id.to_string(),
        parent_id.to_string(),
        depth.to_string(),
        rank.to_string(),
        node.title.clone(),
        node.summary.clone().unwrap_or_default(),
        line_start.to_string(),
        line_end.to_string(),
        node.metadata.token_count.to_string(),
    ]);

    for child in &node.children {
        collect_page_index_rows(
            page_id,
            Some(node.node_id.as_str()),
            depth + 1,
            child,
            node_rows,
            edge_rows,
            node_ids,
        );
    }
}

fn write_tsv_file(path: &Path, header: &[&str], rows: &[Vec<String>]) -> Result<(), String> {
    let mut content = header.join("\t");
    content.push('\n');
    for row in rows {
        let cells = row
            .iter()
            .map(|cell| sanitize_tsv_cell(cell))
            .collect::<Vec<_>>();
        content.push_str(&cells.join("\t"));
        content.push('\n');
    }
    fs::write(path, content)
        .map_err(|error| format!("write docs PageIndex fixture `{}`: {error}", path.display()))
}

fn sanitize_tsv_cell(cell: &str) -> String {
    cell.replace(['\t', '\n', '\r'], " ")
}
