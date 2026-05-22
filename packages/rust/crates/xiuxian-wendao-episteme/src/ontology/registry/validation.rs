//! Conservative validation for typed `ontology/registry.json` snapshots.

use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use thiserror::Error;

use super::model::{
    EpistemeOntologyRegistryApiSurface, EpistemeOntologyRegistryDatasetMapping,
    EpistemeOntologyRegistryDomain, EpistemeOntologyRegistryObjectPropertyTerm,
    EpistemeOntologyRegistryPolicy, EpistemeOntologyRegistryRdfClassTerm,
    EpistemeOntologyRegistryReadModelInput, EpistemeOntologyRegistryRule,
    EpistemeOntologyRegistrySnapshot, EpistemeOntologyRegistrySnapshotReport,
    ONTOLOGY_REGISTRY_RELATIVE_PATH,
};

const ONTOLOGY_REGISTRY_SCHEMA_VERSION: u32 = 1;
const SOURCE_CONTRACT_ARTIFACT_MODE: &str = "source_contract";
const EPISTEME_DOMAIN_SCHEME: &str = "episteme://";

/// Error returned by Rust-owned ontology registry snapshot admission.
#[derive(Debug, Error)]
pub enum EpistemeOntologyRegistryError {
    /// A file or directory could not be accessed.
    #[error("failed to access `{path}`: {source}")]
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// The ontology registry JSON is malformed.
    #[error("failed to parse ontology registry `{path}`: {source}")]
    RegistryJson {
        /// Registry path.
        path: PathBuf,
        /// Underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
    /// The ontology registry snapshot is invalid.
    #[error("ontology registry snapshot is invalid: {0}")]
    InvalidSnapshot(String),
}

/// Return the absolute ontology registry snapshot path for an Episteme repository root.
#[must_use]
pub fn ontology_registry_path(episteme_root: impl AsRef<Path>) -> PathBuf {
    episteme_root.as_ref().join(ONTOLOGY_REGISTRY_RELATIVE_PATH)
}

/// Read `ontology/registry.json` from an Episteme repository.
///
/// # Errors
///
/// Returns an error when the registry snapshot cannot be read or parsed.
pub fn read_ontology_registry_snapshot(
    episteme_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyRegistrySnapshot, EpistemeOntologyRegistryError> {
    let path = ontology_registry_path(episteme_root);
    let raw = read_to_string(&path)?;
    serde_json::from_str::<EpistemeOntologyRegistrySnapshot>(&raw)
        .map_err(|source| EpistemeOntologyRegistryError::RegistryJson { path, source })
}

/// Read and admit `ontology/registry.json` as a Rust-owned read-model input.
///
/// # Errors
///
/// Returns an error when the registry snapshot is missing, malformed, or
/// violates conservative source-contract/read-model admission rules.
pub fn admit_ontology_registry_snapshot(
    episteme_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyRegistryReadModelInput, EpistemeOntologyRegistryError> {
    let episteme_root = episteme_root.as_ref();
    let snapshot = read_ontology_registry_snapshot(episteme_root)?;
    let report = validate_ontology_registry_snapshot(episteme_root, &snapshot)?;
    Ok(EpistemeOntologyRegistryReadModelInput { snapshot, report })
}

/// Validate a registry snapshot and return deterministic read-model counts.
///
/// # Errors
///
/// Returns an error when schema, source-contract, domain, artifact, RDF-term,
/// dataset-mapping, or API references are unsafe or inconsistent.
pub fn validate_ontology_registry_snapshot(
    episteme_root: impl AsRef<Path>,
    snapshot: &EpistemeOntologyRegistrySnapshot,
) -> Result<EpistemeOntologyRegistrySnapshotReport, EpistemeOntologyRegistryError> {
    let episteme_root = episteme_root.as_ref();
    validate_snapshot_header(snapshot)?;
    validate_source_contract(episteme_root, snapshot)?;

    let declared_domains = validate_domains(episteme_root, &snapshot.domains)?;
    validate_flattened_rules(episteme_root, &snapshot.rules, &declared_domains)?;
    validate_flattened_policies(episteme_root, &snapshot.policies, &declared_domains)?;
    validate_dataset_mappings(episteme_root, &snapshot.dataset_mappings, &declared_domains)?;
    validate_rdf_terms(
        episteme_root,
        &snapshot.rdf_terms.classes,
        &snapshot.rdf_terms.object_properties,
        &declared_domains,
    )?;
    validate_api_surface(&snapshot.api, &declared_domains)?;
    validate_reference_nouns(&snapshot.reference_nouns)?;

    Ok(EpistemeOntologyRegistrySnapshotReport {
        domains: snapshot.domains.len(),
        rdf_files: snapshot
            .domains
            .iter()
            .map(|domain| domain.rdf_files.len())
            .sum(),
        rules: snapshot.rules.len(),
        policies: snapshot.policies.len(),
        dataset_mappings: snapshot.dataset_mappings.len(),
        rdf_classes: snapshot.rdf_terms.classes.len(),
        rdf_object_properties: snapshot.rdf_terms.object_properties.len(),
        api_objects: snapshot.api.objects.len(),
        api_links: snapshot.api.links.len(),
        api_actions: snapshot.api.actions.len(),
        api_queries: snapshot.api.queries.len(),
        api_interfaces: snapshot.api.interfaces.len(),
        reference_nouns: snapshot.reference_nouns.len(),
    })
}

fn validate_snapshot_header(
    snapshot: &EpistemeOntologyRegistrySnapshot,
) -> Result<(), EpistemeOntologyRegistryError> {
    if snapshot.schema_version != ONTOLOGY_REGISTRY_SCHEMA_VERSION {
        return Err(invalid_snapshot(format!(
            "unsupported ontology registry schema_version: {}",
            snapshot.schema_version
        )));
    }
    ensure_non_blank("ontology", &snapshot.ontology)?;
    ensure_non_blank("compatibility", &snapshot.compatibility)?;
    if snapshot.domains.is_empty() {
        return Err(invalid_snapshot(
            "registry snapshot must declare at least one domain",
        ));
    }
    Ok(())
}

fn validate_source_contract(
    episteme_root: &Path,
    snapshot: &EpistemeOntologyRegistrySnapshot,
) -> Result<(), EpistemeOntologyRegistryError> {
    let contract = &snapshot.source_contract;
    ensure_non_blank("source_contract.manifest", &contract.manifest)?;
    ensure_non_blank(
        "source_contract.artifact_mode",
        contract.artifact_mode.as_str(),
    )?;
    if contract.artifact_mode.as_str() != SOURCE_CONTRACT_ARTIFACT_MODE {
        return Err(invalid_snapshot(format!(
            "source_contract.artifact_mode must be `{SOURCE_CONTRACT_ARTIFACT_MODE}`"
        )));
    }
    if contract.mutation_allowed {
        return Err(invalid_snapshot(
            "source_contract.mutation_allowed must be false",
        ));
    }
    ensure_non_blank(
        "source_contract.runtime_compilation_owner",
        &contract.runtime_compilation_owner,
    )?;
    ensure_non_blank(
        "source_contract.sdk_generation_owner",
        &contract.sdk_generation_owner,
    )?;
    ensure_existing_ontology_artifact(
        episteme_root,
        &contract.manifest,
        "source_contract.manifest",
    )?;
    if let Some(api_surface) = &contract.api_surface {
        ensure_existing_ontology_artifact(
            episteme_root,
            api_surface,
            "source_contract.api_surface",
        )?;
    }
    Ok(())
}

fn validate_domains(
    episteme_root: &Path,
    domains: &[EpistemeOntologyRegistryDomain],
) -> Result<BTreeSet<String>, EpistemeOntologyRegistryError> {
    let mut declared = BTreeSet::new();
    for domain in domains {
        ensure_non_blank("domains[].id", &domain.id)?;
        if !domain.id.starts_with(EPISTEME_DOMAIN_SCHEME) {
            return Err(invalid_snapshot(format!(
                "domain id must use {EPISTEME_DOMAIN_SCHEME} scheme: {}",
                domain.id
            )));
        }
        if !declared.insert(domain.id.clone()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry domain id: {}",
                domain.id
            )));
        }
        ensure_non_blank("domains[].name", &domain.name)?;
        ensure_artifact_list(episteme_root, &domain.rdf_files, "domains[].rdf_files")?;
        ensure_artifact_list(episteme_root, &domain.rules, "domains[].rules")?;
        ensure_artifact_list(episteme_root, &domain.policies, "domains[].policies")?;
        ensure_artifact_list(
            episteme_root,
            &domain.dataset_mappings,
            "domains[].dataset_mappings",
        )?;
    }
    Ok(declared)
}

