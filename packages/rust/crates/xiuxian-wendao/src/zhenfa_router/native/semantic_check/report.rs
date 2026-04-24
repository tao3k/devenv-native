//! Reporting helpers for semantic checking.

use std::fmt::Write as _;
use std::path::Path;

use super::docs_governance;
use super::episteme::EpistemeLoadReport;
use super::types::{
    FileAuditReport, FuzzySuggestionData, IssueLocation, SemanticCheckResult, SemanticIssue,
};

fn issue_penalty(count: usize, weight: u8) -> u8 {
    u8::try_from(count).map_or(u8::MAX, |count| count.saturating_mul(weight))
}

/// Build per-file audit reports with health scores.
pub(super) fn build_file_reports(
    issues: &[SemanticIssue],
    docs: &[String],
) -> Vec<FileAuditReport> {
    let mut reports = Vec::new();
    let deduped_docs = collect_report_doc_paths(docs, issues);

    for doc_id in &deduped_docs {
        let doc_identity = report_doc_identity(doc_id);
        let doc_issues: Vec<_> = issues
            .iter()
            .filter(|issue| report_doc_identity(&issue.doc) == doc_identity)
            .collect();
        let error_count = doc_issues.iter().filter(|i| i.severity == "error").count();
        let warning_count = doc_issues
            .iter()
            .filter(|i| i.severity == "warning")
            .count();

        let health_score = (100u8)
            .saturating_sub(issue_penalty(error_count, 20))
            .saturating_sub(issue_penalty(warning_count, 5));

        reports.push(FileAuditReport {
            path: doc_id.clone(),
            health_score,
            error_count,
            warning_count,
        });
    }

    reports
}

pub(super) fn collect_report_doc_paths(
    issues_docs: &[String],
    issues: &[SemanticIssue],
) -> Vec<String> {
    let mut report_docs: Vec<String> = Vec::new();
    let mut identities: Vec<String> = Vec::new();

    for doc_path in issues_docs
        .iter()
        .cloned()
        .chain(issues.iter().map(|issue| issue.doc.clone()))
    {
        let identity = report_doc_identity(&doc_path);
        if let Some(existing_idx) = identities.iter().position(|existing| existing == &identity) {
            if should_prefer_report_path(&report_docs[existing_idx], &doc_path) {
                report_docs[existing_idx] = doc_path;
            }
            continue;
        }

        identities.push(identity);
        report_docs.push(doc_path);
    }

    report_docs
}

pub(super) fn report_doc_identity(doc_path: &str) -> String {
    canonicalize_doc_path(doc_path).map_or_else(
        || normalize_doc_path_key(doc_path),
        |path| normalize_doc_path_key(&path),
    )
}

pub(super) fn canonicalize_doc_path(doc_path: &str) -> Option<String> {
    let path = Path::new(doc_path);
    path.is_file()
        .then(|| path.canonicalize().ok())
        .flatten()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

pub(super) fn normalize_doc_path_key(doc_path: &str) -> String {
    doc_path.replace('\\', "/")
}

pub(super) fn should_prefer_report_path(existing: &str, candidate: &str) -> bool {
    let existing_path = Path::new(existing);
    let candidate_path = Path::new(candidate);

    match (existing_path.is_absolute(), candidate_path.is_absolute()) {
        (true, false) => return true,
        (false, true) => return false,
        _ => {}
    }

    if candidate.len() != existing.len() {
        return candidate.len() < existing.len();
    }

    candidate < existing
}

fn file_health_status(health_score: u8) -> &'static str {
    if health_score >= 80 {
        "HEALTHY"
    } else if health_score >= 50 {
        "DEGRADED"
    } else {
        "UNHEALTHY"
    }
}

fn append_file_reports_xml(output: &mut String, file_reports: &[FileAuditReport]) {
    if file_reports.is_empty() {
        return;
    }

    output.push_str("  <files>\n");
    for file_report in file_reports {
        let _ = writeln!(
            output,
            "    <file path=\"{}\" health=\"{}\" score=\"{}\">",
            xml_escape(&file_report.path),
            file_health_status(file_report.health_score),
            file_report.health_score
        );
        let _ = writeln!(output, "      <errors>{}</errors>", file_report.error_count);
        let _ = writeln!(
            output,
            "      <warnings>{}</warnings>",
            file_report.warning_count
        );
        output.push_str("    </file>\n");
    }
    output.push_str("  </files>\n");
}

