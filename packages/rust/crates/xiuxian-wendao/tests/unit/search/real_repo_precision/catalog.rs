use xiuxian_wendao_core::repo_intelligence::RepositoryRefreshPolicy;

use crate::search::real_repo_precision::{
    RealRepoGoldQuery, RealRepoGoldQueryKind, RealRepoPrecisionCatalogEntry,
    default_real_repo_precision_catalog,
};

#[test]
fn default_catalog_uses_managed_repo_contracts() {
    let catalog = default_real_repo_precision_catalog();
    assert_eq!(catalog.len(), 2);
    let entry = catalog
        .iter()
        .find(|entry| entry.repository.id == "xiuxian-artisan-workshop")
        .unwrap_or_else(|| panic!("missing xiuxian-artisan-workshop catalog entry"));

    assert_artisan_catalog_contract(entry);

    let pi_wendao = catalog
        .iter()
        .find(|entry| entry.repository.id == "pi-wendao")
        .unwrap_or_else(|| panic!("missing pi-wendao catalog entry"));
    assert_pi_wendao_catalog_contract(pi_wendao);
}

fn assert_artisan_catalog_contract(entry: &RealRepoPrecisionCatalogEntry) {
    assert_eq!(entry.repository.id, "xiuxian-artisan-workshop");
    assert!(entry.repository.path.is_none());
    assert_eq!(
        entry.repository.url.as_deref(),
        Some("https://github.com/tao3k/xiuxian-artisan-workshop.git")
    );
    assert_eq!(entry.repository.refresh, RepositoryRefreshPolicy::Manual);
    assert!(entry.include_dirs.iter().any(|path| path == "semantic"));
    assert_semantic_object_gold_query(
        &entry.gold_queries,
        "semantic-decision-repo-native-authority",
        "semantic/objects/decision/semantic-ssot-repo-native-first.md",
    );
    assert_docs_gold_query(
        &entry.gold_queries,
        "docs-polyglot-compute-orchestrator-rfc",
        "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md",
    );
    assert!(entry.repository.plugins.is_empty());
    assert!(entry.knowledge_scenarios.len() >= 7);
    assert!(
        entry
            .knowledge_scenarios
            .iter()
            .all(|scenario| !scenario.query_variants.is_empty())
    );
}

fn assert_pi_wendao_catalog_contract(pi_wendao: &RealRepoPrecisionCatalogEntry) {
    assert_eq!(
        pi_wendao.repository.path.as_deref(),
        Some(std::path::Path::new(".data/pi-wendao"))
    );
    assert_eq!(
        pi_wendao.repository.url.as_deref(),
        Some("https://github.com/tao3k/pi-wendao.git")
    );
    assert_eq!(pi_wendao.include_dirs, vec![".".to_string()]);
    assert_docs_gold_query(
        &pi_wendao.gold_queries,
        "pi-wendao-readme-subagents-host",
        "README.md",
    );
    assert_docs_gold_query(
        &pi_wendao.gold_queries,
        "pi-wendao-named-workflows-brainstorm-cache",
        "docs/named-workflows.md",
    );
    assert!(pi_wendao.repository.plugins.is_empty());
    assert!(
        pi_wendao
            .knowledge_scenarios
            .iter()
            .any(|scenario| scenario.id == "pi-wendao-agent-workflow-boundary")
    );
}

fn assert_docs_gold_query(gold_queries: &[RealRepoGoldQuery], query_id: &str, expected_path: &str) {
    assert!(
        gold_queries.iter().any(|query| query.id == query_id
            && matches!(query.kind, RealRepoGoldQueryKind::LinkGraph)
            && query.limit >= 20
            && query
                .must_hit_paths
                .iter()
                .any(|path| path == expected_path)),
        "missing docs gold query `{query_id}` for `{expected_path}`"
    );
}

fn assert_semantic_object_gold_query(
    gold_queries: &[RealRepoGoldQuery],
    query_id: &str,
    expected_path: &str,
) {
    assert!(
        gold_queries.iter().any(|query| query.id == query_id
            && matches!(query.kind, RealRepoGoldQueryKind::LinkGraph)
            && query.must_hit_paths == vec![expected_path.to_string()]
            && query.required_top_path.as_deref() == Some(expected_path)),
        "missing semantic object gold query `{query_id}` for `{expected_path}`"
    );
}
