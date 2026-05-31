use super::facets::candidate_evidence_from_candidate;
use super::model::{EvidenceCorpus, QueryEvidence, RankedCandidate, RecallCandidate};
use super::scoring::{
    candidate_identity_matches, candidate_org_facet_score, compare_ranked_candidates,
    prune_completed_noise, temporary_memory_scores,
};
use super::temporal::{TemporalRecallContext, task_row_section_lens};
use crate::orgize::read_model::model::AgentOrgTaskListRow;

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
        .map(|row| RecallCandidate {
            row,
            lens: task_row_section_lens(row),
        })
        .collect::<Vec<_>>();
    let candidates = prune_completed_noise(
        candidates,
        &query,
        scope.include_done,
        scope.include_archived,
    );
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
        .map(candidate_evidence_from_candidate)
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
