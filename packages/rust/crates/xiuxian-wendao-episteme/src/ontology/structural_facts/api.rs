use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use super::super::manifest::read_ontology_manifest;
use crate::{configured_episteme_corpus_root_env, load_episteme_runtime_config};

use super::{
    builder::StructuralFactsBuilder,
    rdf_seed::write_structural_facts_rdf_seed,
    read_model::{compile_structural_facts_read_model, write_structural_facts_read_model},
    types::{
        EpistemeOntologyStructuralFactsConfiguredRequest, EpistemeOntologyStructuralFactsReport,
        EpistemeOntologyStructuralFactsRequest, EpistemeOntologyStructuralFactsSafetyFlags,
        EpistemeOntologyStructuralFactsSnapshot, EpistemeOntologyStructuralFactsValidationMode,
        STRUCTURAL_FACTS_REPORT_SCHEMA_VERSION, STRUCTURAL_FACTS_SCHEMA_VERSION,
        StructuralFactsOutputPaths,
    },
    validation::validate_run_id,
    write::{
        write_anchors_tsv, write_documents_tsv, write_json, write_relations_tsv,
        write_structural_facts_org,
    },
};

/// Compile structural facts using Episteme-owned runtime defaults.
///
/// # Errors
///
/// Returns an error when runtime defaults cannot resolve the source corpus,
/// the structural facts request is invalid, source validation fails, or output
/// artifacts cannot be written.
pub fn write_episteme_ontology_structural_facts_from_config(
    request: &EpistemeOntologyStructuralFactsConfiguredRequest,
) -> Result<EpistemeOntologyStructuralFactsReport> {
    let config = load_episteme_runtime_config(request.episteme_root.as_path())
        .context("failed to load Episteme runtime config")?;
    let corpus_root = resolve_configured_corpus_root(request, config.as_ref())?;
    let run_root = request
        .run_root
        .clone()
        .or_else(|| {
            config
                .as_ref()
                .and_then(|config| config.structure_runs.clone())
        })
        .unwrap_or_else(|| request.episteme_root.join("runs/structure"));
    let structural_request = EpistemeOntologyStructuralFactsRequest::new(
        request.episteme_root.clone(),
        corpus_root,
        request.run_id.clone(),
    )
    .with_validation_mode(request.validation_mode);
    write_episteme_ontology_structural_facts(&structural_request, run_root)
}

