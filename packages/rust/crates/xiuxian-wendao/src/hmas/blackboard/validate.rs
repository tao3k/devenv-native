//! `hmas::blackboard::validate` owns Wendao hmas blackboard validate behavior.

use super::extract::{ExtractedBlock, collect_blocks};
use super::report::HmasValidationReport;
use crate::hmas::protocol::{
    HmasConclusionPayload, HmasDigitalThreadPayload, HmasEvidencePayload, HmasRecordKind,
    HmasSourceNode, HmasTaskPayload,
};
use std::collections::HashSet;
use std::path::Path;

fn has_empty_source_nodes(source_nodes: &[HmasSourceNode]) -> bool {
    source_nodes
        .iter()
        .any(|node| node.node_id.trim().is_empty())
}

fn push_invalid_json(
    report: &mut HmasValidationReport,
    block: &ExtractedBlock,
    label: &str,
    err: impl std::fmt::Display,
) {
    report.push_issue(
        block.line,
        "invalid_json_payload",
        format!("failed to decode {label} payload: {err}"),
        Some(block.kind),
    );
}

fn validate_task_block(report: &mut HmasValidationReport, block: &ExtractedBlock) {
    report.task_count += 1;
    match serde_json::from_str::<HmasTaskPayload>(&block.json_payload) {
        Ok(payload) => {
            if payload.requirement_id.trim().is_empty() {
                report.push_issue(
                    block.line,
                    "missing_requirement_id",
                    "task.requirement_id must be non-empty",
                    Some(block.kind),
                );
            }
            if payload.hard_constraints.is_empty() {
                report.push_issue(
                    block.line,
                    "missing_hard_constraints",
                    "task.hard_constraints must be non-empty",
                    Some(block.kind),
                );
            }
        }
        Err(err) => push_invalid_json(report, block, "task", err),
    }
}

fn validate_evidence_block(report: &mut HmasValidationReport, block: &ExtractedBlock) {
    report.evidence_count += 1;
    match serde_json::from_str::<HmasEvidencePayload>(&block.json_payload) {
        Ok(payload) => {
            if payload.requirement_id.trim().is_empty() {
                report.push_issue(
                    block.line,
                    "missing_requirement_id",
                    "evidence.requirement_id must be non-empty",
                    Some(block.kind),
                );
            }
        }
        Err(err) => push_invalid_json(report, block, "evidence", err),
    }
}

fn validate_conclusion_block(
    report: &mut HmasValidationReport,
    block: &ExtractedBlock,
    conclusion_requirements: &mut Vec<(String, usize)>,
) {
    report.conclusion_count += 1;
    match serde_json::from_str::<HmasConclusionPayload>(&block.json_payload) {
        Ok(payload) => {
            let requirement_id = payload.requirement_id.trim().to_string();
            if requirement_id.is_empty() {
                report.push_issue(
                    block.line,
                    "missing_requirement_id",
                    "conclusion.requirement_id must be non-empty",
                    Some(block.kind),
                );
            } else {
                conclusion_requirements.push((requirement_id, block.line));
            }
            if !(0.0..=1.0).contains(&payload.confidence_score) {
                report.push_issue(
                    block.line,
                    "invalid_confidence_score",
                    "conclusion.confidence_score must be between 0 and 1",
                    Some(block.kind),
                );
            }
        }
        Err(err) => push_invalid_json(report, block, "conclusion", err),
    }
}

fn validate_digital_thread_block(
    report: &mut HmasValidationReport,
    block: &ExtractedBlock,
    digital_thread_requirements: &mut HashSet<String>,
) {
    report.digital_thread_count += 1;
    match serde_json::from_str::<HmasDigitalThreadPayload>(&block.json_payload) {
        Ok(payload) => {
            let requirement_id = payload.requirement_id.trim();
            if requirement_id.is_empty() {
                report.push_issue(
                    block.line,
                    "missing_requirement_id",
                    "digital_thread.requirement_id must be non-empty",
                    Some(block.kind),
                );
            } else {
                digital_thread_requirements.insert(requirement_id.to_string());
            }

            if payload.source_nodes_accessed.is_empty() {
                report.push_issue(
                    block.line,
                    "missing_source_nodes",
                    "digital_thread.source_nodes_accessed must be non-empty",
                    Some(block.kind),
                );
            } else if has_empty_source_nodes(&payload.source_nodes_accessed) {
                report.push_issue(
                    block.line,
                    "empty_source_node_id",
                    "digital_thread.source_nodes_accessed[*].node_id must be non-empty",
                    Some(block.kind),
                );
            }

            if payload.hard_constraints_checked.is_empty() {
                report.push_issue(
                    block.line,
                    "missing_constraints_checked",
                    "digital_thread.hard_constraints_checked must be non-empty",
                    Some(block.kind),
                );
            }
            if !(0.0..=1.0).contains(&payload.confidence_score) {
                report.push_issue(
                    block.line,
                    "invalid_confidence_score",
                    "digital_thread.confidence_score must be between 0 and 1",
                    Some(block.kind),
                );
            }
        }
        Err(err) => push_invalid_json(report, block, "digital_thread", err),
    }
}

/// Validate HMAS protocol blocks from markdown text.
///
/// This parser accepts heading-driven or fenced-tag driven HMAS JSON blocks and
/// enforces payload structure and cross-block consistency checks.
#[must_use]
pub fn validate_blackboard_markdown(markdown: &str) -> HmasValidationReport {
    let mut report = HmasValidationReport::ok();
    let blocks = collect_blocks(markdown, &mut report);

    let mut digital_thread_requirements = HashSet::new();
    let mut conclusion_requirements = Vec::new();

    for block in blocks {
        match block.kind {
            HmasRecordKind::Task => validate_task_block(&mut report, &block),
            HmasRecordKind::Evidence => validate_evidence_block(&mut report, &block),
            HmasRecordKind::Conclusion => {
                validate_conclusion_block(&mut report, &block, &mut conclusion_requirements);
            }
            HmasRecordKind::DigitalThread => {
                validate_digital_thread_block(
                    &mut report,
                    &block,
                    &mut digital_thread_requirements,
                );
            }
        }
    }

    for (requirement_id, line) in conclusion_requirements {
        if !digital_thread_requirements.contains(&requirement_id) {
            report.push_issue(
                line,
                "missing_digital_thread",
                format!(
                    "conclusion requirement_id={requirement_id} has no matching digital_thread payload"
                ),
                Some(HmasRecordKind::Conclusion),
            );
        }
    }

    report
}

/// Validate HMAS protocol blocks from a file path.
///
/// # Errors
///
/// Returns an error string when the markdown file cannot be read from disk.
pub fn validate_blackboard_file(path: &Path) -> Result<HmasValidationReport, String> {
    let content = std::fs::read_to_string(path).map_err(|err| {
        format!(
            "failed to read blackboard markdown file {}: {err}",
            path.display()
        )
    })?;
    Ok(validate_blackboard_markdown(&content))
}
