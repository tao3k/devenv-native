//! Read-model materialization for admitted ontology registry snapshots.

use std::path::Path;

use sha2::{Digest, Sha256};
use xiuxian_wendao_episteme::{
    EpistemeOntologyRegistryActionType, EpistemeOntologyRegistryDatasetMapping,
    EpistemeOntologyRegistryInterfaceType, EpistemeOntologyRegistryLinkType,
    EpistemeOntologyRegistryObjectPropertyTerm, EpistemeOntologyRegistryObjectType,
    EpistemeOntologyRegistryPolicy, EpistemeOntologyRegistryQueryType,
    EpistemeOntologyRegistryRdfClassTerm, EpistemeOntologyRegistryReadModelInput,
    EpistemeOntologyRegistryRule, admit_ontology_registry_snapshot,
};

use super::{
    EpistemeError, EpistemeReadModelMaterialization, EpistemeReadModelTable, OBJECTS_TABLE,
    PROJECTION_STATE_TABLE, RECORDED_AT, RECORDED_BY, RELATIONS_TABLE, STALENESS_FRESH,
    STATUS_ACTIVE, SemanticObjectRow, SemanticProjectionStateRow, SemanticRelationRow, json_array,
    owners_json, semantic_objects_batch, semantic_projection_state_batch, semantic_relation_counts,
    semantic_relations_batch,
};

const SNAPSHOT_OBJECT_KIND: &str = "episteme_registry.snapshot";
const DOMAIN_OBJECT_KIND: &str = "episteme_registry.domain";
const RDF_CLASS_OBJECT_KIND: &str = "episteme_registry.rdf_class";
const RDF_PROPERTY_OBJECT_KIND: &str = "episteme_registry.rdf_object_property";
const RULE_OBJECT_KIND: &str = "episteme_registry.validation_rule";
const POLICY_OBJECT_KIND: &str = "episteme_registry.policy";
const DATASET_MAPPING_OBJECT_KIND: &str = "episteme_registry.dataset_mapping";
const API_OBJECT_KIND: &str = "episteme_registry.api_object_type";
const API_LINK_OBJECT_KIND: &str = "episteme_registry.api_link_type";
const API_ACTION_OBJECT_KIND: &str = "episteme_registry.api_action_type";
const API_QUERY_OBJECT_KIND: &str = "episteme_registry.api_query_type";
const API_INTERFACE_OBJECT_KIND: &str = "episteme_registry.api_interface_type";

const SNAPSHOT_DOMAIN_RELATION_KIND: &str = "episteme_registry.snapshot.declares_domain";
const DOMAIN_TERM_RELATION_KIND: &str = "episteme_registry.domain.declares_term";
const DOMAIN_RULE_RELATION_KIND: &str = "episteme_registry.domain.declares_rule";
const DOMAIN_POLICY_RELATION_KIND: &str = "episteme_registry.domain.declares_policy";
const DOMAIN_MAPPING_RELATION_KIND: &str = "episteme_registry.domain.declares_dataset_mapping";
const DOMAIN_API_RELATION_KIND: &str = "episteme_registry.domain.declares_api_surface";
const API_LINK_SOURCE_RELATION_KIND: &str = "episteme_registry.api_link.from_object";
const API_LINK_TARGET_RELATION_KIND: &str = "episteme_registry.api_link.to_object";
const API_ACTION_OBJECT_RELATION_KIND: &str = "episteme_registry.api_action.affects_object";
const API_QUERY_OBJECT_RELATION_KIND: &str = "episteme_registry.api_query.returns_object";
const API_INTERFACE_OBJECT_RELATION_KIND: &str = "episteme_registry.api_interface.implemented_by";

const REGISTRY_SNAPSHOT_CONFIDENCE_SOURCE: &str = "ontology_registry_snapshot_admission";
const REGISTRY_SNAPSHOT_PROJECTION_ID: &str = "episteme_registry.snapshot_read_model_seed.v1";
const REGISTRY_SNAPSHOT_PROJECTION_REVISION: &str = "episteme_registry.snapshot_read_model_seed.v1";
const REGISTRY_SNAPSHOT_SOURCE_PATH: &str = "ontology/registry.json";