fn validate_flattened_rules(
    episteme_root: &Path,
    rules: &[EpistemeOntologyRegistryRule],
    declared_domains: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    let mut seen = BTreeSet::new();
    for rule in rules {
        ensure_declared_domain("rules[].domain", &rule.domain, declared_domains)?;
        ensure_non_blank("rules[].kind", rule.kind.as_str())?;
        ensure_existing_ontology_artifact(episteme_root, &rule.path, "rules[].path")?;
        if !seen.insert(rule.path.as_str()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry rule path: {}",
                rule.path
            )));
        }
    }
    Ok(())
}

fn validate_flattened_policies(
    episteme_root: &Path,
    policies: &[EpistemeOntologyRegistryPolicy],
    declared_domains: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    let mut seen = BTreeSet::new();
    for policy in policies {
        ensure_declared_domain("policies[].domain", &policy.domain, declared_domains)?;
        ensure_non_blank("policies[].kind", policy.kind.as_str())?;
        ensure_existing_ontology_artifact(episteme_root, &policy.path, "policies[].path")?;
        if !seen.insert(policy.path.as_str()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry policy path: {}",
                policy.path
            )));
        }
    }
    Ok(())
}

fn validate_dataset_mappings(
    episteme_root: &Path,
    mappings: &[EpistemeOntologyRegistryDatasetMapping],
    declared_domains: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    let mut seen = BTreeSet::new();
    for mapping in mappings {
        ensure_declared_domain(
            "dataset_mappings[].domain",
            &mapping.domain,
            declared_domains,
        )?;
        ensure_non_blank("dataset_mappings[].kind", mapping.kind.as_str())?;
        ensure_non_blank("dataset_mappings[].mapping_id", &mapping.mapping_id)?;
        if !seen.insert(mapping.mapping_id.as_str()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry dataset mapping id: {}",
                mapping.mapping_id
            )));
        }
        ensure_existing_ontology_artifact(episteme_root, &mapping.path, "dataset_mappings[].path")?;
        ensure_existing_ontology_artifact(
            episteme_root,
            &mapping.ledger_org,
            "dataset_mappings[].ledger_org",
        )?;
        for (key, path) in &mapping.materialization {
            ensure_non_blank("dataset_mappings[].materialization key", key)?;
            ensure_existing_ontology_artifact(
                episteme_root,
                path,
                "dataset_mappings[].materialization",
            )?;
        }
        for raw_table in &mapping.raw_tables {
            ensure_non_blank("dataset_mappings[].raw_tables[]", raw_table)?;
        }
        ensure_artifact_list(
            episteme_root,
            &mapping.validation_rules,
            "dataset_mappings[].validation_rules",
        )?;
    }
    Ok(())
}

