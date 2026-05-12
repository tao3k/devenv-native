use crate::analyzers::{DocRecord, RepoProjectedRetrievalContextQuery, RepositoryAnalysisOutput};

use super::{BootstrapProjectionCache, projected_page_id_variants};

#[test]
fn bootstrap_projection_cache_resolves_page_index_and_retrieval_context() {
    let analysis = RepositoryAnalysisOutput {
        docs: vec![
            doc("main", "docs/search.md", "Search Strategy Flow"),
            doc("main", "docs/validation.md", "Validation Path"),
        ],
        ..RepositoryAnalysisOutput::default()
    };
    let cache = BootstrapProjectionCache::build(&analysis)
        .unwrap_or_else(|error| panic!("build projection cache: {error}"));
    let page_id = cache.pages[0].page_id.clone();
    let tree = cache
        .page_index_tree(&crate::analyzers::RepoProjectedPageIndexTreeQuery {
            repo_id: "main".to_owned(),
            page_id: page_id.clone(),
        })
        .unwrap_or_else(|error| panic!("resolve cached page-index tree: {error}"));
    let node_id = tree
        .tree
        .as_ref()
        .and_then(|tree| tree.roots.first())
        .map(|node| node.node_id.clone())
        .unwrap_or_else(|| panic!("cached tree should expose a root node"));

    let context = cache
        .retrieval_context(&RepoProjectedRetrievalContextQuery {
            repo_id: "main".to_owned(),
            page_id,
            node_id: Some(node_id),
            related_limit: 4,
        })
        .unwrap_or_else(|error| panic!("resolve cached retrieval context: {error}"));

    assert_eq!(context.repo_id, "main");
    assert!(context.node_context.is_some());
}

#[test]
fn projected_page_id_variants_include_collapsed_and_expanded_doc_ids() {
    let variants = projected_page_id_variants(
        "main",
        "repo:main:projection:explanation:doc:repo:main:doc:docs/search.md",
    );

    assert!(variants.contains(&"repo:main:projection:explanation:doc:docs/search.md".to_owned()));
    assert!(
        variants
            .iter()
            .any(|variant| variant.ends_with("repo:main:doc:docs/search.md"))
    );
}

fn doc(repo_id: &str, path: &str, title: &str) -> DocRecord {
    DocRecord {
        repo_id: repo_id.to_owned().into(),
        doc_id: format!("repo:{repo_id}:doc:{path}").into(),
        title: title.to_owned(),
        path: path.to_owned().into(),
        format: Some("md".to_owned()),
        doc_target: None,
    }
}
