//! Agent-oriented temporary task-memory ranking.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xiuxian_memory_engine::{
    Episode, EpisodeDraft, IntentEncoder, QTable, TwoPhaseSearch, TwoPhaseSearchRequest,
};

use super::super::model::AgentOrgTaskListRow;
use super::super::row_view::property_value;
use super::super::section_lens::TaskSectionLens;

const TEMPORARY_MEMORY_SCOPE: &str = "wendao-client:agent-org-temporary-memory";
const EMBEDDING_DIMENSION: usize = 128;
const MEMORY_Q_WEIGHT: f32 = 0.35;
const SDD_EVIDENCE_MAX_LINES: usize = 96;
const SDD_EVIDENCE_MAX_HEADINGS: usize = 16;
const SDD_EVIDENCE_MAX_PROPERTIES: usize = 24;

#[derive(Debug, Clone, Copy, Default)]
pub(in crate::orgize::read_model) struct ProbeRecallScope {
    include_done: bool,
    include_archived: bool,
}

impl ProbeRecallScope {
    pub(in crate::orgize::read_model) const fn new(
        include_done: bool,
        include_archived: bool,
    ) -> Self {
        Self {
            include_done,
            include_archived,
        }
    }
}

pub(in crate::orgize::read_model) fn rank_probe_rows<'a>(
    rows: Vec<&'a AgentOrgTaskListRow>,
    query: &str,
    limit: usize,
    scope: ProbeRecallScope,
) -> Vec<&'a AgentOrgTaskListRow> {
    if limit == 0 {
        return Vec::new();
    }
    if rows.is_empty() {
        return rows;
    }

    let query = QueryEvidence::from_query(query);
    let candidates = rows
        .into_iter()
        .map(RecallCandidate::from_row)
        .collect::<Vec<_>>();
    let candidates = prune_completed_noise(candidates, &query, scope);
    let identity_matches = candidates
        .iter()
        .filter(|candidate| candidate_identity_matches(candidate, &query))
        .take(limit)
        .map(|candidate| candidate.row)
        .collect::<Vec<_>>();
    if !identity_matches.is_empty() {
        return identity_matches;
    }
    let temporal = TemporalRecallContext::from_candidates(&candidates);
    let evidence_windows = candidates
        .iter()
        .map(CandidateEvidence::from_candidate)
        .collect::<Vec<_>>();
    let corpus = EvidenceCorpus::from_windows(&query, &evidence_windows);
    let base_scores = candidates
        .iter()
        .zip(evidence_windows.iter())
        .map(|(candidate, evidence)| {
            candidate_org_facet_score(candidate, evidence, &query, &corpus, &temporal)
        })
        .collect::<Vec<_>>();
    let memory_scores =
        temporary_memory_scores(&candidates, &evidence_windows, &query, &base_scores);

    let mut ranked = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| RankedCandidate {
            index,
            score: base_scores[index].with_memory_score(
                memory_scores
                    .get(candidate.row.orgid.as_str())
                    .copied()
                    .unwrap_or_default(),
            ),
            candidate,
        })
        .collect::<Vec<_>>();
    if !query.is_empty() {
        ranked.retain(|ranked| ranked.score.has_query_evidence());
    }
    ranked.sort_by(compare_ranked_candidates);

    ranked
        .into_iter()
        .take(limit)
        .map(|ranked| ranked.candidate.row)
        .collect()
}

struct RecallCandidate<'a> {
    row: &'a AgentOrgTaskListRow,
    lens: Option<TaskSectionLens>,
}

impl<'a> RecallCandidate<'a> {
    fn from_row(row: &'a AgentOrgTaskListRow) -> Self {
        Self {
            row,
            lens: task_row_section_lens(row),
        }
    }
}

#[derive(Debug, Clone)]
struct QueryEvidence {
    raw: String,
    normalized: String,
    tokens: Vec<String>,
}

impl QueryEvidence {
    fn from_query(query: &str) -> Self {
        Self {
            raw: query.trim().to_string(),
            normalized: normalized_text(query),
            tokens: normalized_words(query),
        }
    }

    fn is_empty(&self) -> bool {
        self.raw.trim().is_empty()
    }
}

struct CandidateEvidence {
    text: String,
    normalized: String,
    tokens: HashSet<String>,
    facets: Vec<OrgEvidenceFacet>,
}