/// Admit `ontology/registry.json` from an Episteme root and compile it into
/// graph-readable semantic read-model seed batches.
///
/// # Errors
///
/// Returns an error when registry snapshot admission fails, row materialization
/// fails, or emitted relation endpoints do not reference emitted objects.
pub fn admit_and_materialize_episteme_ontology_registry_snapshot_read_model_seed(
    episteme_root: impl AsRef<Path>,
) -> Result<EpistemeReadModelMaterialization, EpistemeError> {
    let input = admit_ontology_registry_snapshot(episteme_root).map_err(EpistemeError::from)?;
    let materialization = materialize_episteme_ontology_registry_snapshot_read_model_seed(&input)?;
    super::validate_episteme_read_model_relation_endpoints(&materialization)?;
    Ok(materialization)
}

struct RegistrySnapshotObjectSeed {
    id: String,
    kind: &'static str,
    title: String,
    source_path: String,
    evidence: Vec<String>,
}

/// Compile an admitted ontology registry snapshot into graph-readable semantic
/// read-model seed batches.
///
/// # Errors
///
/// Returns an error when registry snapshot rows cannot be encoded into the
/// stable Arrow read-model table schemas.
pub fn materialize_episteme_ontology_registry_snapshot_read_model_seed(
    input: &EpistemeOntologyRegistryReadModelInput,
) -> Result<EpistemeReadModelMaterialization, EpistemeError> {
    let source_revision = registry_snapshot_revision(input)?;
    let snapshot_id = snapshot_object_id(&input.snapshot.ontology);
    let mut seeds = registry_snapshot_object_seeds(input);
    let mut relations = registry_snapshot_relation_rows(input, snapshot_id.as_str());
    add_rdf_term_rows(input, &mut seeds, &mut relations);
    add_rule_policy_mapping_rows(input, &mut seeds, &mut relations);
    add_api_rows(input, &mut seeds, &mut relations);

    stamp_relation_rows(&mut relations, source_revision.as_str());
    let object_rows = object_rows_from_seeds(&seeds, &relations, source_revision.as_str())?;
    let projection_rows =
        registry_snapshot_projection_rows(&object_rows, source_revision.as_str())?;

    Ok(EpistemeReadModelMaterialization {
        source_revision,
        tables: vec![
            EpistemeReadModelTable::new(OBJECTS_TABLE, semantic_objects_batch(&object_rows)?),
            EpistemeReadModelTable::new(RELATIONS_TABLE, semantic_relations_batch(&relations)?),
            EpistemeReadModelTable::new(
                PROJECTION_STATE_TABLE,
                semantic_projection_state_batch(&projection_rows)?,
            ),
        ],
    })
}

fn registry_snapshot_object_seeds(
    input: &EpistemeOntologyRegistryReadModelInput,
) -> Vec<RegistrySnapshotObjectSeed> {
    let snapshot_id = snapshot_object_id(&input.snapshot.ontology);
    let mut seeds = vec![RegistrySnapshotObjectSeed {
        id: snapshot_id,
        kind: SNAPSHOT_OBJECT_KIND,
        title: input.snapshot.ontology.clone(),
        source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
        evidence: vec![
            format!("schema_version:{}", input.snapshot.schema_version),
            format!("domain_count:{}", input.report.domains),
            format!("rdf_class_count:{}", input.report.rdf_classes),
            format!("api_object_count:{}", input.report.api_objects),
        ],
    }];
    seeds.extend(
        input
            .snapshot
            .domains
            .iter()
            .map(|domain| RegistrySnapshotObjectSeed {
                id: domain_object_id(&domain.id),
                kind: DOMAIN_OBJECT_KIND,
                title: domain.name.clone(),
                source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
                evidence: vec![
                    format!("domain:{}", domain.id),
                    format!("rdf_files:{}", domain.rdf_files.len()),
                ],
            }),
    );
    seeds
}

