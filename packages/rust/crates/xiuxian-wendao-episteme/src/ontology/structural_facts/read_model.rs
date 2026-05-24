//! Structural facts read-model rows for downstream graph/search consumers.

use std::{collections::BTreeSet, fmt::Write as FmtWrite, fs::File, path::Path, sync::Arc};

use anyhow::{Context, Result};
use arrow::{
    array::{ArrayRef, BooleanArray, StringArray},
    datatypes::{DataType, Field, Schema, SchemaRef},
    record_batch::RecordBatch,
};
use parquet::arrow::ArrowWriter;
use serde::Serialize;

use super::{
    ids::stable_id,
    types::{EpistemeOntologyStructuralFactsSnapshot, StructuralFactsOutputPaths},
    write::{write_json, write_string},
};

const ACTIVE_STATUS: &str = "active";
const FRESH_STALENESS: &str = "fresh";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StructuralFactsReadModel {
    pub(super) objects: Vec<StructuralFactsReadModelObjectRow>,
    pub(super) relations: Vec<StructuralFactsReadModelRelationRow>,
    pub(super) projection_state: Vec<StructuralFactsProjectionStateRow>,
    pub(super) quality_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StructuralFactsReadModelObjectRow {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) title: String,
    pub(super) read_model_projection_staleness: String,
    pub(super) domain_id: String,
    pub(super) source_contract_id: String,
    pub(super) document_id: String,
    pub(super) file_id: String,
    pub(super) relative_path: String,
    pub(super) source_content_hash: String,
    pub(super) ontology_truth: bool,
    pub(super) status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StructuralFactsReadModelRelationRow {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) source: String,
    pub(super) target: String,
    pub(super) read_model_projection_staleness: String,
    pub(super) domain_id: String,
    pub(super) source_contract_id: String,
    pub(super) evidence_path: String,
    pub(super) ontology_truth: bool,
    pub(super) status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StructuralFactsProjectionStateRow {
    pub(super) projection: String,
    pub(super) status: String,
    pub(super) staleness: String,
    pub(super) source_object_count: usize,
    pub(super) source_relation_count: usize,
    pub(super) source_document_count: usize,
    pub(super) source_anchor_count: usize,
}

pub(super) fn compile_structural_facts_read_model(
    snapshot: &EpistemeOntologyStructuralFactsSnapshot,
) -> StructuralFactsReadModel {
    let mut objects = snapshot
        .documents
        .iter()
        .map(|document| StructuralFactsReadModelObjectRow {
            id: document.document_id.clone(),
            kind: "source_document".to_string(),
            title: document.relative_path.clone(),
            read_model_projection_staleness: FRESH_STALENESS.to_string(),
            domain_id: document.domain_id.clone(),
            source_contract_id: document.source_contract_id.clone(),
            document_id: document.document_id.clone(),
            file_id: document.file_id.clone(),
            relative_path: document.relative_path.clone(),
            source_content_hash: document.sha256.clone(),
            ontology_truth: false,
            status: ACTIVE_STATUS.to_string(),
        })
        .collect::<Vec<_>>();

    objects.extend(snapshot.anchors.iter().map(|anchor| {
        let title = if anchor.relative_path.is_empty() {
            anchor.source_contract_id.clone()
        } else {
            anchor.relative_path.clone()
        };
        StructuralFactsReadModelObjectRow {
            id: anchor.anchor_id.clone(),
            kind: anchor.anchor_kind.clone(),
            title,
            read_model_projection_staleness: FRESH_STALENESS.to_string(),
            domain_id: anchor.domain_id.clone(),
            source_contract_id: anchor.source_contract_id.clone(),
            document_id: anchor.document_id.clone(),
            file_id: anchor.file_id.clone(),
            relative_path: anchor.relative_path.clone(),
            source_content_hash: anchor.source_content_hash.clone(),
            ontology_truth: false,
            status: ACTIVE_STATUS.to_string(),
        }
    }));

    let mut relations = snapshot
        .relations
        .iter()
        .map(|relation| StructuralFactsReadModelRelationRow {
            id: relation.relation_id.clone(),
            kind: relation.relation_kind.clone(),
            source: relation.source_anchor_id.clone(),
            target: relation.target_anchor_id.clone(),
            read_model_projection_staleness: FRESH_STALENESS.to_string(),
            domain_id: relation.domain_id.clone(),
            source_contract_id: relation.source_contract_id.clone(),
            evidence_path: relation.evidence_path.clone(),
            ontology_truth: false,
            status: ACTIVE_STATUS.to_string(),
        })
        .collect::<Vec<_>>();

    relations.extend(
        snapshot
            .anchors
            .iter()
            .filter(|anchor| {
                anchor.anchor_kind == "document_root" && !anchor.document_id.is_empty()
            })
            .map(|anchor| StructuralFactsReadModelRelationRow {
                id: stable_id(
                    "structural_facts.read_model_relation",
                    &format!("{}:{}", anchor.document_id, anchor.anchor_id),
                ),
                kind: "has_document_root".to_string(),
                source: anchor.document_id.clone(),
                target: anchor.anchor_id.clone(),
                read_model_projection_staleness: FRESH_STALENESS.to_string(),
                domain_id: anchor.domain_id.clone(),
                source_contract_id: anchor.source_contract_id.clone(),
                evidence_path: anchor.relative_path.clone(),
                ontology_truth: false,
                status: ACTIVE_STATUS.to_string(),
            }),
    );

    let projection_state = vec![StructuralFactsProjectionStateRow {
        projection: "structural_facts_seed_read_model".to_string(),
        status: ACTIVE_STATUS.to_string(),
        staleness: FRESH_STALENESS.to_string(),
        source_object_count: objects.len(),
        source_relation_count: relations.len(),
        source_document_count: snapshot.documents.len(),
        source_anchor_count: snapshot.anchors.len(),
    }];

    let mut read_model = StructuralFactsReadModel {
        objects,
        relations,
        projection_state,
        quality_issues: Vec::new(),
    };
    read_model.quality_issues = quality_issues(&read_model);
    read_model
}

pub(super) fn write_structural_facts_read_model(
    paths: &StructuralFactsOutputPaths,
    read_model: &StructuralFactsReadModel,
) -> Result<()> {
    write_objects_tsv(paths.read_model_objects_tsv.as_path(), &read_model.objects)?;
    write_json(paths.read_model_objects_json.as_path(), &read_model.objects)?;
    write_relations_tsv(
        paths.read_model_relations_tsv.as_path(),
        &read_model.relations,
    )?;
    write_json(
        paths.read_model_relations_json.as_path(),
        &read_model.relations,
    )?;
    write_json(
        paths.read_model_projection_state_json.as_path(),
        &read_model.projection_state,
    )?;
    write_parquet(
        paths.read_model_objects_parquet.as_path(),
        &object_batch(&read_model.objects)?,
    )?;
    write_parquet(
        paths.read_model_relations_parquet.as_path(),
        &relation_batch(&read_model.relations)?,
    )?;
    Ok(())
}

fn quality_issues(read_model: &StructuralFactsReadModel) -> Vec<String> {
    let mut issues = Vec::new();
    let mut object_ids = BTreeSet::new();
    for object in &read_model.objects {
        if object.id.trim().is_empty() {
            issues.push("structural read-model object id is blank".to_string());
        }
        if !object_ids.insert(object.id.as_str()) {
            issues.push(format!(
                "structural read-model object id `{}` is duplicated",
                object.id
            ));
        }
        if object.domain_id.trim().is_empty() {
            issues.push(format!(
                "structural read-model object `{}` domain_id is blank",
                object.id
            ));
        }
        if object.ontology_truth {
            issues.push(format!(
                "structural read-model object `{}` attempted to mark ontology truth",
                object.id
            ));
        }
    }

    let mut relation_ids = BTreeSet::new();
    for relation in &read_model.relations {
        if relation.id.trim().is_empty() {
            issues.push("structural read-model relation id is blank".to_string());
        }
        if !relation_ids.insert(relation.id.as_str()) {
            issues.push(format!(
                "structural read-model relation id `{}` is duplicated",
                relation.id
            ));
        }
        if !object_ids.contains(relation.source.as_str()) {
            issues.push(format!(
                "structural read-model relation `{}` source `{}` is missing",
                relation.id, relation.source
            ));
        }
        if !object_ids.contains(relation.target.as_str()) {
            issues.push(format!(
                "structural read-model relation `{}` target `{}` is missing",
                relation.id, relation.target
            ));
        }
        if relation.ontology_truth {
            issues.push(format!(
                "structural read-model relation `{}` attempted to mark ontology truth",
                relation.id
            ));
        }
    }

    if read_model.objects.is_empty() {
        issues.push("structural read-model has no objects".to_string());
    }
    if !read_model.objects.is_empty() && read_model.projection_state.is_empty() {
        issues.push("structural read-model projection state is empty".to_string());
    }
    issues
}

fn write_objects_tsv(path: &Path, rows: &[StructuralFactsReadModelObjectRow]) -> Result<()> {
    let mut lines = String::from(
        "id\tkind\ttitle\tread_model_projection_staleness\tdomain_id\tsource_contract_id\tdocument_id\tfile_id\trelative_path\tsource_content_hash\tontology_truth\tstatus\n",
    );
    for row in rows {
        writeln!(
            lines,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.id),
            escape_tsv(&row.kind),
            escape_tsv(&row.title),
            escape_tsv(&row.read_model_projection_staleness),
            escape_tsv(&row.domain_id),
            escape_tsv(&row.source_contract_id),
            escape_tsv(&row.document_id),
            escape_tsv(&row.file_id),
            escape_tsv(&row.relative_path),
            escape_tsv(&row.source_content_hash),
            row.ontology_truth,
            escape_tsv(&row.status)
        )
        .context("failed to render structural facts object TSV row")?;
    }
    write_string(path, &lines)
}

