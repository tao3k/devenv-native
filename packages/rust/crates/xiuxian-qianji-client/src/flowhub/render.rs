//! Markdown rendering for `qianji-client flowhub` reports.

use std::fmt::Write as _;
use std::path::PathBuf;

use serde_json::json;

use super::model::{
    FlowhubCheckStatus, FlowhubCliOutput, FlowhubGeneratedFile, FlowhubSourcePairSummary,
    FlowhubValidation,
};
use super::parse::{FlowhubAction, FlowhubOutputFormat};

pub(crate) struct RenderInput {
    pub(crate) action: FlowhubAction,
    pub(crate) project_root: PathBuf,
    pub(crate) cache_agent_root: PathBuf,
    pub(crate) flowhub_root: Option<PathBuf>,
    pub(crate) generated_files: Vec<FlowhubGeneratedFile>,
    pub(crate) source_pairs: Vec<FlowhubSourcePairSummary>,
    pub(crate) validation: FlowhubValidation,
    pub(crate) output_format: FlowhubOutputFormat,
}

pub(crate) fn render_output(input: RenderInput) -> FlowhubCliOutput {
    let passed = input.validation.passed();
    let rendered = match input.output_format {
        FlowhubOutputFormat::Markdown => render_markdown(&input, passed),
        FlowhubOutputFormat::Json => render_json(&input, passed),
    };

    FlowhubCliOutput {
        action: input.action,
        passed,
        rendered,
        generated_paths: input
            .generated_files
            .iter()
            .map(|file| file.path.clone())
            .collect(),
        generated_files: input.generated_files,
        source_pairs: input.source_pairs,
    }
}

fn render_markdown(input: &RenderInput, passed: bool) -> String {
    let mut rendered = String::new();
    rendered.push_str(match input.action {
        FlowhubAction::Init => "# Qianji Client Flowhub Init\n\n",
        FlowhubAction::Check => "# Qianji Client Flowhub Check\n\n",
        FlowhubAction::Scenarios => "# Qianji Client Flowhub Scenarios\n\n",
    });
    let _ = writeln!(rendered, "- Project root: {}", input.project_root.display());
    let _ = writeln!(
        rendered,
        "- Agent root: {}",
        input.cache_agent_root.display()
    );
    let flowhub_contract = input.flowhub_root.as_deref().map_or_else(
        || "embedded agent-coding contract".to_string(),
        |path| path.display().to_string(),
    );
    let _ = writeln!(rendered, "- Flowhub contract: {flowhub_contract}");
    let _ = writeln!(
        rendered,
        "- Status: {}\n",
        if passed { "passed" } else { "failed" }
    );

    if !input.generated_files.is_empty() {
        rendered.push_str("## Files\n\n");
        for file in &input.generated_files {
            let status = if file.created { "created" } else { "existing" };
            let _ = writeln!(rendered, "- {status}: {}", file.path.display());
        }
        rendered.push('\n');
    }

    if !input.source_pairs.is_empty() {
        rendered.push_str("## Scenarios\n\n");
        for source_pair in &input.source_pairs {
            let _ = writeln!(rendered, "- `{}`", source_pair.scenario_id);
            let _ = writeln!(rendered, "  - Org: {}", source_pair.org_source.display());
            let _ = writeln!(rendered, "  - BPMN: {}", source_pair.bpmn_source.display());
            let _ = writeln!(rendered, "  - Process: `{}`", source_pair.bpmn_process_id);
        }
        rendered.push('\n');
    }

    rendered.push_str("## Validation\n\n");
    let _ = writeln!(
        rendered,
        "- Flowhub contract: {}",
        validation_label(input.validation.flowhub_contract)
    );
    let _ = writeln!(
        rendered,
        "- Generated files: {}",
        validation_label(input.validation.generated_files)
    );
    let _ = writeln!(
        rendered,
        "- Generated metadata: {}",
        validation_label(input.validation.generated_metadata)
    );
    let _ = writeln!(
        rendered,
        "- Org lint: {}",
        validation_label(input.validation.org_lint)
    );
    if !input.validation.diagnostics.is_empty() {
        rendered.push_str("\n## Diagnostics\n\n");
        for diagnostic in &input.validation.diagnostics {
            let _ = writeln!(rendered, "- {diagnostic}");
        }
    }

    rendered
}

fn render_json(input: &RenderInput, passed: bool) -> String {
    let flowhub_root = input
        .flowhub_root
        .as_ref()
        .map(|path| path.display().to_string());
    let value = json!({
        "action": action_label(input.action),
        "passed": passed,
        "projectRoot": input.project_root.display().to_string(),
        "cacheAgentRoot": input.cache_agent_root.display().to_string(),
        "flowhubRoot": flowhub_root,
        "generatedFiles": input.generated_files.iter().map(|file| {
            json!({
                "path": file.path.display().to_string(),
                "created": file.created,
            })
        }).collect::<Vec<_>>(),
        "sourcePairs": input.source_pairs.iter().map(|source_pair| {
            json!({
                "scenarioId": source_pair.scenario_id,
                "orgSource": source_pair.org_source.display().to_string(),
                "orgSha256": source_pair.org_sha256,
                "bpmnSource": source_pair.bpmn_source.display().to_string(),
                "bpmnSha256": source_pair.bpmn_sha256,
                "bpmnProcessId": source_pair.bpmn_process_id,
            })
        }).collect::<Vec<_>>(),
        "validation": {
            "flowhubContractPassed": input.validation.flowhub_contract.as_bool(),
            "generatedFilesPresent": input.validation.generated_files.as_bool(),
            "generatedMetadataMatched": input.validation.generated_metadata.as_bool(),
            "orgLintPassed": input.validation.org_lint.as_bool(),
            "diagnostics": input.validation.diagnostics,
            "generatedMetadataFailures": input.validation.generated_metadata_failures.iter().map(|failure| {
                json!({
                    "path": failure.path.display().to_string(),
                    "key": failure.key,
                    "actual": failure.actual,
                    "expected": failure.expected,
                })
            }).collect::<Vec<_>>(),
        },
    });
    match serde_json::to_string_pretty(&value) {
        Ok(rendered) => rendered,
        Err(error) => format!(
            "{{\"action\":\"{}\",\"passed\":false,\"serializationError\":\"{}\"}}",
            action_label(input.action),
            error
        ),
    }
}

fn validation_label(status: FlowhubCheckStatus) -> &'static str {
    if status.is_passed() {
        "passed"
    } else {
        "failed"
    }
}

fn action_label(action: FlowhubAction) -> &'static str {
    match action {
        FlowhubAction::Init => "init",
        FlowhubAction::Check => "check",
        FlowhubAction::Scenarios => "scenarios",
    }
}
