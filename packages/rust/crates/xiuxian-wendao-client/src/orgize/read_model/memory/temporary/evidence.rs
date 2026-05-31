use super::model::{OrgEvidenceFacet, OrgEvidenceFacetKind};
use super::token::normalized_words;

pub(super) fn push_evidence_facet(
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
