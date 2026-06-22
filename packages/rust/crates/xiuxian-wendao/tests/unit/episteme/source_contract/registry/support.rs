use std::fs;
pub(super) use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub(super) use crate::episteme::source_contract::support::{
    EpistemeFixture, cleanup_managed_git_entry, i64_column, init_git_repository, string_column,
    table, write_registry_manifest,
};
pub(super) use xiuxian_wendao::episteme::{
    EpistemeRegistryEntry, LoadedEpistemeSourceKind,
    admit_and_materialize_episteme_ontology_registry_snapshot_read_model_seed,
    load_episteme_registry_entries,
    materialize_episteme_ontology_registry_snapshot_read_model_seed,
    materialize_episteme_registry_reference_graph_read_model_seed,
    validate_episteme_read_model_relation_endpoints, validate_episteme_registry_reference_graph,
};
pub(super) use xiuxian_wendao_episteme::{
    EpistemeOntologyRegistryReadModelInput, EpistemeOntologyRegistrySnapshot,
    EpistemeOntologyRegistrySnapshotReport,
};

pub(super) fn registry_snapshot_input()
-> Result<EpistemeOntologyRegistryReadModelInput, serde_json::Error> {
    let snapshot = serde_json::from_str::<EpistemeOntologyRegistrySnapshot>(include_str!(
        "../../../../fixtures/episteme_registry_snapshot.json"
    ))?;
    Ok(EpistemeOntologyRegistryReadModelInput {
        snapshot,
        report: EpistemeOntologyRegistrySnapshotReport {
            domains: 2,
            rdf_files: 2,
            rules: 1,
            policies: 1,
            dataset_mappings: 1,
            rdf_classes: 2,
            rdf_object_properties: 1,
            api_objects: 2,
            api_links: 1,
            api_actions: 1,
            api_queries: 1,
            api_interfaces: 1,
            reference_nouns: 2,
        },
    })
}

pub(super) fn write_registry_snapshot_fixture(root: &Path) -> std::io::Result<()> {
    let ontology = root.join("ontology");
    fs::create_dir_all(ontology.as_path())?;
    fs::write(
        ontology.join("registry.json"),
        include_str!("../../../../fixtures/episteme_registry_snapshot.json"),
    )?;
    for relative_path in [
        "manifest.toml",
        "api_surface.toml",
        "00_Core/ontology.rdf",
        "10_Domain/ontology.rdf",
        "10_Domain/rules/01_rule.sql",
        "10_Domain/policies/policy.md",
        "10_Domain/mappings/mapping.toml",
        "10_Domain/mappings/ledger.org",
        "10_Domain/mappings/sql/01_object_observations.sql",
    ] {
        let path = ontology.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, "synthetic ontology fixture\n")?;
    }
    Ok(())
}
