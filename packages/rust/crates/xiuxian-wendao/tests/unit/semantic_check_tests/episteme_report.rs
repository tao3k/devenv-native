use super::{
    EpistemeLoadReport, EpistemePolicyQueryReport, SemanticCheckResult, format_result_as_xml,
};

#[test]
fn format_result_as_xml_includes_loaded_episteme_summary() {
    let result = SemanticCheckResult {
        status: "pass".to_string(),
        issue_count: 0,
        issues: Vec::new(),
        summary: "Found 0 errors and 0 warnings across 0 documents".to_string(),
        file_reports: Vec::new(),
        episteme: Some(EpistemeLoadReport {
            name: Some("test-episteme".to_string()),
            schema_version: Some(1),
            manifest_path: "/tmp/episteme/episteme.toml".to_string(),
            root_path: "/tmp/episteme".to_string(),
            policy_query_count: 1,
            diagnostic_mapping_count: 1,
            repair_prompt_count: 1,
            repair_guard_count: 1,
            source_evolution_skill_count: 1,
            policy_queries: vec![EpistemePolicyQueryReport {
                id: "johnny-decimal.anchor-id-validation".to_string(),
                framework: Some("johnny-decimal".to_string()),
                path: "policies/johnny_decimal/validation.sql".to_string(),
                statement_mode: "select_only".to_string(),
            }],
        }),
    };

    let xml = format_result_as_xml(&result);

    assert!(xml.contains("<episteme status=\"loaded\""));
    assert!(xml.contains("name=\"test-episteme\""));
    assert!(xml.contains("policy_queries=\"1\""));
    assert!(xml.contains("source_evolution_skills=\"1\""));
    assert!(xml.contains("id=\"johnny-decimal.anchor-id-validation\""));
}