fn append_issue_location_xml(output: &mut String, location: &IssueLocation) {
    let byte_range_attr = if let Some((start, end)) = location.byte_range {
        format!(" byte_start=\"{start}\" byte_end=\"{end}\"")
    } else {
        String::new()
    };
    let _ = writeln!(
        output,
        "      <location line=\"{}\" path=\"{}\"{}/>",
        location.line,
        xml_escape(&location.heading_path),
        byte_range_attr
    );
}

fn append_fuzzy_suggestion_xml(output: &mut String, fuzzy: &FuzzySuggestionData) {
    output.push_str("      <fuzzy_suggestion>\n");
    let _ = writeln!(
        output,
        "        <text>Pattern '{}' found with {:.0}% similarity.</text>",
        xml_escape(&fuzzy.suggested_pattern),
        fuzzy.confidence * 100.0
    );
    let _ = writeln!(
        output,
        "        <replacement_drawer>{}</replacement_drawer>",
        xml_escape(&fuzzy.replacement_drawer)
    );
    let _ = writeln!(
        output,
        "        <confidence>{:.2}</confidence>",
        fuzzy.confidence
    );
    if let Some(ref location) = fuzzy.source_location {
        let _ = writeln!(
            output,
            "        <source_location>{}</source_location>",
            xml_escape(location)
        );
    }
    output.push_str("      </fuzzy_suggestion>\n");
}

fn append_issue_xml(output: &mut String, issue: &SemanticIssue) {
    let _ = writeln!(
        output,
        "    <issue severity=\"{}\" code=\"{}\">",
        issue.severity.to_uppercase(),
        issue_type_to_code(&issue.issue_type)
    );
    let _ = writeln!(
        output,
        "      <message>{}</message>",
        xml_escape(&issue.message)
    );
    let _ = writeln!(output, "      <doc>{}</doc>", xml_escape(&issue.doc));
    let _ = writeln!(
        output,
        "      <node_id>{}</node_id>",
        xml_escape(&issue.node_id)
    );
    if let Some(ref location) = issue.location {
        append_issue_location_xml(output, location);
    }
    if let Some(ref suggestion) = issue.suggestion {
        let _ = writeln!(
            output,
            "      <suggestion>{}</suggestion>",
            xml_escape(suggestion)
        );
    }
    if let Some(ref fuzzy) = issue.fuzzy_suggestion {
        append_fuzzy_suggestion_xml(output, fuzzy);
    }
    output.push_str("    </issue>\n");
}

fn append_issues_xml(output: &mut String, issues: &[SemanticIssue]) {
    if issues.is_empty() {
        return;
    }

    output.push_str("  <issues>\n");
    for issue in issues {
        append_issue_xml(output, issue);
    }
    output.push_str("  </issues>\n");
}

fn append_episteme_xml(output: &mut String, episteme: &EpistemeLoadReport) {
    let schema_version = episteme
        .schema_version
        .map_or_else(String::new, |version| version.to_string());
    let name = episteme.name.as_deref().unwrap_or("");

    let _ = writeln!(
        output,
        "  <episteme status=\"loaded\" name=\"{}\" schema_version=\"{}\" manifest_path=\"{}\" root_path=\"{}\">",
        xml_escape(name),
        xml_escape(&schema_version),
        xml_escape(&episteme.manifest_path),
        xml_escape(&episteme.root_path)
    );
    let _ = writeln!(
        output,
        "    <counts policy_queries=\"{}\" diagnostic_mappings=\"{}\" repair_prompts=\"{}\" repair_guards=\"{}\" source_evolution_skills=\"{}\"/>",
        episteme.policy_query_count,
        episteme.diagnostic_mapping_count,
        episteme.repair_prompt_count,
        episteme.repair_guard_count,
        episteme.source_evolution_skill_count
    );

    if !episteme.policy_queries.is_empty() {
        output.push_str("    <policy_queries>\n");
        for query in &episteme.policy_queries {
            let framework = query.framework.as_deref().unwrap_or("");
            let _ = writeln!(
                output,
                "      <policy_query id=\"{}\" framework=\"{}\" path=\"{}\" statement_mode=\"{}\"/>",
                xml_escape(&query.id),
                xml_escape(framework),
                xml_escape(&query.path),
                xml_escape(&query.statement_mode)
            );
        }
        output.push_str("    </policy_queries>\n");
    }

    output.push_str("  </episteme>\n");
}

