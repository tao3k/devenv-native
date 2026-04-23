use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unexpected_bpmn_issue(source: &BpmnSourceFile, error: &BpmnEngineError) -> LintIssue {
    LintIssue::new(
        "bpmn.unexpected_engine_error",
        "Unexpected BPMN lint error",
        format!(
            "BPMN linting for source '{}' returned unexpected engine error: {error}",
            source.source_id
        ),
        "The linter expected a parse or validation error but received a broader engine error, which usually indicates a missing lint mapping.",
        vec![
            "Inspect the source and the emitted evidence before making broad edits.".to_string(),
            "If the source is valid, extend the linter mapping instead of forcing a speculative workflow rewrite.".to_string(),
        ],
        format!(
            "Do not rewrite BPMN source '{}' blindly. First inspect the unexpected engine error and repair only the concrete structure proven by the evidence.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "engine_error": error.to_string(),
        }),
    )
}
