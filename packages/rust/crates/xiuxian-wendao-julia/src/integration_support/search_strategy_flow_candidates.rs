//! Rust-owned candidate discovery for `WendaoGraph` `SearchStrategyFlow` probes.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::{DirEntry, WalkDir};

const MAX_CANDIDATES: usize = 12;
pub(crate) const MARKDOWN_HEADING_CANDIDATE_SOURCE: &str = "rust-markdown-headings";
pub(crate) const FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE: &str = "rust-flight-repo-search";

const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "for", "from", "how", "in", "is", "it", "of", "on", "or", "the",
    "to", "with",
];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchStrategyFlowCandidateInput {
    pub(crate) relative_path: String,
    pub(crate) heading_anchor: String,
    pub(crate) title: String,
    pub(crate) line_start: usize,
    pub(crate) line_end: usize,
    pub(crate) context_cost: usize,
    pub(crate) evidence_coverage: f64,
    pub(crate) graph_score: f64,
    pub(crate) authority_score: f64,
    pub(crate) structural_score: f64,
    pub(crate) uncertainty: f64,
    pub(crate) blocked: bool,
    pub(crate) edge_kinds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowCandidateInputBatch {
    pub(crate) source: &'static str,
    pub(crate) row_count: usize,
    pub(crate) tsv: String,
}

pub(crate) struct SearchStrategyFlowRepoSearchHit<'a> {
    pub(crate) relative_path: &'a str,
    pub(crate) title: Option<&'a str>,
    pub(crate) best_section: Option<&'a str>,
    pub(crate) line_start: Option<usize>,
    pub(crate) line_end: Option<usize>,
    pub(crate) score: Option<f64>,
}

#[derive(Debug, Clone)]
struct HeadingSection {
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
    PageIndex,
    LinkGraph,
    Validation,
}

impl CandidateRoute {
    const DIVERSE_FRONTIER_ORDER: [Self; 4] = [
        Self::SearchStrategy,
        Self::PageIndex,
        Self::LinkGraph,
        Self::Validation,
    ];
}

pub(crate) fn search_strategy_flow_candidate_input_batch_from_markdown(
    intent: &str,
    search_root: &Path,
) -> Result<SearchStrategyFlowCandidateInputBatch, String> {
    let candidates = discover_search_strategy_flow_candidate_inputs(intent, search_root)?;
    Ok(search_strategy_flow_candidate_input_batch(
        MARKDOWN_HEADING_CANDIDATE_SOURCE,
        &candidates,
    ))
}

pub(crate) fn search_strategy_flow_candidate_input_batch(
    source: &'static str,
    candidates: &[SearchStrategyFlowCandidateInput],
) -> SearchStrategyFlowCandidateInputBatch {
    SearchStrategyFlowCandidateInputBatch {
        source,
        row_count: candidates.len(),
        tsv: serialize_candidate_inputs_tsv(candidates),
    }
}

pub(crate) fn search_strategy_flow_candidate_input_from_repo_search_hit(
    hit: &SearchStrategyFlowRepoSearchHit<'_>,
) -> SearchStrategyFlowCandidateInput {
    let title = non_blank(hit.best_section)
        .or_else(|| non_blank(hit.title))
        .unwrap_or(hit.relative_path);
    let line_start = hit.line_start.unwrap_or(1).max(1);
    let line_end = hit.line_end.unwrap_or(line_start).max(line_start);
    let score = hit.score.unwrap_or(0.5).clamp(0.0, 1.0);

    SearchStrategyFlowCandidateInput {
        relative_path: hit.relative_path.to_owned(),
        heading_anchor: markdown_anchor(title),
        title: title.to_owned(),
        line_start,
        line_end,
        context_cost: line_context_cost(line_start, line_end),
        evidence_coverage: clamp_score(0.58 + (score * 0.34)),
        graph_score: clamp_score(0.62 + (score * 0.25)),
        authority_score: clamp_score(0.70 + (score * 0.12)),
        structural_score: clamp_score(0.66 + (score * 0.12)),
        uncertainty: clamp_score(0.34 - (score * 0.20)),
        blocked: false,
        edge_kinds: repo_search_edge_kinds(hit.relative_path, title),
    }
}

