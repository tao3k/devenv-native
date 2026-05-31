use std::collections::HashSet;
use std::path::Path;

use xiuxian_memory_engine::InferredMemoryObjectKind;

use super::evidence::push_evidence_facet;
use super::model::{CandidateEvidence, OrgEvidenceFacet, OrgEvidenceFacetKind, RecallCandidate};
use super::sdd::push_candidate_sdd_facets;
use super::token::normalized_text;
use crate::orgize::read_model::memory::inferred_memory_objects_for_row;
use crate::orgize::read_model::model::AgentOrgTaskListRow;

pub(super) fn candidate_evidence_from_candidate(
    candidate: &RecallCandidate<'_>,
) -> CandidateEvidence {
    let mut facets = Vec::new();
    push_candidate_base_facets(&mut facets, candidate);
    push_candidate_property_facets(&mut facets, candidate);
    push_candidate_reflection_memory_facets(&mut facets, candidate);
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
    CandidateEvidence {
        text,
        normalized,
        tokens,
        facets,
    }
}

pub(super) fn push_candidate_base_facets(
    facets: &mut Vec<OrgEvidenceFacet>,
    candidate: &RecallCandidate<'_>,
) {
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

pub(super) fn push_candidate_property_facets(
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

pub(super) fn push_candidate_reflection_memory_facets(
    facets: &mut Vec<OrgEvidenceFacet>,
    candidate: &RecallCandidate<'_>,
) {
    for object in inferred_memory_objects_for_row(candidate.row) {
        let value = format!("{} {}", object.question, object.value);
        push_evidence_facet(
            facets,
            memory_object_facet_kind(object.kind),
            object.kind.facet_label(),
            Some(value.as_str()),
        );
    }
}

pub(super) fn push_candidate_lens_facets(
    facets: &mut Vec<OrgEvidenceFacet>,
    candidate: &RecallCandidate<'_>,
) {
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

pub(super) const fn memory_object_facet_kind(
    kind: InferredMemoryObjectKind,
) -> OrgEvidenceFacetKind {
    match kind {
        InferredMemoryObjectKind::Finality => OrgEvidenceFacetKind::MemoryFinality,
        InferredMemoryObjectKind::Claim => OrgEvidenceFacetKind::MemoryClaim,
        InferredMemoryObjectKind::Evidence => OrgEvidenceFacetKind::MemoryEvidence,
        InferredMemoryObjectKind::Failure => OrgEvidenceFacetKind::MemoryFailure,
        InferredMemoryObjectKind::Preference => OrgEvidenceFacetKind::MemoryPreference,
    }
}

pub(super) fn property_facet_kind(key: &str) -> OrgEvidenceFacetKind {
    match key {
        "NEXT_ACTION" | "RESUME_QUERY" => OrgEvidenceFacetKind::NextAction,
        "SDD" | "SDD_PARENT" | "SDD_KIND" | "SDD_STATUS" => OrgEvidenceFacetKind::Graph,
        _ => OrgEvidenceFacetKind::Property,
    }
}

fn task_row_file_key(row: &AgentOrgTaskListRow) -> String {
    Path::new(row.source_path.as_str())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(row.source_path.as_str())
        .to_string()
}