fn registry_snapshot_relation_rows(
    input: &EpistemeOntologyRegistryReadModelInput,
    snapshot_id: &str,
) -> Vec<SemanticRelationRow> {
    input
        .snapshot
        .domains
        .iter()
        .map(|domain| {
            relation(
                snapshot_id,
                SNAPSHOT_DOMAIN_RELATION_KIND,
                domain_object_id(&domain.id),
            )
        })
        .collect()
}

fn add_rdf_term_rows(
    input: &EpistemeOntologyRegistryReadModelInput,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    for class in &input.snapshot.rdf_terms.classes {
        push_rdf_class(class, seeds, relations);
    }
    for property in &input.snapshot.rdf_terms.object_properties {
        push_rdf_property(property, seeds, relations);
    }
}

fn push_rdf_class(
    class: &EpistemeOntologyRegistryRdfClassTerm,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = rdf_class_object_id(&class.iri);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: RDF_CLASS_OBJECT_KIND,
        title: class.label.clone(),
        source_path: class.source_file.clone(),
        evidence: vec![
            format!("domain:{}", class.domain),
            format!("iri:{}", class.iri),
            format!("api_candidate:{}", class.api_candidate),
        ],
    });
    relations.push(relation(
        &domain_object_id(&class.domain),
        DOMAIN_TERM_RELATION_KIND,
        id,
    ));
}

fn push_rdf_property(
    property: &EpistemeOntologyRegistryObjectPropertyTerm,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = rdf_property_object_id(&property.iri);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: RDF_PROPERTY_OBJECT_KIND,
        title: property.label.clone(),
        source_path: property.source_file.clone(),
        evidence: vec![
            format!("domain:{}", property.domain),
            format!("iri:{}", property.iri),
            format!("api_candidate:{}", property.api_candidate),
        ],
    });
    relations.push(relation(
        &domain_object_id(&property.domain),
        DOMAIN_TERM_RELATION_KIND,
        id,
    ));
}

fn add_rule_policy_mapping_rows(
    input: &EpistemeOntologyRegistryReadModelInput,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    for rule in &input.snapshot.rules {
        push_rule(rule, seeds, relations);
    }
    for policy in &input.snapshot.policies {
        push_policy(policy, seeds, relations);
    }
    for mapping in &input.snapshot.dataset_mappings {
        push_mapping(mapping, seeds, relations);
    }
}

fn push_rule(
    rule: &EpistemeOntologyRegistryRule,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = rule_object_id(&rule.path);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: RULE_OBJECT_KIND,
        title: rule.path.clone(),
        source_path: rule.path.clone(),
        evidence: vec![format!("kind:{}", rule.kind.as_str())],
    });
    relations.push(relation(
        &domain_object_id(&rule.domain),
        DOMAIN_RULE_RELATION_KIND,
        id,
    ));
}

fn push_policy(
    policy: &EpistemeOntologyRegistryPolicy,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = policy_object_id(&policy.path);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: POLICY_OBJECT_KIND,
        title: policy.path.clone(),
        source_path: policy.path.clone(),
        evidence: vec![format!("kind:{}", policy.kind.as_str())],
    });
    relations.push(relation(
        &domain_object_id(&policy.domain),
        DOMAIN_POLICY_RELATION_KIND,
        id,
    ));
}

fn push_mapping(
    mapping: &EpistemeOntologyRegistryDatasetMapping,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = mapping_object_id(&mapping.mapping_id);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: DATASET_MAPPING_OBJECT_KIND,
        title: mapping.mapping_id.clone(),
        source_path: mapping.path.clone(),
        evidence: vec![
            format!("kind:{}", mapping.kind.as_str()),
            format!("raw_tables:{}", mapping.raw_tables.len()),
        ],
    });
    relations.push(relation(
        &domain_object_id(&mapping.domain),
        DOMAIN_MAPPING_RELATION_KIND,
        id,
    ));
}

fn add_api_rows(
    input: &EpistemeOntologyRegistryReadModelInput,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    for object in &input.snapshot.api.objects {
        push_api_object(object, seeds, relations);
    }
    for interface in &input.snapshot.api.interfaces {
        push_api_interface(interface, seeds, relations);
    }
    for link in &input.snapshot.api.links {
        push_api_link(link, seeds, relations);
    }
    for action in &input.snapshot.api.actions {
        push_api_action(action, seeds, relations);
    }
    for query in &input.snapshot.api.queries {
        push_api_query(query, seeds, relations);
    }
}