impl CandidateEvidence {
    fn from_candidate(candidate: &RecallCandidate<'_>) -> Self {
        let mut facets = Vec::new();
        push_candidate_base_facets(&mut facets, candidate);
        push_candidate_property_facets(&mut facets, candidate);
        push_candidate_sdd_facets(&mut facets, candidate);
        push_candidate_lens_facets(&mut facets, candidate);

        let text = facets
            .iter()
            .map(|facet| facet.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let normalized = normalized_text(text.as_str());
        let tokens = normalized
            .split_whitespace()
            .map(str::to_string)
            .collect::<HashSet<_>>();
        Self {
            text,
            normalized,
            tokens,
            facets,
        }
    }
}

fn push_candidate_base_facets(facets: &mut Vec<OrgEvidenceFacet>, candidate: &RecallCandidate<'_>) {
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Identity,
        "orgid",
        Some(candidate.row.orgid.as_str()),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Heading,
        "title",
        Some(candidate.row.title.as_str()),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Lifecycle,
        "todo",
        candidate.row.todo_state.as_deref(),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Source,
        "file",
        Some(task_row_file_key(candidate.row).as_str()),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Source,
        "source",
        Some(candidate.row.source_path.as_str()),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Heading,
        "outline",
        Some(candidate.row.outline_path.join(" / ").as_str()),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Tags,
        "tags",
        Some(
            candidate
                .row
                .tags
                .iter()
                .chain(candidate.row.effective_tags.iter())
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
                .as_str(),
        ),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Planning,
        "scheduled",
        candidate.row.scheduled.as_deref(),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Planning,
        "deadline",
        candidate.row.deadline.as_deref(),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Planning,
        "closed",
        candidate.row.closed.as_deref(),
    );
}

fn push_candidate_property_facets(
    facets: &mut Vec<OrgEvidenceFacet>,
    candidate: &RecallCandidate<'_>,
) {
    for property in &candidate.row.properties {
        if property.key == "STATUS" || property.key == "EXECPLAN" {
            continue;
        }
        push_evidence_facet(
            facets,
            property_facet_kind(property.key.as_str()),
            property.key.as_str(),
            Some(property.value.as_str()),
        );
    }
}

fn push_candidate_lens_facets(facets: &mut Vec<OrgEvidenceFacet>, candidate: &RecallCandidate<'_>) {
    let Some(lens) = candidate.lens.as_ref() else {
        return;
    };
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Progress,
        "progress",
        lens.progress_label().as_deref(),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::Checklist,
        "checklist",
        Some(lens.checklist_text().as_str()),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::ChildHeadings,
        "children",
        Some(lens.child_heading_text().as_str()),
    );
    push_evidence_facet(
        facets,
        OrgEvidenceFacetKind::NextAction,
        "next-unchecked",
        lens.next_unchecked.as_deref(),
    );
}

fn push_candidate_sdd_facets(facets: &mut Vec<OrgEvidenceFacet>, candidate: &RecallCandidate<'_>) {
    let Some(path) = task_sdd_evidence_path(candidate.row) else {
        return;
    };
    let Ok(source) = std::fs::read_to_string(path.as_path()) else {
        return;
    };

    let mut headings = 0usize;
    let mut properties = 0usize;
    for line in source.lines().take(SDD_EVIDENCE_MAX_LINES) {
        let trimmed = line.trim();
        if let Some(title) = org_keyword_value_from_line(trimmed, "TITLE") {
            push_evidence_facet(
                facets,
                OrgEvidenceFacetKind::Graph,
                "sdd-title",
                Some(title),
            );
            continue;
        }
        if headings < SDD_EVIDENCE_MAX_HEADINGS
            && let Some(heading) = org_heading_title_for_evidence(trimmed)
        {
            push_evidence_facet(
                facets,
                OrgEvidenceFacetKind::Graph,
                "sdd-heading",
                Some(heading.as_str()),
            );
            headings += 1;
            continue;
        }
        if properties < SDD_EVIDENCE_MAX_PROPERTIES
            && let Some((key, value)) = org_property_from_line(trimmed)
            && sdd_property_is_evidence(key)
        {
            let label = format!("sdd-{}", key.to_ascii_lowercase().replace('_', "-"));
            push_evidence_facet(
                facets,
                OrgEvidenceFacetKind::Graph,
                label.as_str(),
                Some(value),
            );
            properties += 1;
        }
    }
}

struct OrgEvidenceFacet {
    kind: OrgEvidenceFacetKind,
    text: String,
    tokens: HashSet<String>,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum OrgEvidenceFacetKind {
    Identity,
    Heading,
    Lifecycle,
    Source,
    Tags,
    Planning,
    Property,
    NextAction,
    Graph,
    Progress,
    Checklist,
    ChildHeadings,
}

impl OrgEvidenceFacetKind {
    const COUNT: usize = 12;

    const fn label(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Heading => "heading",
            Self::Lifecycle => "lifecycle",
            Self::Source => "source",
            Self::Tags => "tags",
            Self::Planning => "planning",
            Self::Property => "property",
            Self::NextAction => "next-action",
            Self::Graph => "graph",
            Self::Progress => "progress",
            Self::Checklist => "checklist",
            Self::ChildHeadings => "child-headings",
        }
    }
}

struct EvidenceCorpus {
    document_frequency: HashMap<String, usize>,
    document_count: usize,
}

