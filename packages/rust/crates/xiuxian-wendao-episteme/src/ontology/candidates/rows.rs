use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use xiuxian_wendao_parsers::EpistemeFileRow;

use super::{
    identifiers::{evidence_candidate_id, relation_candidate_id, short_hash, source_candidate_id},
    model::{
        CacheEvidence, CandidateEvidenceRow, CandidateGenerationInputs, CandidateObjectRow,
        CandidateRelationRow, CandidateRows, EVIDENCE_SUPPORTS_SOURCE_RELATION,
        EXTRACTION_EVIDENCE_KIND, MappingTerm, MappingTermKind, ONTOLOGY_TRUTH, PROMOTION_STATUS,
        RAW_TO_RDF_PROMOTION_ALLOWED, REVIEW_STATUS, SOURCE_ARTIFACT_KIND, STATUS_CANDIDATE,
        SUGGESTED_OBJECT_TYPE_RELATION,
    },
};

pub(super) fn build_candidate_rows(inputs: &CandidateGenerationInputs) -> CandidateRows {
    let object_terms = inputs
        .mapping_terms
        .iter()
        .filter(|term| term.term_kind == MappingTermKind::Object)
        .collect::<Vec<_>>();
    let cache_by_file_id = cache_evidence_by_file_id(&inputs.cache_evidence);
    let mut rows = CandidateRows {
        objects: Vec::new(),
        relations: Vec::new(),
        evidence: Vec::new(),
    };
    rows.objects.extend(
        inputs
            .mapping_terms
            .iter()
            .map(|term| mapping_term_object_row(term, inputs)),
    );
    add_source_file_rows(inputs, &object_terms, &cache_by_file_id, &mut rows);
    add_cache_evidence_rows(inputs, &mut rows);
    rows
}

fn add_source_file_rows(
    inputs: &CandidateGenerationInputs,
    object_terms: &[&MappingTerm],
    cache_by_file_id: &BTreeMap<&str, Vec<&CacheEvidence>>,
    rows: &mut CandidateRows,
) {
    for file in &inputs.files {
        let matched_term = best_mapping_term(
            file,
            cache_by_file_id.get(file.file_id.as_str()),
            object_terms,
        );
        rows.objects
            .push(source_file_object_row(file, matched_term));
        if let Some(term) = matched_term {
            rows.relations
                .push(source_to_mapping_term_relation_row(file, term));
        }
    }
}

fn add_cache_evidence_rows(inputs: &CandidateGenerationInputs, rows: &mut CandidateRows) {
    for evidence in &inputs.cache_evidence {
        let evidence_candidate_id =
            evidence_candidate_id(evidence.run_id.as_str(), evidence.queue_id.as_str());
        rows.objects.push(cache_evidence_object_row(evidence));
        rows.relations.push(cache_evidence_relation_row(
            evidence,
            evidence_candidate_id.as_str(),
        ));
        rows.evidence.push(cache_evidence_tsv_row(
            evidence,
            evidence_candidate_id.as_str(),
        ));
    }
}

fn mapping_term_object_row(
    term: &MappingTerm,
    inputs: &CandidateGenerationInputs,
) -> CandidateObjectRow {
    CandidateObjectRow {
        candidate_id: term.candidate_id.clone(),
        candidate_kind: term.term_kind.candidate_kind(),
        status: STATUS_CANDIDATE,
        label: term.label.clone(),
        suggested_term_key: term.stable_key.clone(),
        suggested_term_label: term.label.clone(),
        source_file_id: String::new(),
        source_queue_id: String::new(),
        source_path: inputs.mapping_ledger_path.clone(),
        category: "mapping_ledger".to_string(),
        language: inputs.primary_language.clone(),
        extraction_route: "mapping_ledger".to_string(),
        extraction_run_id: String::new(),
        source_sha256: inputs.source_revision.clone(),
        evidence_sha256: short_hash(term.note.as_str()),
        text_char_count: "0".to_string(),
        review_status: REVIEW_STATUS,
        promotion_status: PROMOTION_STATUS,
        raw_to_rdf_promotion_allowed: RAW_TO_RDF_PROMOTION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    }
}

fn source_file_object_row(
    file: &EpistemeFileRow,
    matched_term: Option<&MappingTerm>,
) -> CandidateObjectRow {
    CandidateObjectRow {
        candidate_id: source_candidate_id(file.file_id.as_str()),
        candidate_kind: SOURCE_ARTIFACT_KIND,
        status: STATUS_CANDIDATE,
        label: file_label(file.relative_path.as_str()),
        suggested_term_key: matched_term
            .map(|term| term.stable_key.clone())
            .unwrap_or_default(),
        suggested_term_label: matched_term
            .map(|term| term.label.clone())
            .unwrap_or_default(),
        source_file_id: file.file_id.clone(),
        source_queue_id: String::new(),
        source_path: file.relative_path.clone(),
        category: file.category.clone(),
        language: file.language.clone(),
        extraction_route: file.extraction_route.clone(),
        extraction_run_id: String::new(),
        source_sha256: file.sha256.clone(),
        evidence_sha256: file.sha256.clone(),
        text_char_count: "0".to_string(),
        review_status: REVIEW_STATUS,
        promotion_status: PROMOTION_STATUS,
        raw_to_rdf_promotion_allowed: RAW_TO_RDF_PROMOTION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    }
}

