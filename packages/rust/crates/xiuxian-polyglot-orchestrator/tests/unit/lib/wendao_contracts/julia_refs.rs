use crate::wendao_contracts::{
    WendaoSearchLegacyRerankProfileRefInput, julia_graph_compute_profile_refs,
    memory_julia_compute_profile_refs,
};
use crate::{
    ContractOwner, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
    WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
};
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

#[test]
fn wendao_contracts_project_julia_profile_refs_without_wendao_julia_crate() {
    let config = MemoryJuliaComputeRuntimeConfig {
        schema_version: "v7".to_string(),
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    let memory_refs = memory_julia_compute_profile_refs(&config);
    let graph_refs = julia_graph_compute_profile_refs(WendaoSearchLegacyRerankProfileRefInput {
        route: Some("/custom/rerank"),
        schema_version: Some("v2"),
    });

    assert_eq!(memory_refs.len(), 4);
    assert!(
        memory_refs
            .iter()
            .all(|reference| reference.owner == ContractOwner::Julia)
    );
    assert!(
        memory_refs
            .iter()
            .all(|reference| reference.schema_version.as_deref() == Some("v7"))
    );
    assert_eq!(graph_refs.len(), 6);
    assert!(graph_refs.iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID)
    ));
    assert!(graph_refs.iter().any(|reference| {
        reference.profile.as_deref() == Some(WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID)
            && reference.route == "/custom/rerank"
            && reference.schema_version.as_deref() == Some("v2")
    }));
    assert!(graph_refs.iter().any(|reference| {
        reference.profile.as_deref() == Some(WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID)
    }));
}