impl EvidenceCorpus {
    fn from_windows(query: &QueryEvidence, windows: &[CandidateEvidence]) -> Self {
        let mut document_frequency = HashMap::<String, usize>::new();
        for token in &query.tokens {
            let count = windows
                .iter()
                .filter(|window| token_set_has_match(&window.tokens, token))
                .count();
            document_frequency.insert(token.clone(), count);
        }
        Self {
            document_frequency,
            document_count: windows.len().max(1),
        }
    }

    fn token_information(&self, token: &str) -> f32 {
        let frequency = self
            .document_frequency
            .get(token)
            .copied()
            .unwrap_or_default();
        if frequency == 0 {
            return 1.0;
        }
        let commonality = ratio(frequency, self.document_count);
        (1.0 - commonality).max(0.05)
    }
}

#[derive(Debug, Clone, Copy)]
struct RecallScore {
    identity: bool,
    phrase: bool,
    token_coverage: f32,
    facet_matches: usize,
    recovery_anchor_coverage: f32,
    facet_coverage: f32,
    facet_signal: f32,
    lifecycle: f32,
    recency: f32,
    memory_score: f32,
}

impl RecallScore {
    fn with_memory_score(mut self, memory_score: f32) -> Self {
        self.memory_score = memory_score;
        self
    }

    fn utility_value(self) -> f32 {
        let identity: f32 = if self.identity { 1.0 } else { 0.0 };
        let phrase: f32 = if self.phrase { 0.85 } else { 0.0 };
        (identity
            .max(phrase)
            .max(self.token_coverage)
            .max(self.recovery_anchor_coverage)
            .max(self.facet_coverage)
            .clamp(0.0, 1.0)
            + (ratio(self.facet_matches, OrgEvidenceFacetKind::COUNT) * 0.08)
            + self.facet_signal
            + self.lifecycle
            + self.recency)
            .clamp(0.0, 1.0)
    }

    fn rank_value(self) -> f32 {
        self.utility_value().max(self.memory_score).clamp(0.0, 1.0)
    }

    fn has_query_evidence(self) -> bool {
        self.identity
            || self.phrase
            || self.token_coverage > 0.0
            || self.facet_matches > 0
            || self.recovery_anchor_coverage > 0.0
            || self.facet_coverage > 0.0
    }
}

struct RankedCandidate<'a, 'b> {
    index: usize,
    score: RecallScore,
    candidate: &'b RecallCandidate<'a>,
}

fn compare_ranked_candidates(
    left: &RankedCandidate<'_, '_>,
    right: &RankedCandidate<'_, '_>,
) -> std::cmp::Ordering {
    right
        .score
        .identity
        .cmp(&left.score.identity)
        .then_with(|| right.score.rank_value().total_cmp(&left.score.rank_value()))
        .then_with(|| {
            right
                .score
                .token_coverage
                .total_cmp(&left.score.token_coverage)
        })
        .then_with(|| {
            right
                .score
                .recovery_anchor_coverage
                .total_cmp(&left.score.recovery_anchor_coverage)
        })
        .then_with(|| right.score.facet_signal.total_cmp(&left.score.facet_signal))
        .then_with(|| {
            right
                .score
                .facet_coverage
                .total_cmp(&left.score.facet_coverage)
        })
        .then_with(|| right.score.facet_matches.cmp(&left.score.facet_matches))
        .then_with(|| right.score.phrase.cmp(&left.score.phrase))
        .then_with(|| right.score.recency.total_cmp(&left.score.recency))
        .then_with(|| left.index.cmp(&right.index))
}

fn candidate_org_facet_score(
    candidate: &RecallCandidate<'_>,
    evidence: &CandidateEvidence,
    query: &QueryEvidence,
    corpus: &EvidenceCorpus,
    temporal: &TemporalRecallContext,
) -> RecallScore {
    let normalized_orgid = normalized_text(candidate.row.orgid.as_str());
    let identity = !query.is_empty()
        && (query.raw.eq_ignore_ascii_case(candidate.row.orgid.as_str())
            || (!query.normalized.is_empty() && query.normalized == normalized_orgid));
    let phrase =
        !query.normalized.is_empty() && evidence.normalized.contains(query.normalized.as_str());
    let token_coverage = token_information_coverage(query, evidence, corpus);
    let matched_facets = matched_org_facet_kinds(query, evidence);
    let facet_matches = matched_facets.len();
    RecallScore {
        identity,
        phrase,
        token_coverage,
        facet_matches,
        recovery_anchor_coverage: org_recovery_anchor_coverage_score(query, evidence, corpus),
        facet_coverage: org_facet_coverage_score(query, evidence, corpus),
        facet_signal: org_facet_signal(&matched_facets),
        lifecycle: lifecycle_signal(candidate),
        recency: temporal.modified_recency_bonus(candidate.row),
        memory_score: 0.0,
    }
}