fn write_relations_tsv(path: &Path, rows: &[StructuralFactsReadModelRelationRow]) -> Result<()> {
    let mut lines = String::from(
        "id\tkind\tsource\ttarget\tread_model_projection_staleness\tdomain_id\tsource_contract_id\tevidence_path\tontology_truth\tstatus\n",
    );
    for row in rows {
        writeln!(
            lines,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.id),
            escape_tsv(&row.kind),
            escape_tsv(&row.source),
            escape_tsv(&row.target),
            escape_tsv(&row.read_model_projection_staleness),
            escape_tsv(&row.domain_id),
            escape_tsv(&row.source_contract_id),
            escape_tsv(&row.evidence_path),
            row.ontology_truth,
            escape_tsv(&row.status)
        )
        .context("failed to render structural facts relation TSV row")?;
    }
    write_string(path, &lines)
}

fn object_batch(rows: &[StructuralFactsReadModelObjectRow]) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema_ref([
            string_field("id"),
            string_field("kind"),
            string_field("title"),
            string_field("read_model_projection_staleness"),
            string_field("domain_id"),
            string_field("source_contract_id"),
            string_field("document_id"),
            string_field("file_id"),
            string_field("relative_path"),
            string_field("source_content_hash"),
            bool_field("ontology_truth"),
            string_field("status"),
        ]),
        vec![
            strings(rows.iter().map(|row| row.id.as_str())),
            strings(rows.iter().map(|row| row.kind.as_str())),
            strings(rows.iter().map(|row| row.title.as_str())),
            strings(
                rows.iter()
                    .map(|row| row.read_model_projection_staleness.as_str()),
            ),
            strings(rows.iter().map(|row| row.domain_id.as_str())),
            strings(rows.iter().map(|row| row.source_contract_id.as_str())),
            strings(rows.iter().map(|row| row.document_id.as_str())),
            strings(rows.iter().map(|row| row.file_id.as_str())),
            strings(rows.iter().map(|row| row.relative_path.as_str())),
            strings(rows.iter().map(|row| row.source_content_hash.as_str())),
            booleans(rows.iter().map(|row| row.ontology_truth)),
            strings(rows.iter().map(|row| row.status.as_str())),
        ],
    )
    .context("failed to build structural facts object read-model")
}

