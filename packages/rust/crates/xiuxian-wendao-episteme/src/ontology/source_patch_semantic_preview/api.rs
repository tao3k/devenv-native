//! Source-patch semantic read-model preview implementation.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    path::Path,
};

use super::types::{
    ACCEPTED_EVIDENCE_STATUS, ACTIVE_STATUS, APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH,
    APPROVED_PROMOTION_DECISION, EpistemeOntologySemanticEvidenceRow,
    EpistemeOntologySemanticObjectRow, EpistemeOntologySemanticProjectionStateRow,
    EpistemeOntologySemanticRelationRow, EpistemeOntologySourcePatchSemanticPreviewReport,
    EpistemeOntologySourcePatchSemanticPreviewRequest, FRESH_STALENESS, INSTANCE_RELATION_KIND,
    OBJECT_INSTANCE_KIND, SEMANTIC_EVIDENCE_JSON, SEMANTIC_EVIDENCE_TSV, SEMANTIC_OBJECTS_JSON,
    SEMANTIC_OBJECTS_TSV, SEMANTIC_PREVIEW_JSON, SEMANTIC_PREVIEW_ORG,
    SEMANTIC_PREVIEW_SCHEMA_VERSION, SEMANTIC_PROJECTION_STATE_JSON, SEMANTIC_RELATIONS_JSON,
    SEMANTIC_RELATIONS_TSV, SOURCE_PATCH_APPLY_PLAN_TSV, SOURCE_PATCH_APPLY_PREVIEW_JSON,
    SOURCE_PATCH_APPLY_PREVIEW_SCHEMA_VERSION, SourcePatchApplyPlanRow,
    SourcePatchApplyPreviewReceipt,
};
use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Compile reviewed source-patch preview artifacts into semantic read-model rows.
///
/// # Errors
///
/// Returns an error when the source-patch apply-plan or preview receipt is
/// missing, unsafe, not admitted, or cannot be projected into a deterministic
/// semantic object/relation/evidence/projection-state read model.
pub fn write_episteme_ontology_source_patch_semantic_preview(
    request: &EpistemeOntologySourcePatchSemanticPreviewRequest,
) -> Result<EpistemeOntologySourcePatchSemanticPreviewReport> {
    write_episteme_ontology_source_patch_semantic_preview_impl(request)
}

fn write_episteme_ontology_source_patch_semantic_preview_impl(
    request: &EpistemeOntologySourcePatchSemanticPreviewRequest,
) -> Result<EpistemeOntologySourcePatchSemanticPreviewReport> {
    let run_dir = request.run_dir();
    let apply_plan_tsv = run_dir.join(SOURCE_PATCH_APPLY_PLAN_TSV);
    let apply_preview_json = run_dir.join(SOURCE_PATCH_APPLY_PREVIEW_JSON);
    let rows = read_apply_plan_rows(apply_plan_tsv.as_path())?;
    let preview = read_apply_preview_receipt(apply_preview_json.as_path())?;
    validate_apply_preview_receipt(&preview, &rows, apply_plan_tsv.as_path())?;
    validate_apply_plan_rows(&rows)?;

    let projection = compile_semantic_projection(&rows)?;
    let quality_issues = projection_quality_issues(&projection);
    if !quality_issues.is_empty() {
        anyhow::bail!(
            "semantic read-model preview quality checks failed: {}",
            quality_issues.join("; ")
        );
    }

    let semantic_objects_tsv = run_dir.join(SEMANTIC_OBJECTS_TSV);
    let semantic_objects_json = run_dir.join(SEMANTIC_OBJECTS_JSON);
    let semantic_relations_tsv = run_dir.join(SEMANTIC_RELATIONS_TSV);
    let semantic_relations_json = run_dir.join(SEMANTIC_RELATIONS_JSON);
    let semantic_evidence_tsv = run_dir.join(SEMANTIC_EVIDENCE_TSV);
    let semantic_evidence_json = run_dir.join(SEMANTIC_EVIDENCE_JSON);
    let semantic_projection_state_json = run_dir.join(SEMANTIC_PROJECTION_STATE_JSON);
    let semantic_read_model_preview_org = run_dir.join(SEMANTIC_PREVIEW_ORG);
    let semantic_read_model_preview_json = run_dir.join(SEMANTIC_PREVIEW_JSON);

    write_objects_tsv(semantic_objects_tsv.as_path(), &projection.objects)?;
    write_json(semantic_objects_json.as_path(), &projection.objects)?;
    write_relations_tsv(semantic_relations_tsv.as_path(), &projection.relations)?;
    write_json(semantic_relations_json.as_path(), &projection.relations)?;
    write_evidence_tsv(semantic_evidence_tsv.as_path(), &projection.evidence)?;
    write_json(semantic_evidence_json.as_path(), &projection.evidence)?;
    write_json(
        semantic_projection_state_json.as_path(),
        &projection.projection_state,
    )?;

    let report = EpistemeOntologySourcePatchSemanticPreviewReport {
        schema_version: SEMANTIC_PREVIEW_SCHEMA_VERSION,
        run_dir: run_dir.to_path_buf(),
        source_patch_apply_plan_tsv: apply_plan_tsv,
        source_patch_apply_preview_json: apply_preview_json,
        semantic_objects_tsv,
        semantic_objects_json,
        semantic_relations_tsv,
        semantic_relations_json,
        semantic_evidence_tsv,
        semantic_evidence_json,
        semantic_projection_state_json,
        semantic_read_model_preview_org,
        semantic_read_model_preview_json,
        apply_plan_row_count: rows.len(),
        semantic_object_count: projection.objects.len(),
        semantic_relation_count: projection.relations.len(),
        semantic_evidence_count: projection.evidence.len(),
        semantic_projection_state_count: projection.projection_state.len(),
        projection_quality_passed: true,
        quality_issues,
        source_mutation_allowed: false,
        ontology_truth: false,
    };
    write_preview_org(report.semantic_read_model_preview_org.as_path(), &report)?;
    write_json(report.semantic_read_model_preview_json.as_path(), &report)?;
    Ok(report)
}

