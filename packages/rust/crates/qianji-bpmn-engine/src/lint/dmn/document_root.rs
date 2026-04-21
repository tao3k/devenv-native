use super::evidence::{augment_evidence, root_context};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn invalid_dmn_xml_issue(source_id: &str, message: &str) -> LintIssue {
    LintIssue::new(
        "dmn.invalid_xml",
        "DMN XML is not well-formed",
        format!("Source '{source_id}' cannot be parsed as DMN XML: {message}"),
        "The DMN parser stops before decision-table validation when the XML tree is malformed.",
        vec![
            "Repair the XML structure first: close tags, fix attributes, and restore valid nesting."
                .to_string(),
            "Preserve decision ids, table ids, and rule ids while repairing XML syntax."
                .to_string(),
        ],
        format!(
            "Repair the XML syntax in DMN source '{source_id}' so it becomes well-formed without changing decision semantics. Preserve ids, hit policies, and rule ordering while fixing XML structure."
        ),
        json!({
            "source_id": source_id,
            "parser_message": message,
        }),
    )
}

pub(super) fn missing_dmn_root_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.missing_root_element",
        "DMN file has no root XML element",
        format!("Source '{source_id}' does not contain a root DMN XML element."),
        "The linter cannot discover `<definitions>` or any decision content when the file is empty or structurally missing its root node.",
        vec![
            "Add one DMN XML root element, typically `<definitions>`, around the decision content."
                .to_string(),
            "Move decisions and decision tables inside that root element.".to_string(),
        ],
        format!(
            "Rewrite DMN source '{source_id}' so it has one valid root element, typically `<definitions>`, and place all decision content inside it."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn invalid_dmn_root_element_issue(
    source_id: &str,
    element: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::new(
        "dmn.invalid_root_element",
        "DMN document root must be `<definitions>`",
        format!(
            "Source '{source_id}' starts with root element '<{element}>' instead of `<definitions>`."
        ),
        format!(
            "The bounded DMN parser expects one document root element named `<definitions>` before any decisions, decision services, or artifacts.{}",
            root_context(snapshot)
        ),
        vec![
            "Wrap the DMN content in one `<definitions>` root element.".to_string(),
            "Keep existing decision ids, decision-table ids, and rule ids stable while moving the current content under `<definitions>`.".to_string(),
            "Declare one supported DMN model namespace on that root element before retrying parser-level repairs.".to_string(),
        ],
        format!(
            "Rewrite DMN source '{source_id}' so its root element is `<definitions>`. Preserve existing decision and rule identifiers, move the current DMN content under that root, and declare one supported DMN model namespace there."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "root_element": element,
            }),
            snapshot,
            None,
        ),
    )
}