fn validate_rdf_terms(
    episteme_root: &Path,
    classes: &[EpistemeOntologyRegistryRdfClassTerm],
    object_properties: &[EpistemeOntologyRegistryObjectPropertyTerm],
    declared_domains: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    let mut seen_iris = BTreeSet::new();
    for class in classes {
        ensure_declared_domain(
            "rdf_terms.classes[].domain",
            &class.domain,
            declared_domains,
        )?;
        validate_rdf_term_common(
            episteme_root,
            &mut seen_iris,
            "rdf_terms.classes",
            &class.iri,
            &class.label,
            &class.api_candidate,
            &class.source_file,
        )?;
    }
    for property in object_properties {
        ensure_declared_domain(
            "rdf_terms.object_properties[].domain",
            &property.domain,
            declared_domains,
        )?;
        validate_rdf_term_common(
            episteme_root,
            &mut seen_iris,
            "rdf_terms.object_properties",
            &property.iri,
            &property.label,
            &property.api_candidate,
            &property.source_file,
        )?;
        if property
            .from_iri
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(invalid_snapshot(
                "rdf_terms.object_properties[].from_iri must not be blank when present",
            ));
        }
        if property
            .to_iri
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(invalid_snapshot(
                "rdf_terms.object_properties[].to_iri must not be blank when present",
            ));
        }
    }
    Ok(())
}

