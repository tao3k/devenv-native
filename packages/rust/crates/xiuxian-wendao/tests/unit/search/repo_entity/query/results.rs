use crate::search::SearchCorpusKind;
use crate::search::SearchPublicationStorageFormat;
use crate::search::repo_entity::query::tests::fixtures::published_repo_entity_fixture;
use crate::search::repo_entity::query::{
    search_repo_entity_example_results, search_repo_entity_import_results,
    search_repo_entity_module_results, search_repo_entity_symbol_results,
    summarize_repo_entity_overview,
};
use crate::test_support::assert_wendao_json_snapshot;

#[tokio::test]
async fn typed_repo_entity_search_reconstructs_module_symbol_and_example_results() {
    let fixture = published_repo_entity_fixture("alpha/repo", "solve", "Shows solve").await;
    let service = &fixture.service;
    let record = service
        .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, "alpha/repo")
        .await
        .unwrap_or_else(|| panic!("repo entity record"));
    let publication = record
        .publication
        .unwrap_or_else(|| panic!("repo entity publication"));
    assert_eq!(
        publication.storage_format,
        SearchPublicationStorageFormat::Parquet
    );
    assert!(
        service
            .repo_publication_parquet_path(
                SearchCorpusKind::RepoEntity,
                publication.table_name.as_str(),
            )
            .exists()
    );

    let module_result = search_repo_entity_module_results(service, "alpha/repo", "BaseModelica", 5)
        .await
        .unwrap_or_else(|error| panic!("module result: {error}"));
    assert_eq!(module_result.modules.len(), 1);
    assert_eq!(module_result.modules[0].qualified_name, "BaseModelica");
    assert!(
        module_result.module_hits[0]
            .projection_page_ids
            .as_ref()
            .is_some_and(|ids| ids.contains(
                &"repo:alpha/repo:projection:reference:module:module:BaseModelica".to_string()
            ))
    );

    let symbol_result = search_repo_entity_symbol_results(service, "alpha/repo", "solve", 5)
        .await
        .unwrap_or_else(|error| panic!("symbol result: {error}"));
    assert_eq!(symbol_result.symbols.len(), 1);
    assert_eq!(
        symbol_result.symbols[0].module_id.as_deref(),
        Some("module:BaseModelica")
    );
    assert_eq!(
        symbol_result.symbol_hits[0].audit_status.as_deref(),
        Some("verified")
    );
    assert_eq!(
        symbol_result.symbol_hits[0]
            .symbol
            .attributes
            .get("arity")
            .map(String::as_str),
        Some("0")
    );

    let example_result = search_repo_entity_example_results(service, "alpha/repo", "solve", 5)
        .await
        .unwrap_or_else(|error| panic!("example result: {error}"));
    assert_eq!(example_result.examples.len(), 1);
    assert_eq!(
        example_result.examples[0].summary.as_deref(),
        Some("Shows solve")
    );
    let import_result = search_repo_entity_import_results(
        service,
        &crate::analyzers::ImportSearchQuery {
            repo_id: "alpha/repo".to_string().into(),
            package: Some("SciMLBase".to_string()),
            module: Some("BaseModelica".to_string()),
            limit: 5,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("import result: {error}"));
    assert_eq!(import_result.imports.len(), 1);
    assert_eq!(import_result.imports[0].target_package, "SciMLBase");
    assert_eq!(import_result.imports[0].source_module, "BaseModelica");
    assert_wendao_json_snapshot(
        "search_plane_repo_entity_typed_results",
        serde_json::json!({
            "module_result": module_result,
            "symbol_result": symbol_result,
            "example_result": example_result,
            "import_result": import_result,
        }),
    );
}

#[tokio::test]
async fn repo_entity_overview_summary_reads_publication_counts() {
    let fixture = published_repo_entity_fixture("alpha/repo", "solve", "Shows solve").await;
    let summary = summarize_repo_entity_overview(&fixture.service, "alpha/repo")
        .await
        .unwrap_or_else(|error| panic!("repo entity overview summary: {error}"))
        .unwrap_or_else(|| panic!("repo entity overview summary should exist"));

    assert_eq!(summary.display_name.as_deref(), Some("BaseModelica"));
    assert_eq!(summary.source_revision.as_deref(), Some("rev-1"));
    assert_eq!(summary.module_count, 1);
    assert_eq!(summary.symbol_count, 1);
    assert_eq!(summary.example_count, 1);
    assert_eq!(summary.doc_count, 3);
}
