//! Rust-owned candidate discovery for `WendaoGraph` `SearchStrategyFlow` probes.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use super::search_strategy_flow_evidence_edge_kinds;
use super::types::{
    MARKDOWN_HEADING_CANDIDATE_SOURCE, MAX_CANDIDATES, SearchStrategyFlowCandidateInput,
    SearchStrategyFlowCandidateInputBatch,
};
use serde_json::{Value, json};

const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "for", "from", "how", "in", "is", "it", "of", "on", "or", "the",
    "to", "with",
];

#[derive(Debug, Clone)]
pub(super) struct HeadingSection {
    level: usize,
    title: String,
    heading_anchor: String,
    line_start: usize,
    line_end: usize,
    text: String,
}

#[derive(Debug, Clone)]
struct ScoredCandidate {
    input: SearchStrategyFlowCandidateInput,
    ranking_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateRoute {
    SearchStrategy,
    Authority,
    PageIndex,
    LinkGraph,
    Validation,
}

impl CandidateRoute {
    const DIVERSE_FRONTIER_ORDER: [Self; 5] = [
        Self::SearchStrategy,
        Self::Authority,
        Self::PageIndex,
        Self::LinkGraph,
        Self::Validation,
    ];
}

pub(crate) fn search_strategy_flow_candidate_input_batch_from_markdown(
    intent: &str,
    search_root: &Path,
) -> Result<SearchStrategyFlowCandidateInputBatch, String> {
    let candidates = discover_search_strategy_flow_candidate_inputs_with_limit(
        intent,
        search_root,
        MAX_CANDIDATES,
    )?;
    Ok(search_strategy_flow_candidate_input_batch(
        MARKDOWN_HEADING_CANDIDATE_SOURCE,
        &candidates,
    ))
}

pub(crate) fn search_strategy_flow_candidate_input_batch(
    source: &'static str,
    candidates: &[SearchStrategyFlowCandidateInput],
) -> SearchStrategyFlowCandidateInputBatch {
    search_strategy_flow_candidate_input_batch_with_discovery_receipt(
        source,
        candidates,
        &default_candidate_discovery_receipt(source, candidates.len()),
    )
}

pub(crate) fn search_strategy_flow_candidate_input_batch_with_discovery_receipt(
    source: &'static str,
    candidates: &[SearchStrategyFlowCandidateInput],
    discovery_receipt: &Value,
) -> SearchStrategyFlowCandidateInputBatch {
    SearchStrategyFlowCandidateInputBatch {
        source,
        row_count: candidates.len(),
        tsv: serialize_candidate_inputs_tsv(candidates),
        discovery_receipt_json: discovery_receipt.to_string(),
    }
}

fn default_candidate_discovery_receipt(source: &'static str, row_count: usize) -> Value {
    json!({
        "receiptSource": source,
        "candidateInputSource": source,
        "candidateInputCount": row_count,
        "transport": "local-markdown-scan",
        "route": "local-markdown-heading-discovery",
        "attemptCount": 1,
        "mergedCandidateCount": row_count,
    })
}

#[cfg(test)]
pub(crate) fn discover_search_strategy_flow_candidate_inputs(
    intent: &str,
    search_root: &Path,
) -> Result<Vec<SearchStrategyFlowCandidateInput>, String> {
    discover_search_strategy_flow_candidate_inputs_with_limit(intent, search_root, MAX_CANDIDATES)
}

pub(crate) fn discover_search_strategy_flow_candidate_inputs_with_limit(
    intent: &str,
    search_root: &Path,
    max_candidates: usize,
) -> Result<Vec<SearchStrategyFlowCandidateInput>, String> {
    let intent_terms = intent_terms(intent);
    let mut scored = markdown_files(search_root)
        .into_iter()
        .map(|path| {
            scored_candidates_from_markdown_file(search_root, path.as_path(), &intent_terms)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    scored.sort_by(compare_scored_candidates);
    Ok(select_route_diverse_candidates(scored, max_candidates))
}

fn scored_candidates_from_markdown_file(
    search_root: &Path,
    path: &Path,
    intent_terms: &[String],
) -> Result<Vec<ScoredCandidate>, String> {
    let relative_path = repo_relative_path(search_root, path)?;
    let text = fs::read_to_string(path).map_err(|error| {
        let path = path.display();
        format!("read SearchStrategyFlow candidate file {path}: {error}")
    })?;
    Ok(heading_sections(&text)
        .into_iter()
        .map(|section| score_section_candidate(&relative_path, &section, intent_terms))
        .collect())
}

fn markdown_files(search_root: &Path) -> Vec<PathBuf> {
    let mut files = WalkDir::new(search_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_ignored_walk_entry(entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(DirEntry::into_path)
        .filter(|path| is_markdown_path(path))
        .collect::<Vec<_>>();
    files.sort();
    files
}

pub(super) fn is_ignored_walk_entry(entry: &DirEntry) -> bool {
    entry.depth() > 0
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with('.') || name == "node_modules" || name == "target")
}

pub(super) fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

pub(super) fn repo_relative_path(search_root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(search_root).map_err(|error| {
        let path = path.display();
        format!("derive SearchStrategyFlow relative path for {path}: {error}")
    })?;
    Ok(relative
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/"))
}

pub(super) fn heading_sections(text: &str) -> Vec<HeadingSection> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut heading_positions = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        if let Some((level, title)) = parse_heading_line(line) {
            heading_positions.push((index, level, title));
        }
    }

    let mut seen_anchors = HashSet::new();
    let mut sections = Vec::new();
    for (position_index, (line_index, level, title)) in heading_positions.iter().enumerate() {
        let anchor = markdown_anchor(title);
        if !seen_anchors.insert(anchor.clone()) {
            continue;
        }

        let line_end = heading_positions
            .iter()
            .skip(position_index + 1)
            .find(|(_, next_level, _)| next_level <= level)
            .map_or(lines.len(), |(next_index, _, _)| *next_index);
        let text = lines[*line_index..line_end].join("\n");
        sections.push(HeadingSection {
            level: *level,
            title: title.clone(),
            heading_anchor: anchor,
            line_start: line_index + 1,
            line_end,
            text,
        });
    }

    sections
}

fn parse_heading_line(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_end();
    let marker_len = trimmed.chars().take_while(|char| *char == '#').count();
    if !(1..=6).contains(&marker_len) {
        return None;
    }

    let after_marker = trimmed.get(marker_len..)?;
    if !after_marker.starts_with(' ') {
        return None;
    }

    let title = after_marker.trim();
    if title.is_empty() {
        return None;
    }
    Some((marker_len, title.to_owned()))
}

pub(super) fn markdown_anchor(title: &str) -> String {
    let mut anchor = String::new();
    let mut last_was_dash = false;

    for char in title.chars().filter(|char| *char != '`') {
        let lowercase = char.to_ascii_lowercase();
        if lowercase.is_ascii_alphanumeric() {
            anchor.push(lowercase);
            last_was_dash = false;
        } else if !last_was_dash {
            anchor.push('-');
            last_was_dash = true;
        }
    }

    let anchor = anchor.trim_matches('-');
    if anchor.is_empty() {
        "section".to_owned()
    } else {
        anchor.to_owned()
    }
}

fn score_section_candidate(
    relative_path: &str,
    section: &HeadingSection,
    intent_terms: &[String],
) -> ScoredCandidate {
    let haystack = format!(
        "{}\n{}\n{}",
        relative_path.to_ascii_lowercase(),
        section.title.to_ascii_lowercase(),
        section.text.to_ascii_lowercase()
    );
    let title_haystack = section.title.to_ascii_lowercase();
    let path_haystack = relative_path.to_ascii_lowercase();
    let matched_terms = intent_terms
        .iter()
        .filter(|term| haystack.contains(term.as_str()))
        .count();
    let title_matches = intent_terms
        .iter()
        .filter(|term| title_haystack.contains(term.as_str()))
        .count();
    let path_matches = intent_terms
        .iter()
        .filter(|term| path_haystack.contains(term.as_str()))
        .count();
    let coverage = if intent_terms.is_empty() {
        0.35
    } else {
        ratio(matched_terms, intent_terms.len())
    };
    let title_coverage = if intent_terms.is_empty() {
        0.0
    } else {
        ratio(title_matches, intent_terms.len())
    };
    let path_coverage = if intent_terms.is_empty() {
        0.0
    } else {
        ratio(path_matches, intent_terms.len())
    };
    let level_bonus = match section.level {
        1 => 0.08,
        2 => 0.06,
        3 => 0.04,
        _ => 0.02,
    };
    let search_bonus = if path_haystack.contains("search") || title_haystack.contains("search") {
        0.04
    } else {
        0.0
    };
    let page_index_bonus =
        if path_haystack.contains("page_index") || title_haystack.contains("reasoning") {
            0.03
        } else {
            0.0
        };

    let evidence_coverage = clamp_score(0.42 + (coverage * 0.46) + (title_coverage * 0.08));
    let graph_score = clamp_score(0.40 + (coverage * 0.34) + (path_coverage * 0.10) + search_bonus);
    let authority_score = clamp_score(0.62 + level_bonus + (path_coverage * 0.12));
    let structural_score = clamp_score(0.56 + level_bonus + page_index_bonus);
    let uncertainty = clamp_score(0.48 - (coverage * 0.30) - (title_coverage * 0.10));
    let ranking_score = evidence_coverage.mul_add(
        0.38,
        graph_score.mul_add(0.27, authority_score.mul_add(0.18, structural_score * 0.17)),
    ) - uncertainty * 0.12;

    ScoredCandidate {
        input: SearchStrategyFlowCandidateInput {
            relative_path: relative_path.to_owned(),
            heading_anchor: section.heading_anchor.clone(),
            title: section.title.clone(),
            line_start: section.line_start,
            line_end: section.line_end,
            context_cost: context_cost(&section.text),
            evidence_coverage,
            graph_score,
            authority_score,
            structural_score,
            uncertainty,
            blocked: false,
            edge_kinds: edge_kinds(relative_path, &section.title),
        },
        ranking_score,
    }
}

pub(super) fn clamp_score(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    usize_to_f64(numerator) / usize_to_f64(denominator)
}

fn usize_to_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

fn context_cost(text: &str) -> usize {
    let bytes = text.len();
    if bytes == 0 {
        512
    } else {
        bytes.div_ceil(20).max(1)
    }
}

pub(super) fn line_context_cost(line_start: usize, line_end: usize) -> usize {
    line_end
        .saturating_sub(line_start)
        .saturating_add(1)
        .saturating_mul(8)
        .max(1)
}

fn intent_terms(intent: &str) -> Vec<String> {
    let stop_words = STOP_WORDS.iter().copied().collect::<HashSet<_>>();
    let mut counts = HashMap::<String, usize>::new();
    for term in split_terms(intent) {
        if term.len() < 2 || stop_words.contains(term.as_str()) {
            continue;
        }
        *counts.entry(term).or_insert(0) += 1;
    }

    let mut terms = counts.into_keys().collect::<Vec<_>>();
    terms.sort();
    terms
}

fn split_terms(text: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();

    for char in text.chars() {
        if char.is_ascii_alphanumeric() {
            current.push(char.to_ascii_lowercase());
        } else if !current.is_empty() {
            terms.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        terms.push(current);
    }
    terms
}

pub(super) fn edge_kinds(relative_path: &str, title: &str) -> Vec<String> {
    let combined = format!(
        "{} {}",
        relative_path.to_ascii_lowercase(),
        title.to_ascii_lowercase()
    );
    let mut kinds = vec![
        "anchor".to_owned(),
        "page-index".to_owned(),
        "rust-discovered".to_owned(),
    ];
    kinds.extend(
        [
            ("search", "search-strategy"),
            ("reasoning", "reasoning-tree"),
            ("graph", "link-graph"),
            ("validation", "validation"),
        ]
        .into_iter()
        .filter(|(needle, _kind)| combined.contains(needle))
        .map(|(_needle, kind)| kind.to_owned()),
    );
    kinds.extend(search_strategy_flow_evidence_edge_kinds(relative_path));
    kinds.sort();
    kinds.dedup();
    kinds
}

fn compare_scored_candidates(left: &ScoredCandidate, right: &ScoredCandidate) -> Ordering {
    match right.ranking_score.partial_cmp(&left.ranking_score) {
        Some(ordering) if ordering != Ordering::Equal => ordering,
        _ => left
            .input
            .relative_path
            .cmp(&right.input.relative_path)
            .then_with(|| left.input.line_start.cmp(&right.input.line_start)),
    }
}

fn select_route_diverse_candidates(
    scored: Vec<ScoredCandidate>,
    max_candidates: usize,
) -> Vec<SearchStrategyFlowCandidateInput> {
    let mut selected = Vec::new();
    let mut selected_keys = HashSet::new();

    for route in CandidateRoute::DIVERSE_FRONTIER_ORDER {
        if selected.len() >= max_candidates {
            break;
        }
        if let Some(candidate) = scored
            .iter()
            .filter(|candidate| {
                !selected_keys.contains(&candidate_input_key(&candidate.input))
                    && candidate_matches_route(&candidate.input, route)
            })
            .max_by(|left, right| compare_route_seed_candidates(route, left, right))
            .cloned()
        {
            push_selected_candidate(candidate, &mut selected, &mut selected_keys);
        }
    }

    for candidate in scored {
        if selected.len() >= max_candidates {
            break;
        }
        push_selected_candidate(candidate, &mut selected, &mut selected_keys);
    }

    selected
        .into_iter()
        .map(|candidate| candidate.input)
        .collect()
}

fn push_selected_candidate(
    candidate: ScoredCandidate,
    selected: &mut Vec<ScoredCandidate>,
    selected_keys: &mut HashSet<String>,
) {
    if selected_keys.insert(candidate_input_key(&candidate.input)) {
        selected.push(candidate);
    }
}

fn candidate_input_key(candidate: &SearchStrategyFlowCandidateInput) -> String {
    format!("{}#{}", candidate.relative_path, candidate.heading_anchor)
}

fn candidate_matches_route(
    candidate: &SearchStrategyFlowCandidateInput,
    route: CandidateRoute,
) -> bool {
    let path = candidate.relative_path.to_ascii_lowercase();
    let title = candidate.title.to_ascii_lowercase();
    let anchor = candidate.heading_anchor.to_ascii_lowercase();
    let combined = format!("{path} {title} {anchor}");

    match route {
        CandidateRoute::SearchStrategy => {
            path.contains("30_search_strategy")
                || path.contains("search")
                || title.contains("search")
                || combined.contains("searchstrategyflow")
                || combined.contains("search_strategy")
                || combined.contains("search-strategy")
                || combined.contains("strategy flow")
        }
        CandidateRoute::Authority => {
            path.starts_with("docs/rfcs/")
                || path == "agents.md"
                || path.starts_with("docs/standards/")
                || combined.contains("ownership")
                || combined.contains("authority")
                || combined.contains("boundary")
                || combined.contains("source authority")
        }
        CandidateRoute::PageIndex => {
            path.contains("20_page_index")
                || combined.contains("pageindex")
                || combined.contains("page_index")
                || combined.contains("page-index")
                || combined.contains("page index")
                || combined.contains("reasoning tree")
                || combined.contains("reasoning-tree")
        }
        CandidateRoute::LinkGraph => {
            path.contains("10_graph_compute")
                || combined.contains("linkgraph")
                || combined.contains("link_graph")
                || combined.contains("link-graph")
                || combined.contains("link graph")
                || combined.contains("graph compute")
                || combined.contains("relation fanout")
        }
        CandidateRoute::Validation => {
            path.contains("90_validation")
                || path.starts_with("docs/testing/")
                || path.starts_with("docs/developer/")
                || combined.contains("validation")
                || combined.contains("test proof")
                || combined.contains("performance gate")
        }
    }
}

fn compare_route_seed_candidates(
    route: CandidateRoute,
    left: &ScoredCandidate,
    right: &ScoredCandidate,
) -> Ordering {
    match candidate_route_quality(&left.input, route)
        .partial_cmp(&candidate_route_quality(&right.input, route))
    {
        Some(ordering) if ordering != Ordering::Equal => ordering,
        _ => match left.ranking_score.partial_cmp(&right.ranking_score) {
            Some(ordering) if ordering != Ordering::Equal => ordering,
            _ => right
                .input
                .context_cost
                .cmp(&left.input.context_cost)
                .then_with(|| right.input.relative_path.cmp(&left.input.relative_path))
                .then_with(|| right.input.line_start.cmp(&left.input.line_start)),
        },
    }
}

fn candidate_route_quality(
    candidate: &SearchStrategyFlowCandidateInput,
    route: CandidateRoute,
) -> f64 {
    let candidate_text = format!(
        "{} {}",
        candidate.relative_path.to_ascii_lowercase(),
        candidate.heading_anchor.to_ascii_lowercase()
    );

    match route {
        CandidateRoute::SearchStrategy => {
            contains_any(
                &candidate_text,
                &[
                    "query-understanding",
                    "query_understanding",
                    "precision-pruning",
                    "precision_pruning",
                ],
            ) * 1.4
                + contains_any(
                    &candidate_text,
                    &[
                        "search-strategy-flow",
                        "search_strategy_flow",
                        "searchstrategyflow",
                    ],
                )
        }
        CandidateRoute::Authority => {
            contains_any(&candidate_text, &["docs/rfcs/"]) * 1.2
                + contains_any(
                    &candidate_text,
                    &[
                        "2026-03-26-wendao-query-engine-rfc",
                        "polyglot-compute-orchestrator",
                    ],
                ) * 1.2
                + contains_any(
                    &candidate_text,
                    &["ownership-boundary", "source-authority", "source authority"],
                ) * 1.0
                + contains_any(&candidate_text, &["current-ownership-matrix"]) * 0.6
                - contains_any(&candidate_text, &["sql-validation", "cognitive-policy"]) * 0.4
        }
        CandidateRoute::PageIndex => {
            contains_any(
                &candidate_text,
                &["reasoning-tree-contracts", "reasoning_tree_contracts"],
            ) * 1.2
                + contains_any(&candidate_text, &["reasoning-tree", "reasoning_tree"])
                + contains_any(
                    &candidate_text,
                    &["request-tables", "response-tables", "planner-actions"],
                ) * 0.5
                - contains_any(
                    &candidate_text,
                    &["not-the-owner", "not-owner", "traces-it-is-not"],
                ) * 1.2
        }
        CandidateRoute::LinkGraph => {
            contains_any(
                &candidate_text,
                &[
                    "pageindex-style-reasoning-tree",
                    "page-index-style-reasoning-tree",
                ],
            ) * 1.4
                + contains_any(
                    &candidate_text,
                    &[
                        "how-this-helps-linkgraph-search",
                        "relation",
                        "semantic-fanout",
                        "hnsw",
                        "ppr",
                        "community",
                        "graph-search",
                    ],
                )
        }
        CandidateRoute::Validation => {
            contains_any(
                &candidate_text,
                &[
                    "docs/testing/readme.md",
                    "docs/developer/testing.md",
                    "default-validation-path",
                    "local-validation",
                    "ci-test-proof",
                ],
            ) * 1.7
                + contains_any(
                    &candidate_text,
                    &["validation", "promotion", "gate", "proof"],
                )
        }
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> f64 {
    if needles.iter().any(|needle| haystack.contains(needle)) {
        1.0
    } else {
        0.0
    }
}

fn serialize_candidate_inputs_tsv(candidates: &[SearchStrategyFlowCandidateInput]) -> String {
    candidates
        .iter()
        .map(|candidate| {
            [
                escape_tsv_field(&candidate.relative_path),
                escape_tsv_field(&candidate.heading_anchor),
                escape_tsv_field(&candidate.title),
                candidate.line_start.to_string(),
                candidate.line_end.to_string(),
                candidate.context_cost.to_string(),
                candidate.evidence_coverage.to_string(),
                candidate.graph_score.to_string(),
                candidate.authority_score.to_string(),
                candidate.structural_score.to_string(),
                candidate.uncertainty.to_string(),
                candidate.blocked.to_string(),
                escape_tsv_field(&candidate.edge_kinds.join(",")),
            ]
            .join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_tsv_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