fn temporary_memory_scores(
    candidates: &[RecallCandidate<'_>],
    evidence_windows: &[CandidateEvidence],
    query: &QueryEvidence,
    base_scores: &[RecallScore],
) -> HashMap<String, f32> {
    if candidates.is_empty() {
        return HashMap::new();
    }

    let encoder = Arc::new(IntentEncoder::new(EMBEDDING_DIMENSION));
    let q_table = Arc::new(QTable::with_params(1.0, 0.95));
    let episodes = candidates
        .iter()
        .zip(evidence_windows.iter())
        .zip(base_scores.iter())
        .map(|((candidate, evidence), score)| {
            let reward = score.utility_value();
            q_table.update(candidate.row.orgid.as_str(), reward);
            let intent = temporary_memory_episode_intent(candidate, evidence);
            Episode::new(EpisodeDraft {
                id: candidate.row.orgid.as_str().into(),
                intent_embedding: encoder.encode(intent.as_str()),
                intent,
                experience: temporary_memory_episode_experience(candidate, evidence, reward),
                outcome: "org-facet-candidate".to_string(),
                scope: Some(TEMPORARY_MEMORY_SCOPE.to_string()),
            })
        })
        .collect::<Vec<_>>();

    let search = TwoPhaseSearch::with_defaults(q_table, encoder);
    search
        .search(TwoPhaseSearchRequest {
            episodes: &episodes,
            intent: temporary_memory_query_intent(query).as_str(),
            k1: Some(episodes.len()),
            k2: Some(episodes.len()),
            lambda: Some(MEMORY_Q_WEIGHT),
        })
        .into_iter()
        .map(|(episode, score)| (episode.id, score))
        .collect()
}

fn temporary_memory_query_intent(query: &QueryEvidence) -> String {
    query.raw.clone()
}

fn temporary_memory_episode_intent(
    candidate: &RecallCandidate<'_>,
    evidence: &CandidateEvidence,
) -> String {
    format!("orgid: {}\n{}", candidate.row.orgid, evidence.text)
}

fn temporary_memory_episode_experience(
    candidate: &RecallCandidate<'_>,
    evidence: &CandidateEvidence,
    reward: f32,
) -> String {
    let facet_labels = evidence
        .facets
        .iter()
        .map(|facet| facet.kind.label())
        .collect::<HashSet<_>>();
    let mut facet_labels = facet_labels.into_iter().collect::<Vec<_>>();
    facet_labels.sort_unstable();
    let mut experience = format!(
        "source: {}:{}\nfacets: {}\norg-utility: {:.3}",
        candidate.row.source_path,
        candidate.row.source_line,
        facet_labels.join(","),
        reward,
    );
    if let Some(lens) = candidate.lens.as_ref()
        && let Some(next_unchecked) = lens.next_unchecked.as_deref()
    {
        experience.push('\n');
        experience.push_str("next-unchecked: ");
        experience.push_str(next_unchecked);
    }
    experience
}

fn matched_org_facet_kinds(
    query: &QueryEvidence,
    evidence: &CandidateEvidence,
) -> HashSet<OrgEvidenceFacetKind> {
    if query.tokens.is_empty() {
        return HashSet::new();
    }

    evidence
        .facets
        .iter()
        .filter(|facet| {
            query
                .tokens
                .iter()
                .any(|query_token| token_set_has_match(&facet.tokens, query_token))
        })
        .map(|facet| facet.kind)
        .collect::<HashSet<_>>()
}

fn org_facet_signal(facets: &HashSet<OrgEvidenceFacetKind>) -> f32 {
    facets
        .iter()
        .map(|facet| match facet {
            OrgEvidenceFacetKind::Identity => 0.18,
            OrgEvidenceFacetKind::Heading | OrgEvidenceFacetKind::NextAction => 0.14,
            OrgEvidenceFacetKind::Checklist => 0.13,
            OrgEvidenceFacetKind::Graph => 0.12,
            OrgEvidenceFacetKind::Source | OrgEvidenceFacetKind::Property => 0.10,
            OrgEvidenceFacetKind::Tags => 0.08,
            OrgEvidenceFacetKind::ChildHeadings => 0.07,
            OrgEvidenceFacetKind::Planning => 0.06,
            OrgEvidenceFacetKind::Progress => 0.04,
            OrgEvidenceFacetKind::Lifecycle => 0.03,
        })
        .sum::<f32>()
        .min(0.22)
}

fn org_facet_coverage_score(
    query: &QueryEvidence,
    evidence: &CandidateEvidence,
    corpus: &EvidenceCorpus,
) -> f32 {
    evidence
        .facets
        .iter()
        .map(|facet| {
            token_information_coverage_for_tokens(query, &facet.tokens, corpus)
                * facet_coverage_weight(facet.kind)
        })
        .fold(0.0, f32::max)
        .clamp(0.0, 1.0)
}

fn org_recovery_anchor_coverage_score(
    query: &QueryEvidence,
    evidence: &CandidateEvidence,
    corpus: &EvidenceCorpus,
) -> f32 {
    evidence
        .facets
        .iter()
        .filter(|facet| facet_is_recovery_anchor(facet.kind))
        .map(|facet| {
            token_information_coverage_for_tokens(query, &facet.tokens, corpus)
                * facet_coverage_weight(facet.kind)
        })
        .fold(0.0, f32::max)
        .clamp(0.0, 1.0)
}

const fn facet_is_recovery_anchor(kind: OrgEvidenceFacetKind) -> bool {
    matches!(
        kind,
        OrgEvidenceFacetKind::Identity
            | OrgEvidenceFacetKind::Source
            | OrgEvidenceFacetKind::NextAction
            | OrgEvidenceFacetKind::Graph
            | OrgEvidenceFacetKind::Checklist
    )
}

fn facet_coverage_weight(kind: OrgEvidenceFacetKind) -> f32 {
    match kind {
        OrgEvidenceFacetKind::Identity => 1.0,
        OrgEvidenceFacetKind::Heading => 0.98,
        OrgEvidenceFacetKind::NextAction => 0.97,
        OrgEvidenceFacetKind::Checklist => 0.95,
        OrgEvidenceFacetKind::Graph => 0.90,
        OrgEvidenceFacetKind::Source => 0.88,
        OrgEvidenceFacetKind::Property => 0.86,
        OrgEvidenceFacetKind::ChildHeadings => 0.80,
        OrgEvidenceFacetKind::Tags => 0.72,
        OrgEvidenceFacetKind::Planning => 0.70,
        OrgEvidenceFacetKind::Progress => 0.45,
        OrgEvidenceFacetKind::Lifecycle => 0.35,
    }
}

fn token_information_coverage(
    query: &QueryEvidence,
    evidence: &CandidateEvidence,
    corpus: &EvidenceCorpus,
) -> f32 {
    token_information_coverage_for_tokens(query, &evidence.tokens, corpus)
}

fn token_information_coverage_for_tokens(
    query: &QueryEvidence,
    tokens: &HashSet<String>,
    corpus: &EvidenceCorpus,
) -> f32 {
    if query.tokens.is_empty() {
        return 0.0;
    }

    let total_information = query
        .tokens
        .iter()
        .map(|token| corpus.token_information(token))
        .sum::<f32>();
    if total_information <= f32::EPSILON {
        return 0.0;
    }

    let matched_information = query
        .tokens
        .iter()
        .filter(|token| token_set_has_match(tokens, token))
        .map(|token| corpus.token_information(token))
        .sum::<f32>();
    (matched_information / total_information).clamp(0.0, 1.0)
}

fn prune_completed_noise<'a>(
    candidates: Vec<RecallCandidate<'a>>,
    query: &QueryEvidence,
    scope: ProbeRecallScope,
) -> Vec<RecallCandidate<'a>> {
    if candidates.len() <= 1 {
        return candidates;
    }
    if scope.include_done || scope.include_archived {
        return candidates;
    }
    let active_indexes = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            !task_candidate_is_completed_noise(candidate)
                || candidate_identity_matches(candidate, query)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if active_indexes.is_empty() {
        return candidates;
    }
    candidates
        .into_iter()
        .enumerate()
        .filter_map(|(index, candidate)| active_indexes.contains(&index).then_some(candidate))
        .collect()
}

