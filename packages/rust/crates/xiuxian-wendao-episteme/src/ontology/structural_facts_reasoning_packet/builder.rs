//! Row builder for structural-facts reasoning packets.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail};
use sha2::{Digest, Sha256};

use crate::ontology::reasoning_target::classify_document_target;

use super::{
    input::{
        StructuralFactsAnchorInput, StructuralFactsDocumentInput, read_structural_facts_input,
    },
    types::{
        EpistemeOntologyStructuralFactsReasoningPacketRequest,
        EpistemeOntologyStructuralFactsReasoningPacketRow,
    },
};

const PACKET_KIND: &str = "document_reasoning_seed";
const EVIDENCE_ACTION: &str = "read_targeted_evidence_before_proposal";
const STATUS_PENDING_REASONING: &str = "pending_reasoning";

pub(super) struct ReasoningPacketBuild {
    pub rows: Vec<EpistemeOntologyStructuralFactsReasoningPacketRow>,
    pub skipped_by_filter_count: usize,
    pub skipped_by_limit_count: usize,
}

#[derive(Default)]
struct PacketSelection {
    seen_packet_ids: BTreeSet<String>,
    rows: Vec<EpistemeOntologyStructuralFactsReasoningPacketRow>,
    skipped_by_filter_count: usize,
    skipped_by_limit_count: usize,
}

pub(super) fn build_reasoning_packet_rows(
    request: &EpistemeOntologyStructuralFactsReasoningPacketRequest,
) -> Result<ReasoningPacketBuild> {
    validate_run_id(&request.run_id)?;
    if request.limit == 0 {
        bail!("reasoning packet limit must be greater than zero");
    }

    let input = read_structural_facts_input(request.structural_facts_json.as_path())?;
    let document_anchors = document_root_anchors(&input.anchors)?;
    let selection = input.documents.iter().try_fold(
        PacketSelection::default(),
        |mut selection, document| {
            validate_document(document, &document_anchors)?;
            if !matches_filters(document, request) {
                selection.skipped_by_filter_count += 1;
                return Ok(selection);
            }
            if selection.rows.len() >= request.limit {
                selection.skipped_by_limit_count += 1;
                return Ok(selection);
            }
            let Some(anchor) = document_anchors.get(document.document_id.as_str()) else {
                bail!(
                    "structural document `{}` has no document_root anchor",
                    document.document_id
                );
            };
            let row = packet_row(document, anchor, request.run_id.as_str());
            if !selection.seen_packet_ids.insert(row.packet_id.clone()) {
                bail!("duplicate reasoning packet id: {}", row.packet_id);
            }
            selection.rows.push(row);
            Ok::<_, anyhow::Error>(selection)
        },
    )?;

    if selection.rows.is_empty() {
        bail!("reasoning packet selection produced no rows");
    }

    Ok(ReasoningPacketBuild {
        rows: selection.rows,
        skipped_by_filter_count: selection.skipped_by_filter_count,
        skipped_by_limit_count: selection.skipped_by_limit_count,
    })
}

fn document_root_anchors(
    anchors: &[StructuralFactsAnchorInput],
) -> Result<BTreeMap<&str, &StructuralFactsAnchorInput>> {
    let mut document_anchors = BTreeMap::new();
    for anchor in anchors
        .iter()
        .filter(|anchor| anchor.anchor_kind == "document_root")
    {
        if anchor.ontology_truth {
            bail!(
                "structural document anchor `{}` attempted to mark ontology truth",
                anchor.anchor_id
            );
        }
        if anchor.document_id.trim().is_empty() {
            bail!(
                "structural document anchor `{}` has blank document_id",
                anchor.anchor_id
            );
        }
        if document_anchors
            .insert(anchor.document_id.as_str(), anchor)
            .is_some()
        {
            bail!(
                "structural facts input has duplicate document_root anchors for `{}`",
                anchor.document_id
            );
        }
    }
    Ok(document_anchors)
}

fn validate_document(
    document: &StructuralFactsDocumentInput,
    document_anchors: &BTreeMap<&str, &StructuralFactsAnchorInput>,
) -> Result<()> {
    if document.ontology_truth {
        bail!(
            "structural document `{}` attempted to mark ontology truth",
            document.document_id
        );
    }
    let Some(anchor) = document_anchors.get(document.document_id.as_str()) else {
        bail!(
            "structural document `{}` has no document_root anchor",
            document.document_id
        );
    };
    if anchor.file_id != document.file_id {
        bail!(
            "document_root anchor `{}` file_id `{}` does not match document `{}` file_id `{}`",
            anchor.anchor_id,
            anchor.file_id,
            document.document_id,
            document.file_id
        );
    }
    if anchor.source_content_hash != document.sha256 {
        bail!(
            "document_root anchor `{}` hash does not match document `{}`",
            anchor.anchor_id,
            document.document_id
        );
    }
    Ok(())
}

fn matches_filters(
    document: &StructuralFactsDocumentInput,
    request: &EpistemeOntologyStructuralFactsReasoningPacketRequest,
) -> bool {
    let category_matches = request
        .category
        .as_ref()
        .is_none_or(|category| document.category == *category);
    let route_matches = request
        .route
        .as_ref()
        .is_none_or(|route| document.extraction_route == *route);
    category_matches && route_matches
}

fn packet_row(
    document: &StructuralFactsDocumentInput,
    anchor: &StructuralFactsAnchorInput,
    run_id: &str,
) -> EpistemeOntologyStructuralFactsReasoningPacketRow {
    let target = classify_document_target(
        document.relative_path.as_str(),
        document.category.as_str(),
        document.extraction_route.as_str(),
    );
    EpistemeOntologyStructuralFactsReasoningPacketRow {
        packet_id: stable_packet_id(
            run_id,
            document.document_id.as_str(),
            anchor.anchor_id.as_str(),
        ),
        packet_kind: PACKET_KIND,
        reasoning_task_kind: reasoning_task_kind(document.extraction_route.as_str()).to_string(),
        evidence_target_intent: target.evidence_target_intent,
        evidence_anchor_kind: anchor.anchor_kind.clone(),
        evidence_structure_hint: target.evidence_structure_hint,
        document_id: document.document_id.clone(),
        document_anchor_id: anchor.anchor_id.clone(),
        file_id: document.file_id.clone(),
        domain_id: document.domain_id.clone(),
        source_contract_id: document.source_contract_id.clone(),
        relative_path: document.relative_path.clone(),
        category: document.category.clone(),
        language: document.language.clone(),
        extraction_route: document.extraction_route.clone(),
        source_content_hash: document.sha256.clone(),
        evidence_action: EVIDENCE_ACTION,
        ontology_truth: false,
        status: STATUS_PENDING_REASONING,
    }
}

fn reasoning_task_kind(extraction_route: &str) -> &'static str {
    match extraction_route {
        "audio_asr_evidence" => "audio_transcript_ontology_proposal",
        "image_ocr_evidence" => "image_ocr_ontology_proposal",
        "legacy_office_document_evidence" => "legacy_office_document_ontology_proposal",
        "document_text_evidence" => "document_text_ontology_proposal",
        _ => "source_document_ontology_proposal",
    }
}

fn stable_packet_id(run_id: &str, document_id: &str, anchor_id: &str) -> String {
    let digest = Sha256::digest(format!("{run_id}:{document_id}:{anchor_id}").as_bytes());
    let suffix = format!("{digest:x}").chars().take(16).collect::<String>();
    format!("structural_facts.reasoning_packet.{suffix}")
}

fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        bail!("invalid run id `{run_id}`; use ASCII letters, digits, '.', '_', or '-'");
    }
    Ok(())
}
