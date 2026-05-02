use super::{load_episteme_manifest, workspace_root};

#[test]
#[ignore = "requires the real wendao-episteme submodule checkout"]
fn load_episteme_manifest_accepts_real_wendao_episteme_submodule() {
    let episteme_root = workspace_root().join("wendao-episteme");

    let report = load_episteme_manifest(&episteme_root)
        .unwrap_or_else(|error| panic!("load real episteme manifest: {error}"));

    assert_eq!(report.name.as_deref(), Some("wendao-episteme"));
    assert!(report.policy_query_count >= 1);
    assert!(report.diagnostic_mapping_count >= 1);
    assert!(report.repair_prompt_count >= 1);
    assert!(report.source_evolution_skill_count >= 1);
    assert!(
        report
            .policy_queries
            .iter()
            .all(|query| query.statement_mode == "select_only")
    );
    assert!(report.policy_queries.iter().any(|query| {
        query.id == "johnny-decimal.anchor-id-validation"
            && query.path == "policies/johnny_decimal/validation.sql"
    }));
}