fn candidate_identity_matches(candidate: &RecallCandidate<'_>, query: &QueryEvidence) -> bool {
    let normalized_orgid = normalized_text(candidate.row.orgid.as_str());
    !query.is_empty()
        && (query.raw.eq_ignore_ascii_case(candidate.row.orgid.as_str())
            || (!query.normalized.is_empty() && query.normalized == normalized_orgid))
}

fn task_candidate_is_completed_noise(candidate: &RecallCandidate<'_>) -> bool {
    if candidate.row.is_done || candidate.row.closed.is_some() {
        return true;
    }
    title_has_complete_progress_cookie(candidate.row.title.as_str())
        || candidate
            .lens
            .as_ref()
            .is_some_and(TaskSectionLens::is_complete)
}

fn title_has_complete_progress_cookie(title: &str) -> bool {
    title.contains("[100%]")
        || title
            .split_whitespace()
            .any(token_has_complete_ratio_cookie)
}

fn token_has_complete_ratio_cookie(token: &str) -> bool {
    let Some(cookie) = token
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let Some((done, total)) = cookie.split_once('/') else {
        return false;
    };
    let Ok(done) = done.parse::<u64>() else {
        return false;
    };
    let Ok(total) = total.parse::<u64>() else {
        return false;
    };
    done > 0 && done == total
}

