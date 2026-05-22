use std::fmt::Write as _;

use anyhow::Result;
use sha2::{Digest, Sha256};

use super::{
    model::{
        CandidateEvidenceRecord, CandidateObjectRecord, CandidateRelationRecord, DraftInputs,
        DraftRender, ONTOLOGY_TRUTH, PROPOSAL_STATUS, RAW_TO_RDF_PROMOTION_ALLOWED,
        RenderedResource, ReviewRecord,
    },
    validation::require_review,
};

const RDF_PREFIXES: &str = concat!(
    "@prefix draft: <urn:wendao:episteme:rdf-draft#> .\n",
    "@prefix wdp: <https://wendao.ai/ontology/proposal/> .\n",
    "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n",
    "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n"
);

pub(super) fn render_rdf_draft(inputs: &DraftInputs) -> Result<DraftRender> {
    let resources = render_all_resources(inputs)?;
    let statement_count = resources
        .iter()
        .map(|resource| resource.statement_count)
        .sum();
    let resource_count = resources.len();
    let ttl = resources
        .into_iter()
        .fold(RDF_PREFIXES.to_string(), |mut output, resource| {
            output.push_str(resource.text.as_str());
            output
        });
    Ok(DraftRender {
        ttl,
        resource_count,
        statement_count,
    })
}

fn render_all_resources(inputs: &DraftInputs) -> Result<Vec<RenderedResource>> {
    let mut resources = Vec::new();
    resources.extend(render_object_resources(inputs)?);
    resources.extend(render_relation_resources(inputs)?);
    resources.extend(render_evidence_resources(inputs)?);
    Ok(resources)
}

fn render_object_resources(inputs: &DraftInputs) -> Result<Vec<RenderedResource>> {
    inputs
        .objects
        .iter()
        .map(|object| {
            let review = require_review(inputs, object.candidate_id.as_str())?;
            Ok(render_object_resource(object, review))
        })
        .collect()
}

fn render_relation_resources(inputs: &DraftInputs) -> Result<Vec<RenderedResource>> {
    inputs
        .relations
        .iter()
        .map(|relation| {
            let review = require_review(inputs, relation.candidate_id.as_str())?;
            Ok(render_relation_resource(relation, review))
        })
        .collect()
}

fn render_evidence_resources(inputs: &DraftInputs) -> Result<Vec<RenderedResource>> {
    inputs
        .evidence
        .iter()
        .map(|evidence| {
            let review = require_review(inputs, evidence.evidence_id.as_str())?;
            Ok(render_evidence_resource(evidence, review))
        })
        .collect()
}

fn render_object_resource(
    object: &CandidateObjectRecord,
    review: &ReviewRecord,
) -> RenderedResource {
    let mut statements = vec![
        ("a", "wdp:OntologyCandidate".to_string()),
        (
            "wdp:candidateId",
            quoted_literal(object.candidate_id.as_str()),
        ),
        (
            "wdp:recordKind",
            quoted_literal(object.candidate_kind.as_str()),
        ),
        ("rdfs:label", quoted_literal(object.label.as_str())),
    ];
    push_literal(
        &mut statements,
        "wdp:suggestedTermKey",
        object.suggested_term_key.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:sourceFileId",
        object.source_file_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:sourceQueueId",
        object.source_queue_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:sourcePath",
        object.source_path.as_str(),
    );
    push_literal(&mut statements, "wdp:category", object.category.as_str());
    push_literal(&mut statements, "wdp:language", object.language.as_str());
    push_literal(
        &mut statements,
        "wdp:extractionRoute",
        object.extraction_route.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:extractionRunId",
        object.extraction_run_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:evidenceSha256",
        object.evidence_sha256.as_str(),
    );
    append_review_statements(&mut statements, review);
    append_proposal_statements(&mut statements);
    render_resource(
        subject_for(object.candidate_id.as_str()).as_str(),
        &statements,
    )
}

fn render_relation_resource(
    relation: &CandidateRelationRecord,
    review: &ReviewRecord,
) -> RenderedResource {
    let mut statements = vec![
        ("a", "wdp:OntologyCandidateRelation".to_string()),
        (
            "wdp:candidateId",
            quoted_literal(relation.candidate_id.as_str()),
        ),
        (
            "wdp:recordKind",
            quoted_literal(relation.relation_kind.as_str()),
        ),
        (
            "wdp:sourceCandidateId",
            quoted_literal(relation.source_candidate_id.as_str()),
        ),
        (
            "wdp:targetCandidateId",
            quoted_literal(relation.target_candidate_id.as_str()),
        ),
    ];
    push_literal(
        &mut statements,
        "wdp:sourceFileId",
        relation.source_file_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:sourceQueueId",
        relation.source_queue_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:extractionRunId",
        relation.extraction_run_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:evidenceSha256",
        relation.evidence_sha256.as_str(),
    );
    append_review_statements(&mut statements, review);
    append_proposal_statements(&mut statements);
    render_resource(
        subject_for(relation.candidate_id.as_str()).as_str(),
        &statements,
    )
}

