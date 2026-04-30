use crate::bpmn_parse_api::BpmnSourceFile;
use crate::ir_node_api::BpmnNodeKind;
use crate::ir_package_api::BpmnPackage;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use serde_json::json;

const SUPPORTED_INTERACTION_TYPES: &[&str] = &["input", "confirm", "choice", "choice_input"];
const LEGACY_CUSTOM_QNAME_MARKERS: &[&str] = &[
    "qianji:config",
    "qianji:interaction",
    "qianji:choice",
    "qianji:choices",
    "qianji:freeText",
    "qianji:inputs",
    "qianji:outputSchema",
    "qianji:outputs",
    "qianji:prompt",
    "qianji:question",
    "qianji:result",
    "qianji:tools",
    "qianji:toolScope",
];

pub(super) fn human_task_interaction_issues(
    source: &BpmnSourceFile,
    package: &BpmnPackage,
) -> Vec<LintIssue> {
    let legacy = legacy_custom_qname_issues(source);
    if !legacy.is_empty() {
        return legacy;
    }
    native_form_issues(source, package)
}

fn legacy_custom_qname_issues(source: &BpmnSourceFile) -> Vec<LintIssue> {
    let Some((marker, start)) = LEGACY_CUSTOM_QNAME_MARKERS
        .iter()
        .filter_map(|marker| source.contents.find(marker).map(|start| (*marker, start)))
        .min_by_key(|(_, start)| *start)
    else {
        return Vec::new();
    };
    vec![LintIssue::new(
        "bpmn.legacy_custom_interaction_xml",
        "Custom QName interaction XML is not supported",
        format!(
            "Source '{}' contains legacy custom interaction XML marker '{}'.",
            source.source_id, marker
        ),
        "Executable human-task interaction metadata must use native BPMN documentation, ioSpecification, dataInputAssociation, and dataOutputAssociation elements. The bounded engine does not provide a compatibility mode for custom QName interaction XML.",
        vec![
            "Remove custom QName interaction elements from the BPMN source.".to_string(),
            "Declare human-task inputs with `<bpmn:ioSpecification>` data inputs named `interactionType`, `question`, `choices`, and `freeText` as needed.".to_string(),
            "Declare the completion field with a data output named `answer` and a `dataOutputAssociation` targetRef.".to_string(),
        ],
        format!(
            "Repair BPMN source '{}' by replacing legacy custom interaction XML with native BPMN IO metadata and preserving BPMN ids and sequence flows.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "marker": marker,
            "contract": "bpmn.native_human_task_io.v1",
        }),
    )
    .with_source_diagnostic(LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(start, start + marker.len()),
        "replace legacy custom interaction XML",
        "Use native BPMN IO metadata for executable human-task interaction.",
    ))]
}

fn native_form_issues(source: &BpmnSourceFile, package: &BpmnPackage) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    for process in &package.processes {
        for node in &process.nodes {
            if !matches!(node.kind, BpmnNodeKind::UserTask | BpmnNodeKind::ManualTask) {
                continue;
            }
            let Some(form) = node.human_task_form.as_ref() else {
                continue;
            };
            if !SUPPORTED_INTERACTION_TYPES.contains(&form.interaction_type.as_ref()) {
                issues.push(unsupported_interaction_type_issue(
                    source,
                    node.bpmn_id.as_ref(),
                    form.interaction_type.as_ref(),
                ));
                continue;
            }
            if requires_choices(form.interaction_type.as_ref())
                && form.choices_ref.is_none()
                && form.choices.is_empty()
            {
                issues.push(missing_choices_issue(
                    source,
                    node.bpmn_id.as_ref(),
                    form.interaction_type.as_ref(),
                ));
                continue;
            }
            if requires_choices(form.interaction_type.as_ref())
                && form.choices_ref.is_some()
                && !form.choices.is_empty()
            {
                issues.push(ambiguous_choices_issue(source, node.bpmn_id.as_ref()));
                continue;
            }
            if form.free_text_fields.len() > 1 {
                issues.push(unsupported_free_text_cardinality_issue(
                    source,
                    node.bpmn_id.as_ref(),
                    form.free_text_fields.len(),
                ));
                continue;
            }
            if form.result_output.is_none() {
                issues.push(missing_result_output_issue(source, node.bpmn_id.as_ref()));
            }
        }
    }
    issues
}