pub(crate) fn discover_search_strategy_flow_candidate_inputs(
    intent: &str,
    search_root: &Path,
) -> Result<Vec<SearchStrategyFlowCandidateInput>, String> {
    let intent_terms = intent_terms(intent);
    let mut scored = Vec::new();

    for path in markdown_files(search_root) {
        let relative_path = repo_relative_path(search_root, &path)?;
        let text = fs::read_to_string(&path).map_err(|error| {
            let path = path.display();
            format!("read SearchStrategyFlow candidate file {path}: {error}")
        })?;
        for section in heading_sections(&text) {
            let candidate = score_section_candidate(&relative_path, &section, &intent_terms);
            scored.push(candidate);
        }
    }

    scored.sort_by(compare_scored_candidates);
    Ok(select_route_diverse_candidates(scored))
}

fn markdown_files(search_root: &Path) -> Vec<PathBuf> {
    let mut files = WalkDir::new(search_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_hidden_entry(entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(DirEntry::into_path)
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn is_hidden_entry(entry: &DirEntry) -> bool {
    entry.depth() > 0
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with('.'))
}

fn repo_relative_path(search_root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(search_root).map_err(|error| {
        let path = path.display();
        format!("derive SearchStrategyFlow relative path for {path}: {error}")
    })?;
    Ok(relative
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/"))
}

fn heading_sections(text: &str) -> Vec<HeadingSection> {
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

fn markdown_anchor(title: &str) -> String {
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

fn non_blank(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() { None } else { Some(value) }
    })
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

fn clamp_score(value: f64) -> f64 {
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

fn line_context_cost(line_start: usize, line_end: usize) -> usize {
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

fn edge_kinds(relative_path: &str, title: &str) -> Vec<String> {
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
    for (needle, kind) in [
        ("search", "search-strategy"),
        ("reasoning", "reasoning-tree"),
        ("graph", "link-graph"),
        ("validation", "validation"),
    ] {
        if combined.contains(needle) {
            kinds.push(kind.to_owned());
        }
    }
    kinds
}

fn repo_search_edge_kinds(relative_path: &str, title: &str) -> Vec<String> {
    let mut kinds = edge_kinds(relative_path, title);
    kinds.push("arrow-flight".to_owned());
    kinds.push("repo-search".to_owned());
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
) -> Vec<SearchStrategyFlowCandidateInput> {
    let mut selected = Vec::new();
    let mut selected_keys = HashSet::new();

    for route in CandidateRoute::DIVERSE_FRONTIER_ORDER {
        if selected.len() >= MAX_CANDIDATES {
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
        if selected.len() >= MAX_CANDIDATES {
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
                || combined.contains("validation")
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
            contains_any(&candidate_text, &["validation", "promotion", "gate"])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_heading_sections_from_real_markdown_shape()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let docs_dir = temp_dir.path().join("docs");
        fs::create_dir_all(&docs_dir)?;
        fs::write(
            docs_dir.join("search.md"),
            "# Search Strategy Flow\n\nIntro.\n\n## Query Understanding\n\nReasoning tree page index links.\n\n## Other\n\nOther text.\n",
        )?;
        fs::write(
            docs_dir.join("unrelated.md"),
            "# Unrelated\n\nDeployment notes only.\n",
        )?;

        let candidates = discover_search_strategy_flow_candidate_inputs(
            "query understanding reasoning tree",
            temp_dir.path(),
        )?;

        let Some(first) = candidates.first() else {
            panic!("expected first candidate");
        };
        assert_eq!(first.relative_path, "docs/search.md");
        assert_eq!(first.heading_anchor, "query-understanding");
        assert!(first.evidence_coverage > 0.8);
        assert!(first.context_cost > 0);
        assert!(first.edge_kinds.contains(&"rust-discovered".to_owned()));
        Ok(())
    }

    #[test]
    fn discovery_preserves_route_diverse_candidates_before_julia_pruning()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let search_dir = temp_dir.path().join("docs/30_search_strategy");
        let page_index_dir = temp_dir.path().join("docs/20_page_index");
        let graph_dir = temp_dir.path().join("docs/10_graph_compute");
        fs::create_dir_all(&search_dir)?;
        fs::create_dir_all(&page_index_dir)?;
        fs::create_dir_all(&graph_dir)?;

        for index in 0..16 {
            fs::write(
                search_dir.join(format!("search_{index:02}.md")),
                format!(
                    "# SearchStrategyFlow Query Understanding {index}\n\nSearchStrategyFlow intent strategy flow query understanding branch pruning.\n",
                ),
            )?;
        }
        fs::write(
            page_index_dir.join("reasoning_tree.md"),
            "# PageIndex Parent Child Evidence\n\nPageIndex reasoning tree parent child section spans and disclosure frontier.\n",
        )?;
        fs::write(
            graph_dir.join("link_graph.md"),
            "# LinkGraph Relation Fanout\n\nLinkGraph relation fanout connects section anchors and provenance edges.\n",
        )?;
        fs::write(
            temp_dir.path().join("docs/index.md"),
            "# Documentation Index\n\nSearchStrategyFlow PageIndex LinkGraph relation path index.\n",
        )?;

        let candidates = discover_search_strategy_flow_candidate_inputs(
            "SearchStrategyFlow PageIndex LinkGraph relation path",
            temp_dir.path(),
        )?;

        assert_eq!(candidates.len(), MAX_CANDIDATES);
        assert!(candidates.iter().any(|candidate| {
            candidate
                .relative_path
                .starts_with("docs/30_search_strategy/")
        }));
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.relative_path.starts_with("docs/20_page_index/"))
        );
        assert!(candidates.iter().any(|candidate| {
            candidate
                .relative_path
                .starts_with("docs/10_graph_compute/")
        }));
        Ok(())
    }

    #[test]
    fn serializes_tsv_without_losing_candidate_boundaries() -> Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = tempfile::tempdir()?;
        fs::write(
            temp_dir.path().join("doc.md"),
            "# Query\tUnderstanding\n\nLine one.\nLine two.\n",
        )?;

        let batch = search_strategy_flow_candidate_input_batch_from_markdown(
            "query understanding",
            temp_dir.path(),
        )?;

        assert_eq!(batch.source, MARKDOWN_HEADING_CANDIDATE_SOURCE);
        assert_eq!(batch.row_count, 1);
        assert!(batch.tsv.contains("doc.md"));
        assert!(batch.tsv.contains("Query\\tUnderstanding"));
        assert_eq!(batch.tsv.lines().count(), 1);
        Ok(())
    }

    #[test]
    fn builds_repo_search_candidate_with_flight_source_edges() {
        let hit = SearchStrategyFlowRepoSearchHit {
            relative_path: "docs/search.md",
            title: Some("Search Strategy"),
            best_section: Some("Query Understanding"),
            line_start: Some(10),
            line_end: Some(14),
            score: Some(0.9),
        };
        let candidate = search_strategy_flow_candidate_input_from_repo_search_hit(&hit);

        assert_eq!(candidate.relative_path, "docs/search.md");
        assert_eq!(candidate.heading_anchor, "query-understanding");
        assert_eq!(candidate.line_start, 10);
        assert_eq!(candidate.line_end, 14);
        assert_eq!(candidate.context_cost, 40);
        assert!(candidate.evidence_coverage > 0.8);
        assert!(candidate.edge_kinds.contains(&"arrow-flight".to_owned()));
        assert!(candidate.edge_kinds.contains(&"repo-search".to_owned()));
    }
}
