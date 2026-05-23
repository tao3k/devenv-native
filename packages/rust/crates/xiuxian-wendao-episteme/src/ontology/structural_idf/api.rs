use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, bail};

use super::super::manifest::read_ontology_manifest;
use super::{
    builder::StructuralIdfBuilder,
    types::{
        EpistemeOntologyStructuralIdfReport, EpistemeOntologyStructuralIdfRequest,
        EpistemeOntologyStructuralIdfSafetyFlags, EpistemeOntologyStructuralIdfSnapshot,
        EpistemeOntologyStructuralIdfValidationMode, STRUCTURAL_IDF_REPORT_SCHEMA_VERSION,
        STRUCTURAL_IDF_SCHEMA_VERSION, StructuralIdfOutputPaths,
    },
    validation::validate_run_id,
    write::{
        write_anchors_tsv, write_documents_tsv, write_json, write_relations_tsv,
        write_structural_idf_org,
    },
};

/// Compile deterministic structural IDF seed artifacts from private source
/// contracts.
///
/// # Errors
///
/// Returns an error when the ontology/source manifests are invalid, source
/// files are missing or drift from the selected validation mode, or output
/// artifacts cannot be written.
pub fn write_episteme_ontology_structural_idf(
    request: &EpistemeOntologyStructuralIdfRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyStructuralIdfReport> {
    validate_run_id(&request.run_id)?;
    let snapshot = build_snapshot(request)?;
    let paths = StructuralIdfOutputPaths::new(run_root.as_ref(), request.run_id.as_str());

    fs::create_dir_all(paths.run_dir.as_path())
        .with_context(|| format!("failed to create `{}`", paths.run_dir.display()))?;
    write_documents_tsv(paths.documents_tsv.as_path(), &snapshot.documents)?;
    write_json(paths.documents_json.as_path(), &snapshot.documents)?;
    write_anchors_tsv(paths.anchors_tsv.as_path(), &snapshot.anchors)?;
    write_json(paths.anchors_json.as_path(), &snapshot.anchors)?;
    write_relations_tsv(paths.relations_tsv.as_path(), &snapshot.relations)?;
    write_json(paths.relations_json.as_path(), &snapshot.relations)?;
    write_json(paths.structural_idf_json.as_path(), &snapshot)?;

    let report = build_report(request, &paths, &snapshot);
    write_structural_idf_org(paths.structural_idf_org.as_path(), &report)?;
    Ok(report)
}

fn build_snapshot(
    request: &EpistemeOntologyStructuralIdfRequest,
) -> Result<EpistemeOntologyStructuralIdfSnapshot> {
    let manifest = read_ontology_manifest(request.episteme_root.as_path())
        .map_err(|source| anyhow::anyhow!(source))?;
    let mut builder = StructuralIdfBuilder::new(request);

    for domain in &manifest.domains {
        for source_manifest in &domain.source_manifests {
            builder.compile_source_manifest(domain.id.as_str(), source_manifest.as_str())?;
        }
    }
    if builder.source_contracts.is_empty() {
        bail!("ontology manifest declares no source manifests for structural IDF compilation");
    }

    Ok(EpistemeOntologyStructuralIdfSnapshot {
        schema_version: STRUCTURAL_IDF_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        source_contracts: builder.source_contracts,
        documents: builder.documents,
        anchors: builder.anchors,
        relations: builder.relations,
    })
}

fn build_report(
    request: &EpistemeOntologyStructuralIdfRequest,
    paths: &StructuralIdfOutputPaths,
    snapshot: &EpistemeOntologyStructuralIdfSnapshot,
) -> EpistemeOntologyStructuralIdfReport {
    EpistemeOntologyStructuralIdfReport {
        schema_version: STRUCTURAL_IDF_REPORT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        run_dir: paths.run_dir.clone(),
        structural_idf_json: paths.structural_idf_json.clone(),
        structural_idf_org: paths.structural_idf_org.clone(),
        documents_tsv: paths.documents_tsv.clone(),
        documents_json: paths.documents_json.clone(),
        anchors_tsv: paths.anchors_tsv.clone(),
        anchors_json: paths.anchors_json.clone(),
        relations_tsv: paths.relations_tsv.clone(),
        relations_json: paths.relations_json.clone(),
        domain_count: snapshot
            .source_contracts
            .iter()
            .map(|contract| contract.domain_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        source_manifest_count: snapshot.source_contracts.len(),
        file_count: snapshot.documents.len(),
        document_count: snapshot.documents.len(),
        anchor_count: snapshot.anchors.len(),
        relation_count: snapshot.relations.len(),
        route_counts: count_by(
            snapshot
                .documents
                .iter()
                .map(|document| document.extraction_route.as_str()),
        ),
        category_counts: count_by(
            snapshot
                .documents
                .iter()
                .map(|document| document.category.as_str()),
        ),
        safety: EpistemeOntologyStructuralIdfSafetyFlags {
            extraction_executed: false,
            source_mutation_allowed: false,
            ontology_truth: false,
        },
        validation_mode: request.validation_mode,
        full_hash_checked: request.validation_mode
            == EpistemeOntologyStructuralIdfValidationMode::FullHash,
        hash_drift_count: 0,
    }
}

fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}