fn push_api_object(
    object: &EpistemeOntologyRegistryObjectType,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = api_object_id(&object.api_name);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: API_OBJECT_KIND,
        title: object.api_name.clone(),
        source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
        evidence: vec![format!("rdf_class:{}", object.rdf_class)],
    });
    relations.push(relation(
        &domain_object_id(&object.domain),
        DOMAIN_API_RELATION_KIND,
        id,
    ));
}

fn push_api_interface(
    interface: &EpistemeOntologyRegistryInterfaceType,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = api_interface_id(&interface.api_name);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: API_INTERFACE_OBJECT_KIND,
        title: interface.api_name.clone(),
        source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
        evidence: vec![format!("implemented_by:{}", interface.implemented_by.len())],
    });
    for object in &interface.implemented_by {
        relations.push(relation(
            &id,
            API_INTERFACE_OBJECT_RELATION_KIND,
            api_object_id(object),
        ));
    }
}

fn push_api_link(
    link: &EpistemeOntologyRegistryLinkType,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = api_link_id(&link.api_name);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: API_LINK_OBJECT_KIND,
        title: link.api_name.clone(),
        source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
        evidence: vec![format!("rdf_property:{}", link.rdf_property)],
    });
    relations.push(relation(
        &domain_object_id(&link.domain),
        DOMAIN_API_RELATION_KIND,
        id.clone(),
    ));
    relations.push(relation(
        &id,
        API_LINK_SOURCE_RELATION_KIND,
        api_object_id(link.from_object_type.as_str()),
    ));
    relations.push(relation(
        &id,
        API_LINK_TARGET_RELATION_KIND,
        api_object_id(link.to_object_type.as_str()),
    ));
}

fn push_api_action(
    action: &EpistemeOntologyRegistryActionType,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = api_action_id(&action.api_name);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: API_ACTION_OBJECT_KIND,
        title: action.api_name.clone(),
        source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
        evidence: vec![format!("requires_evidence:{}", action.requires_evidence)],
    });
    relations.push(relation(
        &domain_object_id(&action.domain),
        DOMAIN_API_RELATION_KIND,
        id.clone(),
    ));
    for object in &action.affected_object_types {
        relations.push(relation(
            &id,
            API_ACTION_OBJECT_RELATION_KIND,
            api_object_id(object),
        ));
    }
}

fn push_api_query(
    query: &EpistemeOntologyRegistryQueryType,
    seeds: &mut Vec<RegistrySnapshotObjectSeed>,
    relations: &mut Vec<SemanticRelationRow>,
) {
    let id = api_query_id(&query.api_name);
    seeds.push(RegistrySnapshotObjectSeed {
        id: id.clone(),
        kind: API_QUERY_OBJECT_KIND,
        title: query.api_name.clone(),
        source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
        evidence: vec![format!("parameters:{}", query.parameters.len())],
    });
    relations.push(relation(
        &domain_object_id(&query.domain),
        DOMAIN_API_RELATION_KIND,
        id.clone(),
    ));
    relations.push(relation(
        &id,
        API_QUERY_OBJECT_RELATION_KIND,
        api_object_id(&query.returns),
    ));
}