fn lifecycle_signal(candidate: &RecallCandidate<'_>) -> f32 {
    let mut signal: f32 = 0.0;
    if !candidate.row.is_done && !candidate.row.archived {
        signal += 0.08;
    }
    if candidate.row.todo_state.as_deref() == Some("DOING") {
        signal += 0.06;
    }
    if let Some(lens) = candidate.lens.as_ref()
        && lens.checked > 0
        && lens.unchecked > 0
    {
        signal += 0.04;
    }
    signal
}

fn normalized_text(value: &str) -> String {
    normalized_words(value).join(" ")
}

fn normalized_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut ascii_run = String::new();
    let mut cjk_run = Vec::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            flush_cjk_run(&mut words, &mut cjk_run);
            ascii_run.push(character);
        } else if is_cjk_character(character) {
            flush_ascii_run(&mut words, &mut ascii_run);
            cjk_run.push(character);
        } else {
            flush_ascii_run(&mut words, &mut ascii_run);
            flush_cjk_run(&mut words, &mut cjk_run);
        }
    }
    flush_ascii_run(&mut words, &mut ascii_run);
    flush_cjk_run(&mut words, &mut cjk_run);
    words.into_iter().filter(|word| word.len() > 1).collect()
}

fn flush_ascii_run(words: &mut Vec<String>, ascii_run: &mut String) {
    if !ascii_run.is_empty() {
        let run = std::mem::take(ascii_run);
        push_unique_word(words, run.to_ascii_lowercase());
        for segment in ascii_semantic_segments(run.as_str()) {
            push_unique_word(words, segment);
        }
    }
}

fn ascii_semantic_segments(value: &str) -> Vec<String> {
    let characters = value.chars().collect::<Vec<_>>();
    let mut segments = Vec::new();
    let mut current = String::new();

    for (index, character) in characters.iter().copied().enumerate() {
        if !current.is_empty() && ascii_segment_boundary(&characters, index) {
            push_unique_word(&mut segments, current.to_ascii_lowercase());
            current.clear();
        }
        current.push(character);
    }
    if !current.is_empty() {
        push_unique_word(&mut segments, current.to_ascii_lowercase());
    }
    segments
}

fn ascii_segment_boundary(characters: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let previous = characters[index - 1];
    let current = characters[index];
    let next = characters.get(index + 1).copied();
    (previous.is_ascii_lowercase() && current.is_ascii_uppercase())
        || (previous.is_ascii_alphabetic() && current.is_ascii_digit())
        || (previous.is_ascii_digit() && current.is_ascii_alphabetic())
        || (previous.is_ascii_uppercase()
            && current.is_ascii_uppercase()
            && next.is_some_and(|next| next.is_ascii_lowercase()))
}

fn flush_cjk_run(words: &mut Vec<String>, cjk_run: &mut Vec<char>) {
    if cjk_run.is_empty() {
        return;
    }
    for character in cjk_run.iter() {
        push_unique_word(words, character.to_string());
    }
    for pair in cjk_run.windows(2) {
        push_unique_word(words, pair.iter().collect::<String>());
    }
    if cjk_run.len() > 2 {
        push_unique_word(words, cjk_run.iter().collect::<String>());
    }
    cjk_run.clear();
}

fn push_unique_word(words: &mut Vec<String>, word: String) {
    if !words.iter().any(|existing| existing == &word) {
        words.push(word);
    }
}

fn token_set_has_match(tokens: &HashSet<String>, query_token: &str) -> bool {
    tokens
        .iter()
        .any(|candidate_token| lexical_token_matches(query_token, candidate_token))
}

fn lexical_token_matches(query_token: &str, candidate_token: &str) -> bool {
    if query_token == candidate_token {
        return true;
    }
    if token_is_cjk(query_token) || token_is_cjk(candidate_token) {
        return false;
    }
    let query_stem = lexical_stem(query_token);
    let candidate_stem = lexical_stem(candidate_token);
    query_stem == candidate_stem
        || (query_token.len() >= 5
            && candidate_token.len() >= 5
            && (candidate_token.contains(query_token) || query_token.contains(candidate_token)))
}

fn lexical_stem(token: &str) -> &str {
    for suffix in ["ing", "ed", "es", "s"] {
        if token.len() > suffix.len() + 3
            && let Some(stem) = token.strip_suffix(suffix)
        {
            return stem;
        }
    }
    token
}

fn token_is_cjk(token: &str) -> bool {
    token.chars().any(is_cjk_character)
}