/// Format the check result as XML-Lite (Blueprint v2.2).
pub(super) fn format_result_as_xml(result: &SemanticCheckResult) -> String {
    let mut output = String::new();

    let _ = writeln!(
        output,
        "<wendao_audit_report version=\"2.9\" engine=\"anchoR-sentinel\" status=\"{}\" issue_count=\"{}\">",
        result.status, result.issue_count
    );

    let _ = writeln!(output, "  <summary>{}</summary>", result.summary);
    if let Some(episteme) = &result.episteme {
        append_episteme_xml(&mut output, episteme);
    }
    append_file_reports_xml(&mut output, &result.file_reports);
    append_issues_xml(&mut output, &result.issues);

    output.push_str("</wendao_audit_report>\n");
    output
}

/// Convert issue type to Blueprint diagnostic code.
pub(super) fn issue_type_to_code(issue_type: &str) -> &'static str {
    match issue_type {
        "dead_link" => "ERR_DEAD_LINK",
        "deprecated_ref" => "WARN_DEPRECATED_REF",
        "contract_violation" => "ERR_CONTRACT_VIOLATION",
        "id_collision" => "ERR_DUPLICATE_ID",
        "content_drift" => "WARN_CONTENT_DRIFT",
        "missing_content_hash" => "INFO_MISSING_HASH",
        "missing_identity" => "ERR_MISSING_IDENTITY",
        "legacy_syntax" => "WARN_LEGACY_SYNTAX",
        "invalid_observation_language" => "ERR_INVALID_OBSERVER_LANG",
        "invalid_observation_pattern" => "ERR_INVALID_OBSERVER_PATTERN",
        docs_governance::DOC_IDENTITY_PROTOCOL_ISSUE_TYPE | "doc_identity_protocol" => {
            "ERR_DOC_IDENTITY_PROTOCOL"
        }
        docs_governance::CANONICAL_DOC_HIDDEN_PATH_LINK_ISSUE_TYPE
        | "canonical_doc_hidden_path_link" => "WARN_CANONICAL_DOC_HIDDEN_PATH_LINK",
        docs_governance::MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE | "missing_package_docs_tree" => {
            "WARN_MISSING_PACKAGE_DOCS_TREE"
        }
        docs_governance::MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE | "missing_package_docs_index" => {
            "ERR_MISSING_PACKAGE_DOCS_INDEX"
        }
        docs_governance::MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE
        | "missing_package_docs_section_landing" => "WARN_MISSING_PACKAGE_DOCS_SECTION",
        docs_governance::MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE
        | "missing_package_docs_index_section_link" => "WARN_MISSING_PACKAGE_DOCS_INDEX_LINK",
        docs_governance::MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE
        | "missing_package_docs_index_relations_block" => {
            "WARN_MISSING_PACKAGE_DOCS_RELATIONS_BLOCK"
        }
        docs_governance::MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE
        | "missing_package_docs_index_footer_block" => "WARN_MISSING_PACKAGE_DOCS_FOOTER_BLOCK",
        docs_governance::INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE
        | "incomplete_package_docs_index_footer_block" => {
            "WARN_INCOMPLETE_PACKAGE_DOCS_FOOTER_BLOCK"
        }
        docs_governance::STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE
        | "stale_package_docs_index_footer_standards" => "WARN_STALE_PACKAGE_DOCS_FOOTER_STANDARDS",
        docs_governance::MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE
        | "missing_package_docs_index_relation_link" => "WARN_MISSING_PACKAGE_DOCS_RELATION_LINK",
        docs_governance::STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE
        | "stale_package_docs_index_relation_link" => "WARN_STALE_PACKAGE_DOCS_RELATION_LINK",
        _ => "UNKNOWN",
    }
}

/// Escape special XML characters.
pub(super) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
