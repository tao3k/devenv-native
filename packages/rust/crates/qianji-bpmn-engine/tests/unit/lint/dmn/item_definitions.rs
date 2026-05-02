use crate::lint::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_item_definition_only_document_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "metadata-only-item-definition-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_item_definition_document");
    assert!(issue.title.contains("item definitions"));
    assert!(issue.summary.contains("<itemDefinition>"));
    assert!(
        issue
            .why_it_failed
            .contains("item definitions as non-executable type metadata only")
    );
    assert!(issue.why_it_failed.contains("name 'tLoanOffer'"));
    assert!(issue.why_it_failed.contains("1 direct itemComponent(s)"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not translate `<itemDefinition>` structures"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "do not fabricate decision-table logic just from top-level `<itemDefinition>` metadata"
    ));
    assert_eq!(issue.evidence["item_definition_count"], json!(1));
    assert_eq!(
        issue.evidence["document_root"]["item_definition_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["document_root"]["item_definitions"][0]["item_definition_id"],
        json!("ItemDefinition_loan_offer")
    );
    assert_eq!(
        issue.evidence["document_root"]["item_definitions"][0]["name"],
        json!("tLoanOffer")
    );
    assert_eq!(
        issue.evidence["document_root"]["item_definitions"][0]["is_collection"],
        json!(false)
    );
    assert_eq!(
        issue.evidence["document_root"]["item_definitions"][0]["item_components"][0]["item_component_id"],
        json!("ItemDefinition_loan_offer_amount")
    );
    assert_eq!(
        issue.evidence["document_root"]["item_definitions"][0]["item_components"][0]["type_ref"],
        json!("number")
    );
    assert_eq!(issue.evidence["document_decision_count"], json!(0));
}