struct SemanticProjection {
    objects: Vec<EpistemeOntologySemanticObjectRow>,
    relations: Vec<EpistemeOntologySemanticRelationRow>,
    evidence: Vec<EpistemeOntologySemanticEvidenceRow>,
    projection_state: Vec<EpistemeOntologySemanticProjectionStateRow>,
}

fn compile_semantic_projection(rows: &[SourcePatchApplyPlanRow]) -> Result<SemanticProjection> {
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
        projection: "source_patch_semantic_read_model_preview".to_string(),
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

fn ontology_target_for(row: &SourcePatchApplyPlanRow) -> Result<String> {
    match row.record_kind.as_str() {
        OBJECT_INSTANCE_KIND => Ok(row.object_type.clone()),
        INSTANCE_RELATION_KIND => Ok(row.predicate.clone()),
        _ => anyhow::bail!(
            "source-patch row `{}` has unsupported record_kind `{}`",
            row.record_id,
            row.record_kind
        ),
    }
}

fn projection_quality_issues(projection: &SemanticProjection) -> Vec<String> {
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

fn read_apply_preview_receipt(path: &Path) -> Result<SourcePatchApplyPreviewReceipt> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse apply-preview JSON `{}`", path.display()))
}

fn validate_apply_preview_receipt(
    preview: &SourcePatchApplyPreviewReceipt,
    rows: &[SourcePatchApplyPlanRow],
    apply_plan_tsv: &Path,
) -> Result<()> {
    if preview.schema_version != SOURCE_PATCH_APPLY_PREVIEW_SCHEMA_VERSION {
        anyhow::bail!(
            "source-patch apply preview has unsupported schemaVersion `{}`",
            preview.schema_version
        );
    }
    if preview.source_mutation_allowed {
        anyhow::bail!("source-patch apply preview attempted to authorize source mutation");
    }
    if preview.ontology_truth {
        anyhow::bail!("source-patch apply preview attempted to mark ontology truth");
    }
    if preview.apply_plan_row_count != rows.len() {
        anyhow::bail!(
            "source-patch apply preview row count mismatch: receipt has {}, TSV has {}",
            preview.apply_plan_row_count,
            rows.len()
        );
    }
    let current_apply_plan_hash = sha256_file(apply_plan_tsv)?;
    if current_apply_plan_hash != preview.apply_plan_tsv_sha256 {
        anyhow::bail!(
            "source-patch apply preview hash mismatch: receipt has {}, current apply-plan TSV has {current_apply_plan_hash}",
            preview.apply_plan_tsv_sha256
        );
    }

    let mut preview_counts_by_target = BTreeMap::<String, usize>::new();
    for target in &preview.preview_targets {
        if !target.proposed_rdf_admission_passed {
            anyhow::bail!(
                "source-patch apply preview target `{}` did not pass proposed RDF admission",
                target.target_rdf_file
            );
        }
        preview_counts_by_target.insert(target.target_rdf_file.clone(), target.preview_row_count);
    }

    let mut row_counts_by_target = BTreeMap::<String, usize>::new();
    for row in rows {
        *row_counts_by_target
            .entry(row.target_rdf_file.clone())
            .or_default() += 1;
    }
    if preview_counts_by_target != row_counts_by_target {
        anyhow::bail!(
            "source-patch apply preview target coverage mismatch: preview has {preview_counts_by_target:?}, TSV has {row_counts_by_target:?}"
        );
    }
    Ok(())
}

fn read_apply_plan_rows(path: &Path) -> Result<Vec<SourcePatchApplyPlanRow>> {
    let file = File::open(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .transpose()
        .with_context(|| format!("failed to read `{}`", path.display()))?
        .with_context(|| format!("source-patch apply-plan TSV `{}` is empty", path.display()))?;
    let columns = header.split('\t').map(str::to_string).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let line = line.with_context(|| format!("failed to read `{}`", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let values = line.split('\t').map(unescape_tsv).collect::<Vec<_>>();
        if values.len() != columns.len() {
            anyhow::bail!(
                "source-patch apply-plan TSV `{}` row {} has {} values for {} columns",
                path.display(),
                line_index + 2,
                values.len(),
                columns.len()
            );
        }
        let row = columns
            .iter()
            .cloned()
            .zip(values)
            .collect::<BTreeMap<_, _>>();
        rows.push(apply_plan_row(path, line_index + 2, &row)?);
    }
    Ok(rows)
}

fn apply_plan_row(
    path: &Path,
    row_number: usize,
    row: &BTreeMap<String, String>,
) -> Result<SourcePatchApplyPlanRow> {
    Ok(SourcePatchApplyPlanRow {
        record_id: required(row, "record_id", path, row_number)?,
        record_kind: required(row, "record_kind", path, row_number)?,
        domain_id: required(row, "domain_id", path, row_number)?,
        target_rdf_file: required(row, "target_rdf_file", path, row_number)?,
        label: optional(row, "label"),
        object_type: optional(row, "object_type"),
        source_object_id: optional(row, "source_object_id"),
        predicate: optional(row, "predicate"),
        target_object_id: optional(row, "target_object_id"),
        evidence_id: required(row, "evidence_id", path, row_number)?,
        review_decision: required(row, "review_decision", path, row_number)?,
        promotion_decision: required(row, "promotion_decision", path, row_number)?,
        reviewer_id: required(row, "reviewer_id", path, row_number)?,
        apply_action: required(row, "apply_action", path, row_number)?,
        source_mutation_allowed: parse_bool(row, "source_mutation_allowed", path, row_number)?,
        ontology_truth: parse_bool(row, "ontology_truth", path, row_number)?,
    })
}

fn validate_apply_plan_rows(rows: &[SourcePatchApplyPlanRow]) -> Result<()> {
    let mut object_ids = BTreeSet::new();
    let mut relation_ids = BTreeSet::new();
    for row in rows {
        if row.apply_action != APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH {
            anyhow::bail!(
                "source-patch row `{}` has unsupported apply_action `{}`",
                row.record_id,
                row.apply_action
            );
        }
        if normalize(row.promotion_decision.as_str()) != APPROVED_PROMOTION_DECISION {
            anyhow::bail!(
                "source-patch row `{}` is not explicitly approved",
                row.record_id
            );
        }
        if row.source_mutation_allowed {
            anyhow::bail!(
                "source-patch row `{}` attempted to authorize source mutation",
                row.record_id
            );
        }
        if row.ontology_truth {
            anyhow::bail!(
                "source-patch row `{}` attempted to mark ontology truth",
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
                        "semantic preview contains duplicate object record `{}`",
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
                        "semantic preview contains duplicate relation record `{}`",
                        row.record_id
                    );
                }
            }
            _ => anyhow::bail!(
                "source-patch row `{}` has unsupported record_kind `{}`",
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

fn write_objects_tsv(path: &Path, rows: &[EpistemeOntologySemanticObjectRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "id\tkind\ttitle\tdomain\tevidence_id\tevidence_status\ttarget_rdf_file\treview_decision\tpromotion_decision\treviewer_id\trelation_count\tstatus\tread_model_projection_staleness"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(row.id.as_str()),
            escape_tsv(row.kind.as_str()),
            escape_tsv(row.title.as_str()),
            escape_tsv(row.domain.as_str()),
            escape_tsv(row.evidence_id.as_str()),
            row.evidence_status,
            escape_tsv(row.target_rdf_file.as_str()),
            escape_tsv(row.review_decision.as_str()),
            escape_tsv(row.promotion_decision.as_str()),
            escape_tsv(row.reviewer_id.as_str()),
            row.relation_count,
            row.status,
            row.read_model_projection_staleness
        )?;
    }
    Ok(())
}

fn write_relations_tsv(path: &Path, rows: &[EpistemeOntologySemanticRelationRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "id\tkind\tsource\ttarget\tdomain\tevidence_id\tevidence_status\ttarget_rdf_file\treview_decision\tpromotion_decision\treviewer_id\tstatus\tread_model_projection_staleness"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(row.id.as_str()),
            escape_tsv(row.kind.as_str()),
            escape_tsv(row.source.as_str()),
            escape_tsv(row.target.as_str()),
            escape_tsv(row.domain.as_str()),
            escape_tsv(row.evidence_id.as_str()),
            row.evidence_status,
            escape_tsv(row.target_rdf_file.as_str()),
            escape_tsv(row.review_decision.as_str()),
            escape_tsv(row.promotion_decision.as_str()),
            escape_tsv(row.reviewer_id.as_str()),
            row.status,
            row.read_model_projection_staleness
        )?;
    }
    Ok(())
}

fn write_evidence_tsv(path: &Path, rows: &[EpistemeOntologySemanticEvidenceRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "id\tevidence_id\trecord_id\trecord_kind\tontology_target\ttarget\tstatus\tdomain\ttarget_rdf_file\treviewer_id"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(row.id.as_str()),
            escape_tsv(row.evidence_id.as_str()),
            escape_tsv(row.record_id.as_str()),
            escape_tsv(row.record_kind.as_str()),
            escape_tsv(row.ontology_target.as_str()),
            escape_tsv(row.target.as_str()),
            row.status,
            escape_tsv(row.domain.as_str()),
            escape_tsv(row.target_rdf_file.as_str()),
            escape_tsv(row.reviewer_id.as_str())
        )?;
    }
    Ok(())
}

fn write_preview_org(
    path: &Path,
    report: &EpistemeOntologySourcePatchSemanticPreviewReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Ontology Semantic Read-Model Preview")?;
    writeln!(file)?;
    writeln!(file, "* Semantic read-model preview")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_source_patch_semantic_preview")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This preview compiles admitted source-patch rows into graph-ready semantic read-model artifacts without mutating ontology source."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| apply_plan_row_count | {} |",
        report.apply_plan_row_count
    )?;
    writeln!(
        file,
        "| semantic_object_count | {} |",
        report.semantic_object_count
    )?;
    writeln!(
        file,
        "| semantic_relation_count | {} |",
        report.semantic_relation_count
    )?;
    writeln!(
        file,
        "| semantic_evidence_count | {} |",
        report.semantic_evidence_count
    )?;
    writeln!(
        file,
        "| semantic_projection_state_count | {} |",
        report.semantic_projection_state_count
    )?;
    writeln!(
        file,
        "| projection_quality_passed | {} |",
        report.projection_quality_passed
    )?;
    writeln!(file, "| source_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
    writeln!(file)?;
    writeln!(file, "** Artifact paths")?;
    writeln!(file, "| artifact | path |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| semantic_objects_tsv | {} |",
        report.semantic_objects_tsv.display()
    )?;
    writeln!(
        file,
        "| semantic_relations_tsv | {} |",
        report.semantic_relations_tsv.display()
    )?;
    writeln!(
        file,
        "| semantic_evidence_tsv | {} |",
        report.semantic_evidence_tsv.display()
    )?;
    writeln!(
        file,
        "| semantic_projection_state_json | {} |",
        report.semantic_projection_state_json.display()
    )?;
    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read `{}`", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn required(
    row: &BTreeMap<String, String>,
    name: &str,
    path: &Path,
    row_number: usize,
) -> Result<String> {
    row.get(name).cloned().with_context(|| {
        format!(
            "source-patch apply-plan TSV `{}` row {row_number} missing `{name}` column",
            path.display()
        )
    })
}

fn optional(row: &BTreeMap<String, String>, name: &str) -> String {
    row.get(name).cloned().unwrap_or_default()
}

fn parse_bool(
    row: &BTreeMap<String, String>,
    name: &str,
    path: &Path,
    row_number: usize,
) -> Result<bool> {
    let value = required(row, name, path, row_number)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!(
            "source-patch apply-plan TSV `{}` row {row_number} has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
}

fn require_nonblank(value: &str, record_id: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("source-patch row `{record_id}` must declare nonblank {field}");
    }
    Ok(())
}

fn normalize(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_lowercase()
}

fn unescape_tsv(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('t') => output.push('\t'),
                Some('n') => output.push('\n'),
                Some('r') => output.push('\r'),
                Some('\\') | None => output.push('\\'),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