fn is_cjk_character(character: char) -> bool {
    matches!(
        character,
        '\u{3400}'..='\u{4dbf}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{f900}'..='\u{faff}'
            | '\u{20000}'..='\u{2a6df}'
            | '\u{2a700}'..='\u{2b73f}'
            | '\u{2b740}'..='\u{2b81f}'
            | '\u{2b820}'..='\u{2ceaf}'
    )
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        return 0.0;
    }
    let numerator = u16::try_from(numerator.min(usize::from(u16::MAX))).unwrap_or(u16::MAX);
    let denominator = u16::try_from(denominator.min(usize::from(u16::MAX))).unwrap_or(u16::MAX);
    f32::from(numerator) / f32::from(denominator)
}

fn task_row_file_key(row: &AgentOrgTaskListRow) -> String {
    Path::new(row.source_path.as_str())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(row.source_path.as_str())
        .to_string()
}

fn property_facet_kind(key: &str) -> OrgEvidenceFacetKind {
    match key {
        "NEXT_ACTION" | "RESUME_QUERY" => OrgEvidenceFacetKind::NextAction,
        "SDD" | "SDD_PARENT" | "SDD_KIND" | "SDD_STATUS" => OrgEvidenceFacetKind::Graph,
        _ => OrgEvidenceFacetKind::Property,
    }
}

fn push_evidence_facet(
    facets: &mut Vec<OrgEvidenceFacet>,
    kind: OrgEvidenceFacetKind,
    label: &str,
    value: Option<&str>,
) {
    let Some(value) = value else {
        return;
    };
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "none" {
        return;
    }
    let text = format!("{label}: {trimmed}");
    let tokens = normalized_words(text.as_str()).into_iter().collect();
    facets.push(OrgEvidenceFacet { kind, text, tokens });
}

fn task_sdd_evidence_path(row: &AgentOrgTaskListRow) -> Option<PathBuf> {
    let raw = property_value(row, "SDD")?.trim();
    if raw.is_empty()
        || raw == "none"
        || raw.starts_with('<')
        || raw.starts_with("id:")
        || raw.starts_with("http://")
        || raw.starts_with("https://")
    {
        return None;
    }

    let root = task_source_project_root(row);
    let expanded = expand_task_sdd_path(raw, root.as_path());
    let path = PathBuf::from(expanded);
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    path.is_file().then_some(path)
}

fn task_source_project_root(row: &AgentOrgTaskListRow) -> PathBuf {
    let source = Path::new(row.source_path.as_str());
    let source_text = source.to_string_lossy();
    if let Some(index) = source_text.find("/.cache/agent/org/") {
        return PathBuf::from(&source_text[..index]);
    }
    if let Some(rest) = source_text.strip_prefix(".cache/agent/org/")
        && rest != source_text
    {
        return PathBuf::from(".");
    }
    source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn expand_task_sdd_path(raw: &str, root: &Path) -> String {
    let cache_home = root.join(".cache");
    let replacements = [
        ("${PRJ_CACHE_HOME}", cache_home.as_path()),
        ("$PRJ_CACHE_HOME", cache_home.as_path()),
        ("${PRJ_ROOT}", root),
        ("$PRJ_ROOT", root),
    ];
    let mut expanded = raw.trim().to_string();
    for (token, path) in replacements {
        let path = path.to_string_lossy();
        if expanded == token {
            expanded = path.into_owned();
        } else if let Some(rest) = expanded.strip_prefix(&format!("{token}/")) {
            expanded = format!("{path}/{rest}");
        }
    }
    expanded
}

fn org_keyword_value_from_line<'a>(trimmed: &'a str, key: &str) -> Option<&'a str> {
    let (raw_key, value) = trimmed.strip_prefix("#+")?.split_once(':')?;
    raw_key
        .trim()
        .eq_ignore_ascii_case(key)
        .then_some(value.trim())
        .filter(|value| !value.is_empty())
}

fn org_property_from_line(trimmed: &str) -> Option<(&str, &str)> {
    let rest = trimmed.strip_prefix(':')?;
    let (key, value) = rest.split_once(':')?;
    let key = key.trim();
    let value = value.trim();
    (!key.is_empty() && !value.is_empty()).then_some((key, value))
}

fn sdd_property_is_evidence(key: &str) -> bool {
    matches!(
        key,
        "ID" | "SDD_KIND" | "SDD_STATUS" | "SDD_PARENT" | "SDD_RATIONALE" | "SDD_DECISION"
    )
}

fn org_heading_title_for_evidence(trimmed: &str) -> Option<String> {
    let level = trimmed.bytes().take_while(|byte| *byte == b'*').count();
    if level == 0
        || !trimmed
            .as_bytes()
            .get(level)
            .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    let mut title = trimmed[level..].trim();
    for keyword in ["TODO", "DOING", "NEXT", "WAITING", "DONE", "CANCELLED"] {
        if let Some(rest) = title.strip_prefix(keyword) {
            title = rest.trim_start();
            break;
        }
    }
    if title.starts_with("[#") && title.get(3..4) == Some("]") {
        title = title[4..].trim_start();
    }
    let tagless = strip_org_heading_tags(title).trim();
    (!tagless.is_empty()).then(|| tagless.to_string())
}