fn relation_batch(rows: &[StructuralFactsReadModelRelationRow]) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema_ref([
            string_field("id"),
            string_field("kind"),
            string_field("source"),
            string_field("target"),
            string_field("read_model_projection_staleness"),
            string_field("domain_id"),
            string_field("source_contract_id"),
            string_field("evidence_path"),
            bool_field("ontology_truth"),
            string_field("status"),
        ]),
        vec![
            strings(rows.iter().map(|row| row.id.as_str())),
            strings(rows.iter().map(|row| row.kind.as_str())),
            strings(rows.iter().map(|row| row.source.as_str())),
            strings(rows.iter().map(|row| row.target.as_str())),
            strings(
                rows.iter()
                    .map(|row| row.read_model_projection_staleness.as_str()),
            ),
            strings(rows.iter().map(|row| row.domain_id.as_str())),
            strings(rows.iter().map(|row| row.source_contract_id.as_str())),
            strings(rows.iter().map(|row| row.evidence_path.as_str())),
            booleans(rows.iter().map(|row| row.ontology_truth)),
            strings(rows.iter().map(|row| row.status.as_str())),
        ],
    )
    .context("failed to build structural facts relation read-model")
}

fn write_parquet(path: &Path, batch: &RecordBatch) -> Result<()> {
    let file =
        File::create(path).with_context(|| format!("failed to create `{}`", path.display()))?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)
        .context("failed to create structural facts Parquet writer")?;
    writer
        .write(batch)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writer
        .close()
        .with_context(|| format!("failed to close `{}`", path.display()))?;
    Ok(())
}

fn schema_ref<const N: usize>(fields: [Field; N]) -> SchemaRef {
    Arc::new(Schema::new(fields.into_iter().collect::<Vec<_>>()))
}

fn string_field(name: &'static str) -> Field {
    Field::new(name, DataType::Utf8, false)
}

fn bool_field(name: &'static str) -> Field {
    Field::new(name, DataType::Boolean, false)
}

fn strings<'a>(values: impl Iterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from(values.collect::<Vec<_>>()))
}

fn booleans(values: impl Iterator<Item = bool>) -> ArrayRef {
    Arc::new(BooleanArray::from(values.collect::<Vec<_>>()))
}

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
