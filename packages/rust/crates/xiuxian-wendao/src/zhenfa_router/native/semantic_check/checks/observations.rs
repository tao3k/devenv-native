use std::path::Path;

use crate::link_graph::PageIndexNode;
use crate::parsers::markdown::CodeObservation;
use crate::zhenfa_router::native::audit::{SourceFile, suggest_pattern_fix_with_threshold};
use crate::zhenfa_router::native::semantic_check::types::{
    FuzzySuggestionData, IssueLocation, SemanticIssue,
};
use xiuxian_code_intelligence::{
    CodeLanguageId, code_language_id_from_path, count_code_pattern_matches_for_language_id,
};

fn push_invalid_observation_language_issue(
    node: &PageIndexNode,
    doc_id: &str,
    obs: &CodeObservation,
    issues: &mut Vec<SemanticIssue>,
) {
    issues.push(SemanticIssue {
        severity: "error".to_string(),
        issue_type: "invalid_observation_language".to_string(),
        doc: doc_id.to_string(),
        node_id: node.node_id.clone(),
        message: format!(
            "Unsupported language '{}' in :OBSERVE: pattern",
            obs.language
        ),
        location: Some(IssueLocation::from_node(node)),
        suggestion: Some(
            "Use a supported language: rust, python, javascript, typescript, go, java, c, cpp, etc.".to_string()
        ),
        fuzzy_suggestion: None,
    });
}

fn build_observation_fuzzy_suggestion(
    obs: &CodeObservation,
    language_id: &CodeLanguageId,
    source_files: &[SourceFile],
    fuzzy_threshold: Option<f32>,
) -> Option<FuzzySuggestionData> {
    if source_files.is_empty() {
        return None;
    }

    suggest_pattern_fix_with_threshold(&obs.pattern, language_id, source_files, fuzzy_threshold)
        .map(|suggestion| FuzzySuggestionData::from_suggestion(suggestion, obs.pattern.clone()))
}

fn format_observation_source_location(source_location: Option<&str>) -> String {
    source_location.map_or_else(String::new, |location| {
        format!("Found similar code at: {location}")
    })
}

fn format_observation_suggestion(
    pattern: &str,
    description: &str,
    fuzzy_suggestion_data: Option<&FuzzySuggestionData>,
    fallback: &str,
) -> String {
    if let Some(data) = fuzzy_suggestion_data {
        format!(
            "Pattern '{pattern}' {description} {}\nConfidence: {:.0}%\n{}",
            data.suggested_pattern,
            data.confidence * 100.0,
            format_observation_source_location(data.source_location.as_deref())
        )
    } else {
        fallback.to_string()
    }
}

fn count_observation_matches(
    obs: &CodeObservation,
    language_id: &CodeLanguageId,
    source_files: &[SourceFile],
) -> usize {
    source_files
        .iter()
        .filter_map(|file| {
            let file_path = Path::new(&file.path);
            code_language_id_from_path(file_path)
                .filter(|file_language_id| *file_language_id == language_id.as_str())
                .and_then(|_| {
                    count_code_pattern_matches_for_language_id(
                        &file.content,
                        &obs.pattern,
                        language_id,
                    )
                    .ok()
                })
        })
        .sum()
}

/// Check :OBSERVE: code patterns for validity using xiuxian-ast (Blueprint v2.7).
pub(crate) fn check_code_observations(
    node: &PageIndexNode,
    doc_id: &str,
    source_files: &[SourceFile],
    fuzzy_threshold: Option<f32>,
    issues: &mut Vec<SemanticIssue>,
) {
    for obs in &node.metadata.observations {
        let Some(lang) = obs.ast_language() else {
            push_invalid_observation_language_issue(node, doc_id, obs, issues);
            continue;
        };
        let language_id = CodeLanguageId::from(lang.as_str());

        if let Err(error) = obs.validate_pattern() {
            let fuzzy_suggestion_data = build_observation_fuzzy_suggestion(
                obs,
                &language_id,
                source_files,
                fuzzy_threshold,
            );
            let suggestion_text = format_observation_suggestion(
                &obs.pattern,
                "is invalid. Consider updating to:",
                fuzzy_suggestion_data.as_ref(),
                "Fix the pattern syntax or check xiuxian-ast documentation for valid sgrep patterns",
            );

            issues.push(SemanticIssue {
                severity: "error".to_string(),
                issue_type: "invalid_observation_pattern".to_string(),
                doc: doc_id.to_string(),
                node_id: node.node_id.clone(),
                message: format!("Invalid sgrep pattern in :OBSERVE:: {error}"),
                location: Some(IssueLocation::from_node(node)),
                suggestion: Some(suggestion_text),
                fuzzy_suggestion: fuzzy_suggestion_data,
            });
            continue;
        }

        if source_files.is_empty() || count_observation_matches(obs, &language_id, source_files) > 0
        {
            continue;
        }

        let fuzzy_suggestion_data =
            build_observation_fuzzy_suggestion(obs, &language_id, source_files, fuzzy_threshold);
        let suggestion_text = format_observation_suggestion(
            &obs.pattern,
            "found no matches in the provided sources. Consider updating to:",
            fuzzy_suggestion_data.as_ref(),
            "Adjust the pattern or provide source files that contain the target code",
        );

        issues.push(SemanticIssue {
            severity: "warning".to_string(),
            issue_type: "observation_target_missing".to_string(),
            doc: doc_id.to_string(),
            node_id: node.node_id.clone(),
            message: format!(
                "Observation pattern '{}' found no matches in source files",
                obs.pattern
            ),
            location: Some(IssueLocation::from_node(node)),
            suggestion: Some(suggestion_text),
            fuzzy_suggestion: fuzzy_suggestion_data,
        });
    }
}
