use crate::analyzers::ImportSearchQuery;

#[test]
fn canonical_import_query_text_preserves_package_and_module_identity() {
    let query = ImportSearchQuery {
        repo_id: "alpha/repo".to_string(),
        package: Some("SciMLBase".to_string()),
        module: Some("BaseModelica".to_string()),
        limit: 10,
    };

    assert_eq!(
        super::canonical_import_query_text(&query),
        "package=SciMLBase;module=BaseModelica"
    );
}

#[test]
fn canonical_import_query_text_uses_stable_wildcards_for_missing_filters() {
    let query = ImportSearchQuery {
        repo_id: "alpha/repo".to_string(),
        package: Some("SciMLBase".to_string()),
        module: None,
        limit: 10,
    };

    assert_eq!(
        super::canonical_import_query_text(&query),
        "package=SciMLBase;module=*"
    );
}
