use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use super::types::{
    ACCEPTED_EVIDENCE_STATUS, ACTIVE_STATUS, APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH,
    APPROVED_PROMOTION_DECISION, FRESH_STALENESS, INSTANCE_RELATION_KIND, OBJECT_INSTANCE_KIND,
    SourcePatchRdfRow,
};
use crate::ontology::{
    EpistemeOntologySemanticEvidenceRow, EpistemeOntologySemanticObjectRow,
    EpistemeOntologySemanticProjectionStateRow, EpistemeOntologySemanticRelationRow,
};

pub(super) struct SemanticProjection {
    pub(super) objects: Vec<EpistemeOntologySemanticObjectRow>,
    pub(super) relations: Vec<EpistemeOntologySemanticRelationRow>,
    pub(super) evidence: Vec<EpistemeOntologySemanticEvidenceRow>,
    pub(super) projection_state: Vec<EpistemeOntologySemanticProjectionStateRow>,
}

pub(super) fn compile_semantic_projection(
    rows: &[SourcePatchRdfRow],
) -> Result<SemanticProjection> {
    validate_source_rows(rows)?;

    let mut relation_count_by_object = BTreeMap::<String, usize>::new();
    for row in rows
        .iter()
        .filter(|row| row.record_kind == INSTANCE_RELATION_KIND)
    {
        *relation_count_by_object
            .entry(row.source_object_id.clone())
            .or_default() += 1;
        *relation_count_by_object
            .entry(row.target_object_id.clone())
            .or_default() += 1;
    }

    let objects = rows
        .iter()
        .filter(|row| row.record_kind == OBJECT_INSTANCE_KIND)
        .map(|row| EpistemeOntologySemanticObjectRow {
            id: row.record_id.clone(),
            kind: row.object_type.clone(),
            title: row.label.clone(),
            domain: row.domain_id.clone(),
            evidence_id: row.evidence_id.clone(),
            evidence_status: ACCEPTED_EVIDENCE_STATUS,
            target_rdf_file: row.target_rdf_file.clone(),
            review_decision: row.review_decision.clone(),
            promotion_decision: row.promotion_decision.clone(),
            reviewer_id: row.reviewer_id.clone(),
            relation_count: *relation_count_by_object
                .get(row.record_id.as_str())
                .unwrap_or(&0),
            status: ACTIVE_STATUS,
            read_model_projection_staleness: FRESH_STALENESS,
        })
        .collect::<Vec<_>>();

    let relations = rows
        .iter()
        .filter(|row| row.record_kind == INSTANCE_RELATION_KIND)
        .map(|row| EpistemeOntologySemanticRelationRow {
            id: row.record_id.clone(),
            kind: row.predicate.clone(),
            source: row.source_object_id.clone(),
            target: row.target_object_id.clone(),
            domain: row.domain_id.clone(),
            evidence_id: row.evidence_id.clone(),
            evidence_status: ACCEPTED_EVIDENCE_STATUS,
            target_rdf_file: row.target_rdf_file.clone(),
            review_decision: row.review_decision.clone(),
            promotion_decision: row.promotion_decision.clone(),
            reviewer_id: row.reviewer_id.clone(),
            status: ACTIVE_STATUS,
            read_model_projection_staleness: FRESH_STALENESS,
        })
        .collect::<Vec<_>>();

    let evidence = rows
        .iter()
        .map(|row| {
            let ontology_target = ontology_target_for(row)?;
            Ok(EpistemeOntologySemanticEvidenceRow {
                id: format!("{}#evidence", row.record_id),
                evidence_id: row.evidence_id.clone(),
                record_id: row.record_id.clone(),
                record_kind: row.record_kind.clone(),
                ontology_target: ontology_target.clone(),
                target: ontology_target,
                status: ACCEPTED_EVIDENCE_STATUS,
                domain: row.domain_id.clone(),
                target_rdf_file: row.target_rdf_file.clone(),
                reviewer_id: row.reviewer_id.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let projection_state = vec![EpistemeOntologySemanticProjectionStateRow {
        projection: "source_patch_rdf_read_model".to_string(),
        status: ACTIVE_STATUS,
        staleness: FRESH_STALENESS,
        source_object_count: objects.len(),
        source_relation_count: relations.len(),
        source_evidence_count: evidence.len(),
    }];

    Ok(SemanticProjection {
        objects,
        relations,
        evidence,
        projection_state,
    })
}

pub(super) fn projection_quality_issues(projection: &SemanticProjection) -> Vec<String> {
    let mut issues = Vec::new();
    let mut object_ids = BTreeSet::new();
    for object in &projection.objects {
        if object.id.trim().is_empty() {
            issues.push("semantic object id is blank".to_string());
        }
        if !object_ids.insert(object.id.as_str()) {
            issues.push(format!("semantic object id `{}` is duplicated", object.id));
        }
        if object.kind.trim().is_empty() {
            issues.push(format!("semantic object `{}` kind is blank", object.id));
        }
        if object.evidence_id.trim().is_empty() {
            issues.push(format!(
                "semantic object `{}` evidence_id is blank",
                object.id
            ));
        }
    }

    let known_object_ids = object_ids;
    let mut relation_ids = BTreeSet::new();
    for relation in &projection.relations {
        if relation.id.trim().is_empty() {
            issues.push("semantic relation id is blank".to_string());
        }
        if !relation_ids.insert(relation.id.as_str()) {
            issues.push(format!(
                "semantic relation id `{}` is duplicated",
                relation.id
            ));
        }
        if !known_object_ids.contains(relation.source.as_str()) {
            issues.push(format!(
                "semantic relation `{}` source `{}` is missing",
                relation.id, relation.source
            ));
        }
        if !known_object_ids.contains(relation.target.as_str()) {
            issues.push(format!(
                "semantic relation `{}` target `{}` is missing",
                relation.id, relation.target
            ));
        }
        if relation.kind.trim().is_empty() {
            issues.push(format!("semantic relation `{}` kind is blank", relation.id));
        }
        if relation.evidence_id.trim().is_empty() {
            issues.push(format!(
                "semantic relation `{}` evidence_id is blank",
                relation.id
            ));
        }
    }

    if !projection.relations.is_empty() && projection.objects.is_empty() {
        issues.push("semantic projection has relations but no objects".to_string());
    }
    if !projection.objects.is_empty() && projection.projection_state.is_empty() {
        issues.push("semantic projection state is empty for nonempty objects".to_string());
    }
    issues
}

fn validate_source_rows(rows: &[SourcePatchRdfRow]) -> Result<()> {
    let mut object_ids = BTreeSet::new();
    let mut relation_ids = BTreeSet::new();
    for row in rows {
        if row.apply_action != APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH {
            anyhow::bail!(
                "source-patch RDF row `{}` has unsupported apply_action `{}`",
                row.record_id,
                row.apply_action
            );
        }
        if normalize(row.promotion_decision.as_str()) != APPROVED_PROMOTION_DECISION {
            anyhow::bail!(
                "source-patch RDF row `{}` is not explicitly approved",
                row.record_id
            );
        }
        if row.source_mutation_allowed {
            anyhow::bail!(
                "source-patch RDF row `{}` attempted to authorize source mutation",
                row.record_id
            );
        }
        if row.ontology_truth {
            anyhow::bail!(
                "source-patch RDF row `{}` attempted to mark ontology truth",
                row.record_id
            );
        }
        require_nonblank(row.domain_id.as_str(), row.record_id.as_str(), "domain_id")?;
        require_nonblank(
            row.target_rdf_file.as_str(),
            row.record_id.as_str(),
            "target_rdf_file",
        )?;
        require_nonblank(
            row.evidence_id.as_str(),
            row.record_id.as_str(),
            "evidence_id",
        )?;
        match row.record_kind.as_str() {
            OBJECT_INSTANCE_KIND => {
                require_nonblank(
                    row.object_type.as_str(),
                    row.record_id.as_str(),
                    "object_type",
                )?;
                require_nonblank(row.label.as_str(), row.record_id.as_str(), "label")?;
                if !object_ids.insert(row.record_id.as_str()) {
                    anyhow::bail!(
                        "RDF source contains duplicate object record `{}`",
                        row.record_id
                    );
                }
            }
            INSTANCE_RELATION_KIND => {
                require_nonblank(
                    row.source_object_id.as_str(),
                    row.record_id.as_str(),
                    "source_object_id",
                )?;
                require_nonblank(row.predicate.as_str(), row.record_id.as_str(), "predicate")?;
                require_nonblank(
                    row.target_object_id.as_str(),
                    row.record_id.as_str(),
                    "target_object_id",
                )?;
                if !relation_ids.insert(row.record_id.as_str()) {
                    anyhow::bail!(
                        "RDF source contains duplicate relation record `{}`",
                        row.record_id
                    );
                }
            }
            _ => anyhow::bail!(
                "source-patch RDF row `{}` has unsupported record_kind `{}`",
                row.record_id,
                row.record_kind
            ),
        }
    }
    for row in rows
        .iter()
        .filter(|row| row.record_kind == INSTANCE_RELATION_KIND)
    {
        if !object_ids.contains(row.source_object_id.as_str()) {
            anyhow::bail!(
                "semantic relation `{}` references source `{}` without a compiled object row",
                row.record_id,
                row.source_object_id
            );
        }
        if !object_ids.contains(row.target_object_id.as_str()) {
            anyhow::bail!(
                "semantic relation `{}` references target `{}` without a compiled object row",
                row.record_id,
                row.target_object_id
            );
        }
    }
    Ok(())
}

fn ontology_target_for(row: &SourcePatchRdfRow) -> Result<String> {
    match row.record_kind.as_str() {
        OBJECT_INSTANCE_KIND => Ok(row.object_type.clone()),
        INSTANCE_RELATION_KIND => Ok(row.predicate.clone()),
        _ => anyhow::bail!(
            "source-patch RDF row `{}` has unsupported record_kind `{}`",
            row.record_id,
            row.record_kind
        ),
    }
}

fn require_nonblank(value: &str, record_id: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("source-patch RDF row `{record_id}` must declare nonblank {field}");
    }
    Ok(())
}

fn normalize(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_lowercase()
}