fn requires_choices(interaction_type: &str) -> bool {
    matches!(interaction_type, "choice" | "choice_input")
}

fn unsupported_interaction_type_issue(
    source: &BpmnSourceFile,
    task_id: &str,
    interaction_type: &str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_native_interaction_type",
        "Native human-task interaction type is unsupported",
        format!(
            "user/manual task '{task_id}' declares unsupported interactionType '{interaction_type}'."
        ),
        "Native BPMN IO metadata must declare one supported interactionType literal so hosts can render the blocked work deterministically.",
        vec![
            "Set the `interactionType` data input assignment literal to one of: input, confirm, choice, choice_input.".to_string(),
            "Keep the interaction type in native BPMN IO metadata, not in custom XML elements.".to_string(),
        ],
        format!(
            "Repair BPMN source '{}' by changing task '{task_id}' interactionType to input, confirm, choice, or choice_input.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "task_id": task_id,
            "interaction_type": interaction_type,
        }),
    )
}

fn missing_choices_issue(
    source: &BpmnSourceFile,
    task_id: &str,
    interaction_type: &str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.missing_native_choice_contract",
        "Choice interaction is missing choices",
        format!(
            "user/manual task '{task_id}' interactionType '{interaction_type}' does not declare static choices or a dynamic choices source."
        ),
        "Choice interactions must have exactly one choices source so hosts can render valid selectable values.",
        vec![
            "For dynamic choices, map a `choices` data input from one upstream variable with `dataInputAssociation/sourceRef`.".to_string(),
            "For static choices, assign a JSON array literal to the `choices` data input.".to_string(),
        ],
        format!(
            "Repair BPMN source '{}' by declaring one native BPMN IO choices source on task '{task_id}'.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "task_id": task_id,
            "interaction_type": interaction_type,
        }),
    )
}

fn ambiguous_choices_issue(source: &BpmnSourceFile, task_id: &str) -> LintIssue {
    LintIssue::new(
        "bpmn.ambiguous_native_choices",
        "Choice interaction has multiple choices sources",
        format!(
            "user/manual task '{task_id}' declares both dynamic choices and static choices."
        ),
        "Native human-task IO metadata must resolve to one choices source before host rendering.",
        vec![
            "Keep either one dynamic `choices` dataInputAssociation sourceRef or one static JSON choices assignment.".to_string(),
            "Do not combine dynamic and static choice sources on the same task.".to_string(),
        ],
        format!(
            "Repair BPMN source '{}' by keeping exactly one choices source on task '{task_id}'.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "task_id": task_id,
        }),
    )
}

fn unsupported_free_text_cardinality_issue(
    source: &BpmnSourceFile,
    task_id: &str,
    field_count: usize,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_native_free_text_cardinality",
        "Native free-text field cardinality is unsupported",
        format!("user/manual task '{task_id}' declares {field_count} free-text fields."),
        "The current flat human-task completion ABI supports at most one supplemental free-text field.",
        vec![
            "Keep at most one `freeText` field in the native BPMN IO assignment literal.".to_string(),
            "Model additional user-entered fields as separate user/manual tasks or derive them in a following service task.".to_string(),
        ],
        format!(
            "Repair BPMN source '{}' by reducing task '{task_id}' to at most one freeText field.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "task_id": task_id,
            "field_count": field_count,
        }),
    )
}

fn missing_result_output_issue(source: &BpmnSourceFile, task_id: &str) -> LintIssue {
    LintIssue::new(
        "bpmn.missing_native_answer_output",
        "Native human-task answer output is missing",
        format!("user/manual task '{task_id}' does not map data output `answer` to a result variable."),
        "Human-task completion requires one native BPMN dataOutputAssociation target so the engine can merge the submitted answer into workflow variables.",
        vec![
            "Declare a data output named `answer` in the task `ioSpecification`.".to_string(),
            "Map that output with `dataOutputAssociation` to the workflow variable that should receive the reply.".to_string(),
        ],
        format!(
            "Repair BPMN source '{}' by adding an `answer` dataOutputAssociation targetRef to task '{task_id}'.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "task_id": task_id,
        }),
    )
}
