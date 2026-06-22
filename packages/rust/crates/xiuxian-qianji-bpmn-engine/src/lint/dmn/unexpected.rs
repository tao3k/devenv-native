use crate::dmn_model_api::DmnSourceFile;
use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unexpected_dmn_issue(source: &DmnSourceFile, error: &BpmnEngineError) -> LintIssue {
    LintIssue::from_parts(
        "dmn.unexpected_engine_error",
        "Unexpected DMN lint error",
        format!(
            "DMN linting for source '{}' returned unexpected engine error: {error}",
            source.source_id
        ),
        "The linter expected a DMN parse or validation error but received a broader engine error, which usually indicates a missing lint mapping.",
        vec![
            "Inspect the emitted evidence before rewriting the DMN source.".to_string(),
            "If the DMN appears valid, extend the linter mapping rather than forcing an unsafe rewrite.".to_string(),
        ],
        format!(
            "Do not rewrite DMN source '{}' blindly. First inspect the unexpected engine error and repair only the concrete problem proven by the evidence.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "engine_error": error.to_string(),
        }),
    )
}