fn strip_org_heading_tags(title: &str) -> &str {
    let Some((before, after)) = title.rsplit_once(' ') else {
        return title;
    };
    if after.starts_with(':')
        && after.ends_with(':')
        && after.trim_matches(':').split(':').all(|tag| {
            !tag.is_empty()
                && tag
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '#')
        })
    {
        before
    } else {
        title
    }
}

#[derive(Debug, Clone)]
struct TemporalRecallContext {
    oldest_modified_unix_ms: u64,
    newest_modified_unix_ms: u64,
    source_line_spans: HashMap<String, (u64, u64)>,
}

impl TemporalRecallContext {
    fn from_candidates(candidates: &[RecallCandidate<'_>]) -> Self {
        let modified_times = candidates
            .iter()
            .map(|candidate| candidate.row.source_modified_unix_ms)
            .filter(|modified| *modified > 0)
            .collect::<Vec<_>>();
        let mut source_line_spans = HashMap::<String, (u64, u64)>::new();
        for candidate in candidates {
            let entry = source_line_spans
                .entry(candidate.row.source_path.clone())
                .or_insert((candidate.row.source_line, candidate.row.source_line));
            entry.0 = entry.0.min(candidate.row.source_line);
            entry.1 = entry.1.max(candidate.row.source_line);
        }
        Self {
            oldest_modified_unix_ms: modified_times.iter().min().copied().unwrap_or_default(),
            newest_modified_unix_ms: modified_times.iter().max().copied().unwrap_or_default(),
            source_line_spans,
        }
    }

    fn modified_recency_bonus(&self, row: &AgentOrgTaskListRow) -> f32 {
        let mut bonus = if let Some(relative) = self.modified_relative_position(row) {
            0.02 + (0.06 * relative)
        } else {
            absolute_modified_recency_bonus(self.modified_age_ms(row))
        };
        if let Some(line_position) = self.source_line_relative_position(row) {
            bonus += 0.035 * line_position;
        }
        bonus
    }

    fn source_line_relative_position(&self, row: &AgentOrgTaskListRow) -> Option<f32> {
        let (min_line, max_line) = self
            .source_line_spans
            .get(row.source_path.as_str())
            .copied()?;
        if min_line == max_line {
            return None;
        }
        let span = max_line.saturating_sub(min_line);
        if span == 0 {
            return None;
        }
        relative_position(row.source_line.saturating_sub(min_line), span)
    }

    fn modified_relative_position(&self, row: &AgentOrgTaskListRow) -> Option<f32> {
        if self.oldest_modified_unix_ms == 0
            || self.newest_modified_unix_ms == 0
            || self.oldest_modified_unix_ms == self.newest_modified_unix_ms
            || row.source_modified_unix_ms == 0
        {
            return None;
        }
        let span = self
            .newest_modified_unix_ms
            .saturating_sub(self.oldest_modified_unix_ms);
        if span == 0 {
            return None;
        }
        let position = row
            .source_modified_unix_ms
            .saturating_sub(self.oldest_modified_unix_ms);
        relative_position(position, span)
    }

    fn modified_age_ms(&self, row: &AgentOrgTaskListRow) -> Option<u64> {
        if self.newest_modified_unix_ms == 0 || row.source_modified_unix_ms == 0 {
            return None;
        }
        Some(
            self.newest_modified_unix_ms
                .saturating_sub(row.source_modified_unix_ms),
        )
    }
}

fn absolute_modified_recency_bonus(age: Option<u64>) -> f32 {
    match age {
        Some(age) if age <= minutes(5) => 0.08,
        Some(age) if age <= hours(1) => 0.06,
        Some(age) if age <= hours(6) => 0.045,
        Some(age) if age <= hours(24) => 0.03,
        Some(age) if age <= days(7) => 0.015,
        None | Some(_) => 0.0,
    }
}

fn relative_position(position: u64, span: u64) -> Option<f32> {
    if span == 0 {
        return None;
    }
    let scaled = position.saturating_mul(1_000).checked_div(span)?;
    let clamped = u16::try_from(scaled.min(1_000)).ok()?;
    Some(f32::from(clamped) / 1_000.0)
}

const fn minutes(value: u64) -> u64 {
    value * 60 * 1_000
}

const fn hours(value: u64) -> u64 {
    minutes(value * 60)
}

const fn days(value: u64) -> u64 {
    hours(value * 24)
}

fn task_row_section_lens(row: &AgentOrgTaskListRow) -> Option<TaskSectionLens> {
    let source = std::fs::read_to_string(row.source_path.as_str()).ok()?;
    let start = usize::try_from(row.source_range_start).ok()?;
    let end = usize::try_from(row.source_range_end).ok()?;
    let section = source.get(start..end)?;
    Some(TaskSectionLens::from_section(section))
}
