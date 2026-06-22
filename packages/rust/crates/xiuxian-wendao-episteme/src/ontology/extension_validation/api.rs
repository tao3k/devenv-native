//! API surface for Episteme extension-pack validation.

use anyhow::{Result, bail};

use crate::ontology::manifest::{read_ontology_manifest, validate_ontology_contract};
use crate::source_contract::source_contract_paths;

use super::{
    EpistemeExtensionValidationReport, EpistemeExtensionValidationRequest, object_model, rdf,
    source,
};

/// Validate an Episteme extension-pack source contract.
///
/// # Errors
///
/// Returns an error when the Episteme repository is not an extension source
/// contract, when source artifacts are missing or inconsistent, when corpus
/// hashes drift, or when object-model/RDF cross references are invalid.
pub fn validate_episteme_extension_contract(
    request: &EpistemeExtensionValidationRequest,
) -> Result<EpistemeExtensionValidationReport> {
    let contract_report = validate_ontology_contract(request.episteme_root.as_path())
        .map_err(|source| anyhow::anyhow!(source))?;
    let manifest = read_ontology_manifest(request.episteme_root.as_path())
        .map_err(|source| anyhow::anyhow!(source))?;
    let artifact_mode = manifest
        .artifact_mode
        .as_ref()
        .unwrap_or(&manifest.boundaries.artifact_mode)
        .as_str();
    if artifact_mode != "extension_source_contract" {
        bail!(
            "extension-pack validation requires artifact_mode `extension_source_contract`; common source contracts validate through `validate_ontology_contract`"
        );
    }
    if manifest
        .primary_language
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        bail!("extension-pack validation requires manifest primary_language");
    }

    let source_paths = source_contract_paths(request.episteme_root.as_path())
        .map_err(|source| anyhow::anyhow!(source))?;
    let source_report = source::validate_extension_sources(request, &source_paths)?;
    let rdf_terms = rdf::collect_extension_rdf_terms(request.episteme_root.as_path(), &manifest)?;
    let object_report = object_model::validate_extension_object_models(
        request.episteme_root.as_path(),
        &manifest,
        &rdf_terms,
    )?;

    Ok(EpistemeExtensionValidationReport {
        domains: contract_report.domain_count,
        rdf_files: contract_report.rdf_file_count,
        object_model_contracts: contract_report.object_model_contract_count,
        source_manifests: source_report.source_manifests,
        source_files: source_report.source_files,
        extraction_queue_rows: source_report.extraction_queue_rows,
        rdf_classes: rdf_terms.class_count(),
        rdf_object_properties: rdf_terms.object_property_count(),
        object_types: object_report.objects,
        property_types: object_report.properties,
        link_types: object_report.links,
        action_types: object_report.actions,
        query_types: object_report.queries,
    })
}