fn validate_rdf_term_common(
    episteme_root: &Path,
    seen_iris: &mut BTreeSet<String>,
    field: &str,
    iri: &str,
    label: &str,
    api_candidate: &str,
    source_file: &str,
) -> Result<(), EpistemeOntologyRegistryError> {
    ensure_non_blank(&format!("{field}[].iri"), iri)?;
    ensure_non_blank(&format!("{field}[].label"), label)?;
    ensure_non_blank(&format!("{field}[].api_candidate"), api_candidate)?;
    ensure_existing_ontology_artifact(
        episteme_root,
        source_file,
        &format!("{field}[].source_file"),
    )?;
    if !seen_iris.insert(iri.to_string()) {
        return Err(invalid_snapshot(format!(
            "duplicate ontology registry RDF term iri: {iri}"
        )));
    }
    Ok(())
}

fn validate_api_surface(
    api: &EpistemeOntologyRegistryApiSurface,
    declared_domains: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    let object_names = validate_api_objects(&api.objects, declared_domains)?;
    let interface_names = validate_api_interfaces(&api.interfaces, &object_names)?;
    validate_object_interfaces(&api.objects, &interface_names)?;
    validate_api_links(&api.links, declared_domains, &object_names)?;
    validate_api_actions(&api.actions, declared_domains, &object_names)?;
    validate_api_queries(&api.queries, declared_domains, &object_names)
}

fn validate_api_objects(
    objects: &[super::model::EpistemeOntologyRegistryObjectType],
    declared_domains: &BTreeSet<String>,
) -> Result<BTreeSet<String>, EpistemeOntologyRegistryError> {
    let mut object_names = BTreeSet::new();
    for object in objects {
        ensure_declared_domain(
            "api.object_types[].domain",
            &object.domain,
            declared_domains,
        )?;
        ensure_non_blank("api.object_types[].api_name", &object.api_name)?;
        ensure_non_blank("api.object_types[].rdf_class", &object.rdf_class)?;
        ensure_non_blank(
            "api.object_types[].display_name_property",
            &object.display_name_property,
        )?;
        if object.primary_key.is_empty() {
            return Err(invalid_snapshot(
                "api.object_types[].primary_key must not be empty",
            ));
        }
        for key in &object.primary_key {
            ensure_non_blank("api.object_types[].primary_key[]", key)?;
        }
        if !object_names.insert(object.api_name.clone()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry API object type: {}",
                object.api_name
            )));
        }
    }
    Ok(object_names)
}

fn validate_api_interfaces(
    interfaces: &[super::model::EpistemeOntologyRegistryInterfaceType],
    object_names: &BTreeSet<String>,
) -> Result<BTreeSet<String>, EpistemeOntologyRegistryError> {
    let mut interface_names = BTreeSet::new();
    for interface in interfaces {
        ensure_non_blank("api.interface_types[].api_name", &interface.api_name)?;
        if !interface_names.insert(interface.api_name.clone()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry API interface type: {}",
                interface.api_name
            )));
        }
        for object in &interface.implemented_by {
            ensure_known_object_type(
                "api.interface_types[].implemented_by[]",
                object,
                object_names,
            )?;
        }
    }
    Ok(interface_names)
}

fn validate_object_interfaces(
    objects: &[super::model::EpistemeOntologyRegistryObjectType],
    interface_names: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    for object in objects {
        for interface in &object.interfaces {
            if !interface_names.contains(interface.as_str()) {
                return Err(invalid_snapshot(format!(
                    "api.object_types[].interfaces[] references undeclared interface `{interface}`"
                )));
            }
        }
    }
    Ok(())
}

fn validate_api_links(
    links: &[super::model::EpistemeOntologyRegistryLinkType],
    declared_domains: &BTreeSet<String>,
    object_names: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    let mut link_names = BTreeSet::new();
    for link in links {
        ensure_declared_domain("api.link_types[].domain", &link.domain, declared_domains)?;
        ensure_non_blank("api.link_types[].api_name", &link.api_name)?;
        ensure_non_blank("api.link_types[].rdf_property", &link.rdf_property)?;
        ensure_non_blank("api.link_types[].cardinality", &link.cardinality)?;
        ensure_known_object_type(
            "api.link_types[].from_object_type",
            link.from_object_type.as_str(),
            object_names,
        )?;
        ensure_known_object_type(
            "api.link_types[].to_object_type",
            link.to_object_type.as_str(),
            object_names,
        )?;
        if !link_names.insert(link.api_name.as_str()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry API link type: {}",
                link.api_name
            )));
        }
    }
    Ok(())
}

