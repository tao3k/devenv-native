use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_invalid_root_element_with_document_level_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "invalid-root-element-decision-root.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.invalid_root_element");
    assert!(issue.title.contains("<definitions>"));
    assert!(issue.summary.contains("'<decision>'"));
    assert!(
        issue
            .why_it_failed
            .contains("expects one document root element named `<definitions>`")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Wrap the DMN content"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("root element is `<definitions>`")
    );
    assert_eq!(issue.evidence["root_element"], json!("decision"));
    assert_eq!(
        issue.evidence["document_root"]["element_name"],
        json!("decision")
    );
}

#[test]
fn dmn_linter_reports_missing_model_namespace_with_document_level_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("invalid-missing-model-namespace.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.missing_model_namespace");
    assert!(issue.title.contains("model namespace"));
    assert!(
        issue
            .why_it_failed
            .contains("needs one DMN model namespace declaration")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Add one DMN model namespace declaration"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("add one supported DMN model namespace declaration")
    );
    assert_eq!(
        issue.evidence["supported_model_namespaces"][0],
        json!("http://www.omg.org/spec/DMN/20180521/MODEL/")
    );
    assert_eq!(
        issue.evidence["supported_model_namespaces"][1],
        json!("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
}

#[test]
fn dmn_linter_reports_unsupported_model_namespace_with_document_level_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "invalid-unsupported-model-namespace-20211108.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_model_namespace");
    assert!(issue.title.contains("model namespace"));
    assert!(
        issue
            .summary
            .contains("https://www.omg.org/spec/DMN/20211108/MODEL/")
    );
    assert!(
        issue
            .why_it_failed
            .contains("recognizes the bounded DMN model namespaces")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("keep it as a non-executable artifact"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not rewrite decision logic")
    );
    assert_eq!(
        issue.evidence["model_namespace_uri"],
        json!("https://www.omg.org/spec/DMN/20211108/MODEL/")
    );
}

#[test]
fn dmn_linter_reports_missing_definitions_namespace_attribute() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "invalid-missing-definitions-namespace-attribute.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.missing_attribute");
    assert!(issue.summary.contains("<definitions>"));
    assert!(issue.summary.contains("namespace"));
    assert_eq!(issue.evidence["element"], json!("definitions"));
    assert_eq!(issue.evidence["attribute"], json!("namespace"));
}

#[test]
fn dmn_linter_reports_unsupported_top_level_import_with_document_level_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "unsupported-top-level-import-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_import");
    assert!(issue.title.contains("top-level imports"));
    assert!(issue.summary.contains("top-level `<import>`"));
    assert!(
        issue
            .why_it_failed
            .contains("does not resolve cross-document `<import>` dependencies")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("keep the file non-executable"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not delete top-level `<import>` declarations blindly")
    );
    assert_eq!(issue.evidence["import_count"], json!(1));
    assert_eq!(issue.evidence["document_root"]["import_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["imports"][0]["name"],
        json!("Partner Services")
    );
    assert_eq!(
        issue.evidence["document_root"]["imports"][0]["namespace"],
        json!("https://example.com/dmn/partner-services")
    );
    assert_eq!(
        issue.evidence["document_root"]["imports"][0]["location_uri"],
        json!("partner-services.dmn")
    );
    assert_eq!(
        issue.evidence["document_root"]["imports"][0]["import_type"],
        json!("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
}