fn object_rows_from_seeds(
    seeds: &[RegistrySnapshotObjectSeed],
    relations: &[SemanticRelationRow],
    source_revision: &str,
) -> Result<Vec<SemanticObjectRow>, EpistemeError> {
    let relation_counts = semantic_relation_counts(relations);
    seeds
        .iter()
        .map(|seed| {
            Ok(SemanticObjectRow {
                id: seed.id.clone(),
                kind: seed.kind,
                title: seed.title.clone(),
                status: STATUS_ACTIVE,
                confidence_score: 1.0,
                confidence_source: REGISTRY_SNAPSHOT_CONFIDENCE_SOURCE,
                owner_count: 1,
                owners_json: owners_json(REGISTRY_SNAPSHOT_CONFIDENCE_SOURCE)?,
                provenance_source: seed.source_path.clone(),
                provenance_recorded_by: RECORDED_BY,
                provenance_recorded_at: RECORDED_AT,
                verification_required_json: json_array(["ontology_registry_snapshot_admission"])?,
                verification_evidence_json: json_array(seed.evidence.iter().map(String::as_str))?,
                relation_count: i64::try_from(
                    relation_counts
                        .get(seed.id.as_str())
                        .copied()
                        .unwrap_or_default(),
                )
                .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
                source_path: seed.source_path.clone(),
                read_model_source_revision: source_revision.to_string(),
                read_model_projection_revision: REGISTRY_SNAPSHOT_PROJECTION_REVISION,
                read_model_projection_staleness: STALENESS_FRESH,
            })
        })
        .collect()
}

fn registry_snapshot_projection_rows(
    object_rows: &[SemanticObjectRow],
    source_revision: &str,
) -> Result<Vec<SemanticProjectionStateRow>, EpistemeError> {
    Ok(vec![SemanticProjectionStateRow {
        projection: REGISTRY_SNAPSHOT_PROJECTION_ID,
        status: STATUS_ACTIVE,
        source_revision: source_revision.to_string(),
        current_source_revision: source_revision.to_string(),
        projection_revision: REGISTRY_SNAPSHOT_PROJECTION_REVISION,
        staleness: STALENESS_FRESH,
        source_object_count: i64::try_from(object_rows.len())
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
        source_objects_json: json_array(object_rows.iter().map(|row| row.id.as_str()))?,
        source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
    }])
}

fn registry_snapshot_revision(
    input: &EpistemeOntologyRegistryReadModelInput,
) -> Result<String, EpistemeError> {
    let mut hasher = Sha256::new();
    let raw = serde_json::to_vec(&input.snapshot)
        .map_err(|error| EpistemeError::ReadModel(error.to_string()))?;
    hasher.update(raw);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn stamp_relation_rows(relations: &mut [SemanticRelationRow], source_revision: &str) {
    for row in relations {
        row.read_model_source_revision = source_revision.to_string();
    }
}

fn relation(source: &str, kind: &'static str, target: String) -> SemanticRelationRow {
    SemanticRelationRow {
        source: source.to_string(),
        kind,
        target,
        source_path: REGISTRY_SNAPSHOT_SOURCE_PATH.to_string(),
        read_model_source_revision: String::new(),
        read_model_projection_revision: REGISTRY_SNAPSHOT_PROJECTION_REVISION,
        read_model_projection_staleness: STALENESS_FRESH,
    }
}

fn snapshot_object_id(ontology: &str) -> String {
    format!("episteme_registry.snapshot:{ontology}")
}

fn domain_object_id(domain: &str) -> String {
    format!("episteme_registry.domain:{domain}")
}

fn rdf_class_object_id(iri: &str) -> String {
    format!("episteme_registry.rdf_class:{iri}")
}

fn rdf_property_object_id(iri: &str) -> String {
    format!("episteme_registry.rdf_object_property:{iri}")
}

fn rule_object_id(path: &str) -> String {
    format!("episteme_registry.validation_rule:{path}")
}

fn policy_object_id(path: &str) -> String {
    format!("episteme_registry.policy:{path}")
}

fn mapping_object_id(mapping_id: &str) -> String {
    format!("episteme_registry.dataset_mapping:{mapping_id}")
}

fn api_object_id(api_name: &str) -> String {
    format!("episteme_registry.api_object_type:{api_name}")
}

fn api_link_id(api_name: &str) -> String {
    format!("episteme_registry.api_link_type:{api_name}")
}

fn api_action_id(api_name: &str) -> String {
    format!("episteme_registry.api_action_type:{api_name}")
}

fn api_query_id(api_name: &str) -> String {
    format!("episteme_registry.api_query_type:{api_name}")
}

fn api_interface_id(api_name: &str) -> String {
    format!("episteme_registry.api_interface_type:{api_name}")
}