/// Compile deterministic structural facts seed artifacts from private source
/// contracts.
///
/// # Errors
///
/// Returns an error when the ontology/source manifests are invalid, source
/// files are missing or drift from the selected validation mode, or output
/// artifacts cannot be written.
pub fn write_episteme_ontology_structural_facts(
    request: &EpistemeOntologyStructuralFactsRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyStructuralFactsReport> {
    validate_run_id(&request.run_id)?;
    let snapshot = build_snapshot(request)?;
    let read_model = compile_structural_facts_read_model(&snapshot);
    if !read_model.quality_issues.is_empty() {
        bail!(
            "structural facts read-model quality checks failed: {}",
            read_model.quality_issues.join("; ")
        );
    }
    let paths = StructuralFactsOutputPaths::new(run_root.as_ref(), request.run_id.as_str());

    fs::create_dir_all(paths.run_dir.as_path())
        .with_context(|| format!("failed to create `{}`", paths.run_dir.display()))?;
    write_documents_tsv(paths.documents_tsv.as_path(), &snapshot.documents)?;
    write_json(paths.documents_json.as_path(), &snapshot.documents)?;
    write_anchors_tsv(paths.anchors_tsv.as_path(), &snapshot.anchors)?;
    write_json(paths.anchors_json.as_path(), &snapshot.anchors)?;
    write_relations_tsv(paths.relations_tsv.as_path(), &snapshot.relations)?;
    write_json(paths.relations_json.as_path(), &snapshot.relations)?;
    write_json(paths.structural_facts_json.as_path(), &snapshot)?;
    write_structural_facts_read_model(&paths, &read_model)?;
    write_structural_facts_rdf_seed(paths.rdf_seed_ttl.as_path(), &read_model)?;

    let report = build_report(request, &paths, &snapshot, &read_model);
    write_structural_facts_org(paths.structural_facts_org.as_path(), &report)?;
    Ok(report)
}

fn resolve_configured_corpus_root(
    request: &EpistemeOntologyStructuralFactsConfiguredRequest,
    config: Option<&crate::EpistemeRuntimeConfig>,
) -> Result<PathBuf> {
    if let Some(corpus_root) = &request.corpus_root {
        return Ok(corpus_root.clone());
    }
    if let Some(corpus_root) = config.and_then(|config| config.corpus.clone()) {
        return Ok(corpus_root);
    }
    let corpus_root_env = configured_episteme_corpus_root_env(request.episteme_root.as_path())
        .context("failed to read Episteme source-contract corpus root env")?;
    env::var_os(corpus_root_env.as_str())
        .map(PathBuf::from)
        .with_context(|| {
            format!(
                "runtime.corpus_root is required in episteme.toml when corpus_root override and {corpus_root_env} are unset"
            )
        })
}

fn build_snapshot(
    request: &EpistemeOntologyStructuralFactsRequest,
) -> Result<EpistemeOntologyStructuralFactsSnapshot> {
    let manifest = read_ontology_manifest(request.episteme_root.as_path())
        .map_err(|source| anyhow::anyhow!(source))?;
    let mut builder = StructuralFactsBuilder::new(request);

    for domain in &manifest.domains {
        for source_manifest in &domain.source_manifests {
            builder.compile_source_manifest(domain.id.as_str(), source_manifest.as_str())?;
        }
    }
    if builder.source_contracts.is_empty() {
        bail!("ontology manifest declares no source manifests for structural facts compilation");
    }

    Ok(EpistemeOntologyStructuralFactsSnapshot {
        schema_version: STRUCTURAL_FACTS_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        source_contracts: builder.source_contracts,
        documents: builder.documents,
        anchors: builder.anchors,
        relations: builder.relations,
    })
}

fn build_report(
    request: &EpistemeOntologyStructuralFactsRequest,
    paths: &StructuralFactsOutputPaths,
    snapshot: &EpistemeOntologyStructuralFactsSnapshot,
    read_model: &super::read_model::StructuralFactsReadModel,
) -> EpistemeOntologyStructuralFactsReport {
    EpistemeOntologyStructuralFactsReport {
        schema_version: STRUCTURAL_FACTS_REPORT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        run_dir: paths.run_dir.clone(),
        structural_facts_json: paths.structural_facts_json.clone(),
        structural_facts_org: paths.structural_facts_org.clone(),
        documents_tsv: paths.documents_tsv.clone(),
        documents_json: paths.documents_json.clone(),
        anchors_tsv: paths.anchors_tsv.clone(),
        anchors_json: paths.anchors_json.clone(),
        relations_tsv: paths.relations_tsv.clone(),
        relations_json: paths.relations_json.clone(),
        rdf_seed_ttl: paths.rdf_seed_ttl.clone(),
        read_model_objects_tsv: paths.read_model_objects_tsv.clone(),
        read_model_objects_json: paths.read_model_objects_json.clone(),
        read_model_objects_parquet: paths.read_model_objects_parquet.clone(),
        read_model_relations_tsv: paths.read_model_relations_tsv.clone(),
        read_model_relations_json: paths.read_model_relations_json.clone(),
        read_model_relations_parquet: paths.read_model_relations_parquet.clone(),
        read_model_projection_state_json: paths.read_model_projection_state_json.clone(),
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
        read_model_object_count: read_model.objects.len(),
        read_model_relation_count: read_model.relations.len(),
        read_model_projection_state_count: read_model.projection_state.len(),
        read_model_quality_passed: read_model.quality_issues.is_empty(),
        read_model_quality_issues: read_model.quality_issues.clone(),
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
        safety: EpistemeOntologyStructuralFactsSafetyFlags {
            extraction_executed: false,
            source_mutation_allowed: false,
            ontology_truth: false,
        },
        validation_mode: request.validation_mode,
        full_hash_checked: request.validation_mode
            == EpistemeOntologyStructuralFactsValidationMode::FullHash,
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
