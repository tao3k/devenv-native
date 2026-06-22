use super::episteme_bootstrap_artifact_cache_options;
use crate::bin_support::wendao::types::{
    EpistemeBootstrapArtifactCacheModeArg, EpistemeBootstrapPipelineArgs,
};

fn bootstrap_args() -> EpistemeBootstrapPipelineArgs {
    EpistemeBootstrapPipelineArgs {
        episteme_root: ".".into(),
        episteme_registry_id: None,
        corpus_root: None,
        structure_run_root: None,
        ontology_generation_run_root: None,
        validation_mode: Default::default(),
        run_id: "bootstrap_seed".to_string(),
        category: None,
        route: None,
        reasoning_packet_limit: 256,
        reasoning_ledger_seed_limit: 512,
        reasoning_fill_plan_limit: 1024,
        artifact_cache_mode: EpistemeBootstrapArtifactCacheModeArg::ReadThrough,
        artifact_cache_source_digest: Some("source-contract-v1".to_string()),
        artifact_cache_profile_digest: Some("bootstrap-v1".to_string()),
    }
}

#[test]
fn bootstrap_artifact_cache_options_accept_safe_digest_components() {
    let args = bootstrap_args();

    let options = episteme_bootstrap_artifact_cache_options(&args)
        .unwrap_or_else(|error| panic!("safe digest components should pass: {error:#}"));

    assert_eq!(options.source_digest, "source-contract-v1");
    assert_eq!(options.profile_digest, "bootstrap-v1");
}

#[test]
fn bootstrap_artifact_cache_options_reject_path_like_digest_components() {
    let mut args = bootstrap_args();
    args.artifact_cache_source_digest = Some("../source".to_string());

    let Err(error) = episteme_bootstrap_artifact_cache_options(&args) else {
        panic!("path-like source digest should be rejected");
    };
    let error = format!("{error:#}");

    assert!(error.contains("invalid Episteme bootstrap artifact-cache digest component"));
    assert!(error.contains("source_digest"));
}
