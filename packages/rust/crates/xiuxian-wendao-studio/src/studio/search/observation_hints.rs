use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::studio::StudioState;
use crate::studio::analysis;
use crate::studio::types::{AnalysisEdgeKind, AnalysisNodeKind, MarkdownAnalysisResponse};
use xiuxian_wendao::parsers::markdown::code_observation::CodeObservation;

#[derive(Debug, Default)]
pub(in crate::studio::search) struct DefinitionObservationHints {
    pub(super) scope_patterns: Vec<String>,
    pub(super) languages: Vec<String>,
}

pub(in crate::studio::search) async fn definition_observation_hints(
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

    let (scope_patterns, languages) = parsed_observations(&observation_ids, &nodes_by_id)
        .fold(ObservationHintAccum::default(), |mut accum, observation| {
            accum.push_language(observation.language);
            if let Some(scope) = observation.scope {
                accum.push_scope(scope);
            }
            accum
        })
        .into_parts();

    (!scope_patterns.is_empty() || !languages.is_empty()).then_some(DefinitionObservationHints {
        scope_patterns,
        languages,
    })
}

#[derive(Default)]
struct ObservationHintAccum {
    scope_patterns: Vec<String>,
    languages: Vec<String>,
    seen_scopes: HashSet<String>,
    seen_languages: HashSet<String>,
}

impl ObservationHintAccum {
    fn push_language(&mut self, language: String) {
        if self.seen_languages.insert(language.clone()) {
            self.languages.push(language);
        }
    }

    fn push_scope(&mut self, scope: String) {
        if self.seen_scopes.insert(scope.clone()) {
            self.scope_patterns.push(scope);
        }
    }

    fn into_parts(self) -> (Vec<String>, Vec<String>) {
        (self.scope_patterns, self.languages)
    }
}

fn parsed_observations<'a>(
    observation_ids: &'a [String],
    nodes_by_id: &'a HashMap<&'a str, &'a crate::studio::types::AnalysisNode>,
) -> impl Iterator<Item = CodeObservation> + 'a {
    observation_ids.iter().filter_map(|observation_id| {
        let node = nodes_by_id.get(observation_id.as_str())?;
        let value = observation_value_from_label(node.label.as_str())?;
        CodeObservation::parse(value)
    })
}

fn matching_observation_ids(
    analysis: &MarkdownAnalysisResponse,
    nodes_by_id: &HashMap<&str, &crate::studio::types::AnalysisNode>,
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
    nodes_by_id: &HashMap<&str, &crate::studio::types::AnalysisNode>,
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