fn validate_api_actions(
    actions: &[super::model::EpistemeOntologyRegistryActionType],
    declared_domains: &BTreeSet<String>,
    object_names: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    let mut action_names = BTreeSet::new();
    for action in actions {
        ensure_declared_domain(
            "api.action_types[].domain",
            &action.domain,
            declared_domains,
        )?;
        ensure_non_blank("api.action_types[].api_name", &action.api_name)?;
        for object in &action.affected_object_types {
            ensure_known_object_type(
                "api.action_types[].affected_object_types[]",
                object,
                object_names,
            )?;
        }
        for rule in &action.validation_rules {
            ensure_non_blank("api.action_types[].validation_rules[]", rule)?;
        }
        if !action_names.insert(action.api_name.as_str()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry API action type: {}",
                action.api_name
            )));
        }
    }
    Ok(())
}

fn validate_api_queries(
    queries: &[super::model::EpistemeOntologyRegistryQueryType],
    declared_domains: &BTreeSet<String>,
    object_names: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    let mut query_names = BTreeSet::new();
    for query in queries {
        ensure_declared_domain("api.query_types[].domain", &query.domain, declared_domains)?;
        ensure_non_blank("api.query_types[].api_name", &query.api_name)?;
        ensure_known_object_type("api.query_types[].returns", &query.returns, object_names)?;
        for parameter in &query.parameters {
            ensure_non_blank("api.query_types[].parameters[]", parameter)?;
        }
        if !query_names.insert(query.api_name.as_str()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry API query type: {}",
                query.api_name
            )));
        }
    }
    Ok(())
}

fn validate_reference_nouns(
    reference_nouns: &[String],
) -> Result<(), EpistemeOntologyRegistryError> {
    let mut seen = BTreeSet::new();
    for noun in reference_nouns {
        ensure_non_blank("reference_nouns[]", noun)?;
        if !seen.insert(noun.as_str()) {
            return Err(invalid_snapshot(format!(
                "duplicate ontology registry reference noun: {noun}"
            )));
        }
    }
    Ok(())
}

fn ensure_known_object_type(
    field: &str,
    object_type: &str,
    object_names: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    ensure_non_blank(field, object_type)?;
    if !object_names.contains(object_type) {
        return Err(invalid_snapshot(format!(
            "{field} references undeclared object type `{object_type}`"
        )));
    }
    Ok(())
}

fn ensure_declared_domain(
    field: &str,
    domain: &str,
    declared_domains: &BTreeSet<String>,
) -> Result<(), EpistemeOntologyRegistryError> {
    ensure_non_blank(field, domain)?;
    if !declared_domains.contains(domain) {
        return Err(invalid_snapshot(format!(
            "{field} references undeclared domain `{domain}`"
        )));
    }
    Ok(())
}

fn ensure_artifact_list(
    episteme_root: &Path,
    paths: &[String],
    field: &str,
) -> Result<(), EpistemeOntologyRegistryError> {
    for path in paths {
        ensure_existing_ontology_artifact(episteme_root, path, field)?;
    }
    Ok(())
}

fn ensure_existing_ontology_artifact(
    episteme_root: &Path,
    raw: &str,
    field: &str,
) -> Result<(), EpistemeOntologyRegistryError> {
    let path = resolve_ontology_artifact_path(episteme_root, raw, field)?;
    if !path.is_file() {
        return Err(invalid_snapshot(format!(
            "`{field}` entry does not exist or is not a file: {raw}"
        )));
    }
    Ok(())
}

fn resolve_ontology_artifact_path(
    episteme_root: &Path,
    raw: &str,
    field: &str,
) -> Result<PathBuf, EpistemeOntologyRegistryError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(invalid_snapshot(format!(
            "`{field}` entries must not be blank"
        )));
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(invalid_snapshot(format!(
            "`{field}` entries must be safe paths relative to ontology/: {trimmed}"
        )));
    }
    Ok(episteme_root.join("ontology").join(path))
}

fn ensure_non_blank(field: &str, value: &str) -> Result<(), EpistemeOntologyRegistryError> {
    if value.trim().is_empty() {
        return Err(invalid_snapshot(format!("{field} must not be blank")));
    }
    Ok(())
}

fn read_to_string(path: &Path) -> Result<String, EpistemeOntologyRegistryError> {
    fs::read_to_string(path).map_err(|source| EpistemeOntologyRegistryError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn invalid_snapshot(message: impl Into<String>) -> EpistemeOntologyRegistryError {
    EpistemeOntologyRegistryError::InvalidSnapshot(message.into())
}