fn render_evidence_resource(
    evidence: &CandidateEvidenceRecord,
    review: &ReviewRecord,
) -> RenderedResource {
    let mut statements = Vec::new();
    statements.push(("a", "wdp:OntologyCandidateEvidence".to_string()));
    statements.push((
        "wdp:evidenceId",
        quoted_literal(evidence.evidence_id.as_str()),
    ));
    statements.push((
        "wdp:recordKind",
        quoted_literal(evidence.evidence_kind.as_str()),
    ));
    push_literal(
        &mut statements,
        "wdp:sourceFileId",
        evidence.source_file_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:sourceQueueId",
        evidence.source_queue_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:extractionRunId",
        evidence.extraction_run_id.as_str(),
    );
    push_literal(
        &mut statements,
        "wdp:evidenceSha256",
        evidence.evidence_sha256.as_str(),
    );
    statements.push((
        "wdp:textCharCount",
        typed_integer_literal(evidence.text_char_count),
    ));
    append_review_statements(&mut statements, review);
    append_proposal_statements(&mut statements);
    render_resource(
        subject_for(evidence.evidence_id.as_str()).as_str(),
        &statements,
    )
}

fn append_review_statements(statements: &mut Vec<(&'static str, String)>, review: &ReviewRecord) {
    statements.push((
        "wdp:reviewRecordKind",
        quoted_literal(review.record_kind.as_str()),
    ));
    statements.push((
        "wdp:reviewDecision",
        quoted_literal(review.review_decision.as_str()),
    ));
    statements.push((
        "wdp:qualityScore",
        typed_integer_literal(review.quality_score),
    ));
    statements.push((
        "wdp:evidenceStrength",
        quoted_literal(review.evidence_strength.as_str()),
    ));
    push_literal(statements, "wdp:issueCodes", review.issue_codes.as_str());
    statements.push((
        "wdp:promotionPreconditionMet",
        typed_bool_literal(review.promotion_precondition_met),
    ));
    push_literal(
        statements,
        "wdp:reviewSuggestedTermKey",
        review.suggested_term_key.as_str(),
    );
    push_literal(statements, "wdp:reviewLabel", review.label.as_str());
}

fn append_proposal_statements(statements: &mut Vec<(&'static str, String)>) {
    statements.push(("wdp:proposalStatus", quoted_literal(PROPOSAL_STATUS)));
    statements.push((
        "wdp:rawToRdfPromotionAllowed",
        typed_bool_literal(RAW_TO_RDF_PROMOTION_ALLOWED),
    ));
    statements.push(("wdp:ontologyTruth", typed_bool_literal(ONTOLOGY_TRUTH)));
}

fn push_literal(
    statements: &mut Vec<(&'static str, String)>,
    predicate: &'static str,
    value: &str,
) {
    if !value.trim().is_empty() {
        statements.push((predicate, quoted_literal(value)));
    }
}

fn render_resource(subject: &str, statements: &[(&str, String)]) -> RenderedResource {
    let mut text = String::new();
    text.push_str(subject);
    text.push('\n');
    for (index, (predicate, object)) in statements.iter().enumerate() {
        let terminator = if index + 1 == statements.len() {
            ".\n\n"
        } else {
            " ;\n"
        };
        text.push_str("  ");
        text.push_str(predicate);
        text.push(' ');
        text.push_str(object);
        text.push_str(terminator);
    }
    RenderedResource {
        text,
        statement_count: statements.len(),
    }
}

fn subject_for(record_id: &str) -> String {
    format!("draft:{}", hash_id(record_id))
}

fn hash_id(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    digest[..12].iter().fold(String::new(), |mut hash, byte| {
        let _ = write!(hash, "{byte:02x}");
        hash
    })
}

fn quoted_literal(value: &str) -> String {
    format!("\"{}\"", escape_literal(value))
}

fn typed_integer_literal(value: usize) -> String {
    format!("\"{value}\"^^xsd:integer")
}

fn typed_bool_literal(value: bool) -> String {
    format!("\"{value}\"^^xsd:boolean")
}

fn escape_literal(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