fn cache_evidence_object_row(evidence: &CacheEvidence) -> CandidateObjectRow {
    CandidateObjectRow {
        candidate_id: evidence_candidate_id(evidence.run_id.as_str(), evidence.queue_id.as_str()),
        candidate_kind: EXTRACTION_EVIDENCE_KIND,
        status: STATUS_CANDIDATE,
        label: format!("{} evidence", evidence.queue_id),
        suggested_term_key: String::new(),
        suggested_term_label: String::new(),
        source_file_id: evidence.file_id.clone(),
        source_queue_id: evidence.queue_id.clone(),
        source_path: evidence.relative_path.clone(),
        category: evidence.category.clone(),
        language: evidence.language.clone(),
        extraction_route: evidence.extraction_route.clone(),
        extraction_run_id: evidence.run_id.clone(),
        source_sha256: evidence.source_sha256.clone(),
        evidence_sha256: evidence.text_sha256.clone(),
        text_char_count: evidence.text_char_count.to_string(),
        review_status: REVIEW_STATUS,
        promotion_status: PROMOTION_STATUS,
        raw_to_rdf_promotion_allowed: RAW_TO_RDF_PROMOTION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    }
}

fn source_to_mapping_term_relation_row(
    file: &EpistemeFileRow,
    term: &MappingTerm,
) -> CandidateRelationRow {
    let source_candidate_id = source_candidate_id(file.file_id.as_str());
    CandidateRelationRow {
        candidate_id: relation_candidate_id(
            SUGGESTED_OBJECT_TYPE_RELATION,
            source_candidate_id.as_str(),
            term.candidate_id.as_str(),
            "",
        ),
        relation_kind: SUGGESTED_OBJECT_TYPE_RELATION,
        source_candidate_id,
        target_candidate_id: term.candidate_id.clone(),
        source_file_id: file.file_id.clone(),
        source_queue_id: String::new(),
        extraction_run_id: String::new(),
        evidence_sha256: file.sha256.clone(),
        review_status: REVIEW_STATUS,
        promotion_status: PROMOTION_STATUS,
        ontology_truth: ONTOLOGY_TRUTH,
    }
}

fn cache_evidence_relation_row(
    evidence: &CacheEvidence,
    evidence_candidate_id: &str,
) -> CandidateRelationRow {
    CandidateRelationRow {
        candidate_id: relation_candidate_id(
            EVIDENCE_SUPPORTS_SOURCE_RELATION,
            evidence_candidate_id,
            source_candidate_id(evidence.file_id.as_str()).as_str(),
            evidence.run_id.as_str(),
        ),
        relation_kind: EVIDENCE_SUPPORTS_SOURCE_RELATION,
        source_candidate_id: evidence_candidate_id.to_string(),
        target_candidate_id: source_candidate_id(evidence.file_id.as_str()),
        source_file_id: evidence.file_id.clone(),
        source_queue_id: evidence.queue_id.clone(),
        extraction_run_id: evidence.run_id.clone(),
        evidence_sha256: evidence.text_sha256.clone(),
        review_status: REVIEW_STATUS,
        promotion_status: PROMOTION_STATUS,
        ontology_truth: ONTOLOGY_TRUTH,
    }
}

fn cache_evidence_tsv_row(
    evidence: &CacheEvidence,
    evidence_candidate_id: &str,
) -> CandidateEvidenceRow {
    CandidateEvidenceRow {
        evidence_id: format!("evidence:{evidence_candidate_id}"),
        evidence_kind: "extraction_cache_text_hash",
        source_file_id: evidence.file_id.clone(),
        source_queue_id: evidence.queue_id.clone(),
        source_path: evidence.relative_path.clone(),
        source_sha256: evidence.source_sha256.clone(),
        extraction_run_id: evidence.run_id.clone(),
        cache_output_path: evidence.output_path.clone(),
        evidence_sha256: evidence.text_sha256.clone(),
        text_char_count: evidence.text_char_count.to_string(),
        review_status: REVIEW_STATUS,
        promotion_status: PROMOTION_STATUS,
        ontology_truth: ONTOLOGY_TRUTH,
    }
}

fn cache_evidence_by_file_id(evidence: &[CacheEvidence]) -> BTreeMap<&str, Vec<&CacheEvidence>> {
    let mut by_file = BTreeMap::<&str, Vec<&CacheEvidence>>::new();
    for row in evidence {
        by_file.entry(row.file_id.as_str()).or_default().push(row);
    }
    by_file
}

fn best_mapping_term<'a>(
    file: &EpistemeFileRow,
    evidence: Option<&Vec<&CacheEvidence>>,
    terms: &[&'a MappingTerm],
) -> Option<&'a MappingTerm> {
    let mut haystack = format!(
        "{}\n{}\n{}\n{}",
        file.relative_path, file.category, file.language, file.extraction_route
    );
    if let Some(rows) = evidence {
        for row in rows {
            haystack.push('\n');
            haystack.push_str(row.extracted_text.as_str());
        }
    }
    terms
        .iter()
        .filter_map(|term| {
            let score = term_score(term, haystack.as_str());
            (score > 0).then_some((*term, score))
        })
        .max_by_key(|(_, score)| *score)
        .map(|(term, _)| term)
}

fn term_score(term: &MappingTerm, haystack: &str) -> usize {
    if term.label.trim().is_empty() {
        return 0;
    }
    if haystack.contains(term.label.as_str()) {
        return 1000 + term.label.chars().count();
    }
    let chars = term
        .label
        .chars()
        .filter(|character| !character.is_ascii_punctuation() && !character.is_whitespace())
        .collect::<BTreeSet<_>>();
    let matched = chars
        .iter()
        .filter(|character| haystack.contains(**character))
        .count();
    if !chars.is_empty() && matched * 2 >= chars.len() {
        matched
    } else {
        0
    }
}

fn file_label(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(relative_path)
        .to_string()
}
