use xiuxian_wendao::analyzers::ImportSearchQuery;
use xiuxian_wendao::analyzers::canonical_import_query_text;

#[test]
fn import_search_cache_identity_uses_both_filters() {
    let left = canonical_import_query_text(&ImportSearchQuery {
        repo_id: "alpha/repo".to_string(),
        package: Some("SciMLBase".to_string()),
        module: Some("BaseModelica".to_string()),
        limit: 10,
    });
    let right = canonical_import_query_text(&ImportSearchQuery {
        repo_id: "alpha/repo".to_string(),
        package: Some("SciMLBase".to_string()),
        module: Some("OtherModule".to_string()),
        limit: 10,
    });

    assert_ne!(left, right);
}
