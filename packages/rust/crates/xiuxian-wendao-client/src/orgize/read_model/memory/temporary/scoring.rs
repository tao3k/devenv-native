use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use xiuxian_memory_engine::{
    Episode, EpisodeDraft, IntentEncoder, QTable, TwoPhaseSearch, TwoPhaseSearchRequest,
    infer_memory_lifecycle_facts_from_properties,
};

use super::model::{
    CandidateEvidence, EvidenceCorpus, OrgEvidenceFacetKind, QueryEvidence, RankedCandidate,
    RecallCandidate, RecallScore,
};
use super::temporal::TemporalRecallContext;
use super::token::{normalized_text, token_set_has_match};
use crate::orgize::read_model::section_lens::TaskSectionLens;

const TEMPORARY_MEMORY_SCOPE: &str = "wendao-client:agent-org-temporary-memory";
const EMBEDDING_DIMENSION: usize = 128;
const MEMORY_Q_WEIGHT: f32 = 0.35;

pub(super) fn compare_ranked_candidates(
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

pub(super) fn candidate_org_facet_score(
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

pub(super) fn temporary_memory_scores(
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
    let lifecycle_priors = candidates
        .iter()
        .map(|candidate| {
            (
                candidate.row.orgid.as_str(),
                candidate_memory_recall_prior(candidate),
            )
        })
        .collect::<HashMap<_, _>>();
    let episodes = candidates
        .iter()
        .zip(evidence_windows.iter())
        .zip(base_scores.iter())
        .map(|((candidate, evidence), score)| {
            let lifecycle_prior = lifecycle_priors
                .get(candidate.row.orgid.as_str())
                .copied()
                .unwrap_or(1.0);
            let reward = score.utility_value() * lifecycle_prior;
            q_table.update(candidate.row.orgid.as_str(), reward);
            let intent = temporary_memory_episode_intent(candidate, evidence);
            Episode::new(EpisodeDraft {
                id: candidate.row.orgid.as_str().into(),
                intent_embedding: encoder.encode(intent.as_str()),
                intent,
                experience: temporary_memory_episode_experience(
                    candidate,
                    evidence,
                    reward,
                    lifecycle_prior,
                ),
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
        .map(|(episode, score)| {
            let lifecycle_prior = lifecycle_priors
                .get(episode.id.as_str())
                .copied()
                .unwrap_or(1.0);
            (episode.id, score * lifecycle_prior)
        })
        .collect()
}

pub(super) fn candidate_memory_recall_prior(candidate: &RecallCandidate<'_>) -> f32 {
    infer_memory_lifecycle_facts_from_properties(
        candidate
            .row
            .properties
            .iter()
            .map(|property| (property.key.as_str(), property.value.as_str())),
    )
    .evaluate()
    .recall_prior
}

pub(super) fn temporary_memory_query_intent(query: &QueryEvidence) -> String {
    query.raw.clone()
}

pub(super) fn temporary_memory_episode_intent(
    candidate: &RecallCandidate<'_>,
    evidence: &CandidateEvidence,
) -> String {
    format!("orgid: {}\n{}", candidate.row.orgid, evidence.text)
}

pub(super) fn temporary_memory_episode_experience(
    candidate: &RecallCandidate<'_>,
    evidence: &CandidateEvidence,
    reward: f32,
    lifecycle_prior: f32,
) -> String {
    let facet_labels = evidence
        .facets
        .iter()
        .map(|facet| facet.kind.label())
        .collect::<HashSet<_>>();
    let mut facet_labels = facet_labels.into_iter().collect::<Vec<_>>();
    facet_labels.sort_unstable();
    let mut experience = format!(
        "source: {}:{}\nfacets: {}\norg-utility: {:.3}\nlifecycle-prior: {:.3}",
        candidate.row.source_path,
        candidate.row.source_line,
        facet_labels.join(","),
        reward,
        lifecycle_prior,
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

pub(super) fn matched_org_facet_kinds(
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

pub(super) fn org_facet_signal(facets: &HashSet<OrgEvidenceFacetKind>) -> f32 {
    facets
        .iter()
        .map(|facet| match facet {
            OrgEvidenceFacetKind::Identity => 0.18,
            OrgEvidenceFacetKind::Heading | OrgEvidenceFacetKind::NextAction => 0.14,
            OrgEvidenceFacetKind::Checklist => 0.13,
            OrgEvidenceFacetKind::Graph => 0.12,
            OrgEvidenceFacetKind::MemoryFinality
            | OrgEvidenceFacetKind::MemoryClaim
            | OrgEvidenceFacetKind::MemoryEvidence
            | OrgEvidenceFacetKind::MemoryFailure
            | OrgEvidenceFacetKind::MemoryPreference => 0.16,
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

pub(super) fn org_facet_coverage_score(
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

pub(super) fn org_recovery_anchor_coverage_score(
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

pub(super) const fn facet_is_recovery_anchor(kind: OrgEvidenceFacetKind) -> bool {
    matches!(
        kind,
        OrgEvidenceFacetKind::Identity
            | OrgEvidenceFacetKind::Source
            | OrgEvidenceFacetKind::NextAction
            | OrgEvidenceFacetKind::Graph
            | OrgEvidenceFacetKind::MemoryFinality
            | OrgEvidenceFacetKind::MemoryClaim
            | OrgEvidenceFacetKind::MemoryEvidence
            | OrgEvidenceFacetKind::MemoryFailure
            | OrgEvidenceFacetKind::MemoryPreference
            | OrgEvidenceFacetKind::Checklist
    )
}

pub(super) fn facet_coverage_weight(kind: OrgEvidenceFacetKind) -> f32 {
    match kind {
        OrgEvidenceFacetKind::Identity => 1.0,
        OrgEvidenceFacetKind::Heading => 0.98,
        OrgEvidenceFacetKind::MemoryFinality
        | OrgEvidenceFacetKind::MemoryClaim
        | OrgEvidenceFacetKind::MemoryEvidence
        | OrgEvidenceFacetKind::MemoryFailure
        | OrgEvidenceFacetKind::MemoryPreference => 0.99,
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

pub(super) fn token_information_coverage(
    query: &QueryEvidence,
    evidence: &CandidateEvidence,
    corpus: &EvidenceCorpus,
) -> f32 {
    token_information_coverage_for_tokens(query, &evidence.tokens, corpus)
}

pub(super) fn token_information_coverage_for_tokens(
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

pub(super) fn prune_completed_noise<'a>(
    candidates: Vec<RecallCandidate<'a>>,
    query: &QueryEvidence,
    include_done: bool,
    include_archived: bool,
) -> Vec<RecallCandidate<'a>> {
    if candidates.len() <= 1 {
        return candidates;
    }
    if include_done || include_archived {
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

pub(super) fn candidate_identity_matches(
    candidate: &RecallCandidate<'_>,
    query: &QueryEvidence,
) -> bool {
    let normalized_orgid = normalized_text(candidate.row.orgid.as_str());
    !query.is_empty()
        && (query.raw.eq_ignore_ascii_case(candidate.row.orgid.as_str())
            || (!query.normalized.is_empty() && query.normalized == normalized_orgid))
}

pub(super) fn task_candidate_is_completed_noise(candidate: &RecallCandidate<'_>) -> bool {
    if candidate.row.is_done || candidate.row.closed.is_some() {
        return true;
    }
    title_has_complete_progress_cookie(candidate.row.title.as_str())
        || candidate
            .lens
            .as_ref()
            .is_some_and(TaskSectionLens::is_complete)
}

pub(super) fn title_has_complete_progress_cookie(title: &str) -> bool {
    title.contains("[100%]")
        || title
            .split_whitespace()
            .any(token_has_complete_ratio_cookie)
}

pub(super) fn token_has_complete_ratio_cookie(token: &str) -> bool {
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

pub(super) fn lifecycle_signal(candidate: &RecallCandidate<'_>) -> f32 {
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
