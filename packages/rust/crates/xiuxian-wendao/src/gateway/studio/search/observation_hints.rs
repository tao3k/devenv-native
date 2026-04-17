use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::gateway::studio::StudioState;
use crate::gateway::studio::analysis;
use crate::gateway::studio::types::{AnalysisEdgeKind, AnalysisNodeKind, MarkdownAnalysisResponse};
use crate::parsers::markdown::code_observation::CodeObservation;

#[derive(Debug, Default)]
pub(in crate::gateway::studio::search) struct DefinitionObservationHints {
    pub(super) scope_patterns: Vec<String>,
    pub(super) languages: Vec<String>,
}

pub(in crate::gateway::studio::search) async fn definition_observation_hints(
    state: &StudioState,
    source_paths: Option<&[String]>,
    source_line: Option<usize>,
    query: &str,
) -> Option<DefinitionObservationHints> {
    let source_paths = source_paths?;
    let source_line = source_line?;

    for source_path in source_paths {
        if !is_markdown_path(source_path.as_str()) {
            continue;
        }

        let Ok(analysis) = analysis::analyze_markdown(state, source_path.as_str()).await else {
            continue;
        };

        if let Some(hints) = hints_from_analysis(&analysis, source_line, query) {
            return Some(hints);
        }
    }

    None
}

fn hints_from_analysis(
    analysis: &MarkdownAnalysisResponse,
    source_line: usize,
    query: &str,
) -> Option<DefinitionObservationHints> {
    let query_lc = query.to_ascii_lowercase();
    let nodes_by_id = analysis
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let observation_ids =
        matching_observation_ids(analysis, &nodes_by_id, source_line, query_lc.as_str());
    if observation_ids.is_empty() {
        return None;
    }

    let mut scope_patterns = Vec::new();
    let mut languages = Vec::new();
    let mut seen_scopes = HashSet::new();
    let mut seen_languages = HashSet::new();
    for observation_id in observation_ids {
        let Some(node) = nodes_by_id.get(observation_id.as_str()) else {
            continue;
        };
        let Some(value) = observation_value_from_label(node.label.as_str()) else {
            continue;
        };
        let Some(observation) = CodeObservation::parse(value) else {
            continue;
        };

        if seen_languages.insert(observation.language.clone()) {
            languages.push(observation.language);
        }
        if let Some(scope) = observation.scope
            && seen_scopes.insert(scope.clone())
        {
            scope_patterns.push(scope);
        }
    }

    (!scope_patterns.is_empty() || !languages.is_empty()).then_some(DefinitionObservationHints {
        scope_patterns,
        languages,
    })
}

fn matching_observation_ids(
    analysis: &MarkdownAnalysisResponse,
    nodes_by_id: &HashMap<&str, &crate::gateway::studio::types::AnalysisNode>,
    source_line: usize,
    query_lc: &str,
) -> Vec<String> {
    let line_matched = analysis
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, AnalysisNodeKind::Observation))
        .filter(|node| node.line_start <= source_line && source_line <= node.line_end)
        .filter(|node| {
            observation_references_query(analysis, nodes_by_id, node.id.as_str(), query_lc)
        })
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if !line_matched.is_empty() {
        return line_matched;
    }

    analysis
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, AnalysisNodeKind::Observation))
        .filter(|node| {
            observation_references_query(analysis, nodes_by_id, node.id.as_str(), query_lc)
        })
        .map(|node| node.id.clone())
        .collect()
}

fn observation_references_query(
    analysis: &MarkdownAnalysisResponse,
    nodes_by_id: &HashMap<&str, &crate::gateway::studio::types::AnalysisNode>,
    observation_id: &str,
    query_lc: &str,
) -> bool {
    analysis.edges.iter().any(|edge| {
        matches!(edge.kind, AnalysisEdgeKind::References)
            && edge.source_id == observation_id
            && (edge.label.to_ascii_lowercase() == query_lc
                || nodes_by_id
                    .get(edge.target_id.as_str())
                    .is_some_and(|node| node.label.to_ascii_lowercase() == query_lc))
    })
}

fn observation_value_from_label(label: &str) -> Option<&str> {
    if !label.starts_with(':') {
        return None;
    }
    let remainder = &label[1..];
    let key_end = remainder.find(':')?;
    let key = remainder[..key_end].trim();
    if key != "OBSERVE" && !key.starts_with("OBSERVE_") {
        return None;
    }
    let value = remainder[key_end + 1..].trim();
    if value.is_empty() { None } else { Some(value) }
}

fn is_markdown_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
}
