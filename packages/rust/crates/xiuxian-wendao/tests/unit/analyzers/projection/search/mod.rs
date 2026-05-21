use crate::analyzers::ProjectionPageKind;
use crate::analyzers::cache::RepositoryAnalysisCacheKey;
use crate::analyzers::repository_search_artifacts;

use crate::analyzers::projection::search::heuristic::heuristic_projected_page_matches;
use crate::analyzers::projection::search::ranking::{
    build_repo_projected_page_search, build_repo_projected_page_search_with_artifacts,
    ranked_projected_page_matches, scored_projected_page_matches,
};

#[test]
fn projected_page_search_uses_shared_tantivy_fuzzy_index_for_typos() {
    let analysis = test_analysis(vec![test_page(
        "repo:projection:reference:solve",
        "Solve Linear Systems",
        "docs/solve.md",
        ProjectionPageKind::Reference,
        vec!["solver".to_string(), "matrix".to_string()],
    )]);

    let matches = ranked_projected_page_matches(
        "slove",
        Some(ProjectionPageKind::Reference),
        &analysis,
        10,
        crate::search::FuzzySearchOptions::document_search(),
    );

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].1.title, "Solve Linear Systems");
    assert_eq!(matches[0].1.path, "docs/solve.md");
    assert!(matches[0].0 >= 45);
}

#[test]
fn scored_projected_page_matches_preserves_keyword_fallback() {
    let pages = vec![test_page(
        "repo:projection:reference:solve",
        "Linear Systems",
        "docs/solve.md",
        ProjectionPageKind::Reference,
        vec!["solver".to_string(), "matrix".to_string()],
    )];

    let matches = heuristic_projected_page_matches(
        "solver",
        Some(ProjectionPageKind::Reference),
        pages.as_slice(),
    );

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].0, 60);
}

#[test]
fn scored_projected_page_matches_exposes_fuzzy_ranked_hits_for_consumers() {
    let analysis = test_analysis(vec![test_page(
        "repo:projection:reference:solve",
        "Solve Linear Systems",
        "docs/solve.md",
        ProjectionPageKind::Reference,
        vec!["solver".to_string(), "matrix".to_string()],
    )]);

    let matches =
        scored_projected_page_matches("slove", Some(ProjectionPageKind::Reference), &analysis);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].1.title, "Solve Linear Systems");
    assert_eq!(matches[0].1.path, "docs/solve.md");
    assert!(matches[0].0 >= 45);
}

#[test]
fn projected_page_search_with_artifacts_preserves_ranked_results() {
    let analysis = test_analysis(vec![test_page(
        "repo:projection:reference:solve",
        "Solve Linear Systems",
        "docs/solve.md",
        ProjectionPageKind::Reference,
        vec!["solver".to_string(), "matrix".to_string()],
    )]);
    let cache_key = RepositoryAnalysisCacheKey {
        repo_id: "projection".to_string(),
        checkout_root: "/virtual/repos/projection".to_string(),
        analysis_identity: "analysis:projection".to_string(),
        checkout_revision: Some("fixture".to_string()),
        mirror_revision: None,
        tracking_revision: None,
        plugin_ids: vec!["docs".to_string()],
    };
    let query = crate::analyzers::RepoProjectedPageSearchQuery {
        repo_id: "repo".to_string(),
        query: "slove".to_string(),
        kind: Some(ProjectionPageKind::Reference),
        limit: 10,
    };

    let artifacts = repository_search_artifacts(&cache_key, &analysis)
        .unwrap_or_else(|error| panic!("artifacts should build: {error}"));
    let plain = build_repo_projected_page_search(&query, &analysis);
    let fast =
        build_repo_projected_page_search_with_artifacts(&query, &analysis, artifacts.as_ref());

    assert_eq!(fast.pages, plain.pages);
}

fn test_page(
    page_id: &str,
    title: &str,
    path: &str,
    kind: ProjectionPageKind,
    keywords: Vec<String>,
) -> crate::analyzers::ProjectedPageRecord {
    crate::analyzers::ProjectedPageRecord {
        repo_id: "repo".to_string(),
        page_id: page_id.to_string(),
        kind,
        title: title.to_string(),
        doc_ids: vec![page_id.to_string()],
        paths: vec![path.to_string()],
        format_hints: vec!["reference".to_string()],
        doc_id: format!("{page_id}:doc"),
        path: path.to_string(),
        keywords,
        ..crate::analyzers::ProjectedPageRecord::default()
    }
}

fn test_analysis(
    pages: Vec<crate::analyzers::ProjectedPageRecord>,
) -> crate::analyzers::RepositoryAnalysisOutput {
    crate::analyzers::RepositoryAnalysisOutput {
        docs: pages
            .into_iter()
            .map(|page| crate::analyzers::DocRecord {
                repo_id: page.repo_id.into(),
                doc_id: page.doc_id.into(),
                title: page.title,
                path: page.path.into(),
                format: page.format_hints.first().cloned(),
                doc_target: None,
            })
            .collect(),
        ..crate::analyzers::RepositoryAnalysisOutput::default()
    }
}
