use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_input_data_only_document_with_artifact_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("metadata-only-input-data-20191111.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_input_data_artifact");
    assert!(issue.title.contains("input-data artifacts"));
    assert!(issue.summary.contains("<inputData>"));
    assert!(
        issue
            .why_it_failed
            .contains("input-data declarations as metadata only")
    );
    assert!(issue.why_it_failed.contains("name 'Applicant Input'"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent outputs"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "do not fabricate decision-table logic just from top-level `<inputData>` metadata"
    ));
    assert_eq!(issue.evidence["input_data_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["input_data_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["input_data"][0]["input_data_id"],
        json!("InputData_applicant")
    );
    assert_eq!(
        issue.evidence["document_root"]["input_data"][0]["name"],
        json!("Applicant Input")
    );
    assert_eq!(
        issue.evidence["document_root"]["input_data"][0]["variable"],
        json!(null)
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}

#[test]
fn dmn_linter_reports_knowledge_source_only_document_with_artifact_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-knowledge-source-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_knowledge_source_artifact");
    assert!(issue.title.contains("knowledge-source artifacts"));
    assert!(issue.summary.contains("<knowledgeSource>"));
    assert!(
        issue
            .why_it_failed
            .contains("knowledge-source declarations as governance metadata only")
    );
    assert!(issue.why_it_failed.contains("name 'Policy Authority'"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not fabricate decision-table rules"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "do not fabricate decision-table logic just from top-level `<knowledgeSource>` metadata"
    ));
    assert_eq!(issue.evidence["knowledge_source_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["knowledge_source_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["knowledge_sources"][0]["knowledge_source_id"],
        json!("KnowledgeSource_policy")
    );
    assert_eq!(
        issue.evidence["document_root"]["knowledge_sources"][0]["name"],
        json!("Policy Authority")
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}

#[test]
fn dmn_linter_reports_business_knowledge_model_only_document_with_artifact_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-business-knowledge-model-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "dmn.unsupported_business_knowledge_model_artifact"
    );
    assert!(issue.title.contains("business-knowledge-model artifacts"));
    assert!(issue.summary.contains("<businessKnowledgeModel>"));
    assert!(
        issue
            .why_it_failed
            .contains("does not execute top-level business-knowledge models directly")
    );
    assert!(issue.why_it_failed.contains("name 'Policy Source'"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not inline or approximate"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not fabricate decision-table logic just from top-level `<businessKnowledgeModel>` metadata")
    );
    assert_eq!(issue.evidence["business_knowledge_model_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["business_knowledge_model_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["business_knowledge_models"][0]["business_knowledge_model_id"],
        json!("BKM_policy_source")
    );
    assert_eq!(
        issue.evidence["document_root"]["business_knowledge_models"][0]["name"],
        json!("Policy Source")
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}

#[test]
fn dmn_linter_keeps_generic_missing_decision_guidance_without_known_root_artifacts() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-empty-definitions-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.missing_decision");
    assert!(issue.title.contains("no decisions"));
    assert!(issue.summary.contains("does not contain any `<decision>`"));
    assert_eq!(
        issue.evidence["document_root"]["item_definition_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["input_data_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["knowledge_source_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["business_knowledge_model_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["organization_unit_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["performance_indicator_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["text_annotation_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["association_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["document_root"]["element_collection_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["document_root"]["group_count"], json!(0));
    assert_eq!(issue.evidence["document_root"]["dmndi_count"], json!(0));
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}
