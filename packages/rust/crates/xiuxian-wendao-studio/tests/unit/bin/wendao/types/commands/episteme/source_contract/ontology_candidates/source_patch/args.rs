use std::path::PathBuf;

use super::{
    EpistemeApplyOntologySourcePatchArgs, EpistemeWriteOntologySourcePatchApplyPlanArgs,
    EpistemeWriteOntologySourcePatchApplyPreviewArgs, EpistemeWriteOntologySourcePatchDraftArgs,
    EpistemeWriteOntologySourcePatchPreflightArgs,
    EpistemeWriteOntologySourcePatchRdfReadModelArgs,
    EpistemeWriteOntologySourcePatchReviewPacketArgs,
    EpistemeWriteOntologySourcePatchSemanticPreviewArgs,
};

#[test]
fn episteme_source_contract_write_ontology_source_patch_preflight_args_capture_run() {
    let args = EpistemeWriteOntologySourcePatchPreflightArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/source-patch-preflight".into()),
        run_id: "ltc_preflight_seed".to_string(),
    };

    assert_eq!(args.episteme_root, PathBuf::from("episteme-repo"));
    assert_eq!(args.run_id, "ltc_preflight_seed");
    assert_eq!(
        args.run_root,
        Some(PathBuf::from("episteme-repo/runs/source-patch-preflight"))
    );
}

#[test]
fn episteme_source_contract_write_ontology_source_patch_draft_args_capture_run() {
    let args = EpistemeWriteOntologySourcePatchDraftArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/source-patch-preflight".into()),
        run_id: "ltc_preflight_seed".to_string(),
    };

    assert_eq!(args.episteme_root, PathBuf::from("episteme-repo"));
    assert_eq!(args.run_id, "ltc_preflight_seed");
    assert_eq!(
        args.run_root,
        Some(PathBuf::from("episteme-repo/runs/source-patch-preflight"))
    );
}

#[test]
fn episteme_source_contract_write_ontology_source_patch_apply_plan_args_capture_run() {
    let args = EpistemeWriteOntologySourcePatchApplyPlanArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/source-patch-preflight".into()),
        run_id: "ltc_preflight_seed".to_string(),
    };

    assert_eq!(args.episteme_root, PathBuf::from("episteme-repo"));
    assert_eq!(args.run_id, "ltc_preflight_seed");
    assert_eq!(
        args.run_root,
        Some(PathBuf::from("episteme-repo/runs/source-patch-preflight"))
    );
}

#[test]
fn episteme_source_contract_write_ontology_source_patch_review_packet_args_capture_run() {
    let args = EpistemeWriteOntologySourcePatchReviewPacketArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/source-patch-preflight".into()),
        run_id: "ltc_preflight_seed".to_string(),
    };

    assert_eq!(args.episteme_root, PathBuf::from("episteme-repo"));
    assert_eq!(args.run_id, "ltc_preflight_seed");
    assert_eq!(
        args.run_root,
        Some(PathBuf::from("episteme-repo/runs/source-patch-preflight"))
    );
}

#[test]
fn episteme_source_contract_write_ontology_source_patch_apply_preview_args_capture_gate() {
    let args = EpistemeWriteOntologySourcePatchApplyPreviewArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/source-patch-preflight".into()),
        run_id: "ltc_preflight_seed".to_string(),
        expected_apply_plan_tsv_sha256: "abc123".to_string(),
    };

    assert_eq!(args.episteme_root, PathBuf::from("episteme-repo"));
    assert_eq!(args.run_id, "ltc_preflight_seed");
    assert_eq!(args.expected_apply_plan_tsv_sha256, "abc123");
}

#[test]
fn episteme_source_contract_write_ontology_source_patch_semantic_preview_args_capture_run() {
    let args = EpistemeWriteOntologySourcePatchSemanticPreviewArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/source-patch-preflight".into()),
        run_id: "ltc_preflight_seed".to_string(),
    };

    assert_eq!(args.episteme_root, PathBuf::from("episteme-repo"));
    assert_eq!(args.run_id, "ltc_preflight_seed");
    assert_eq!(
        args.run_root,
        Some(PathBuf::from("episteme-repo/runs/source-patch-preflight"))
    );
}

#[test]
fn episteme_source_contract_write_ontology_source_patch_rdf_read_model_args_capture_run() {
    let args = EpistemeWriteOntologySourcePatchRdfReadModelArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/source-patch-preflight".into()),
        run_id: "ltc_preflight_seed".to_string(),
    };

    assert_eq!(args.episteme_root, PathBuf::from("episteme-repo"));
    assert_eq!(args.run_id, "ltc_preflight_seed");
    assert_eq!(
        args.run_root,
        Some(PathBuf::from("episteme-repo/runs/source-patch-preflight"))
    );
}

#[test]
fn episteme_source_contract_apply_ontology_source_patch_args_capture_gate() {
    let args = EpistemeApplyOntologySourcePatchArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/source-patch-preflight".into()),
        run_id: "ltc_preflight_seed".to_string(),
        expected_apply_plan_tsv_sha256: "abc123".to_string(),
        allow_source_mutation: true,
    };

    assert_eq!(args.episteme_root, PathBuf::from("episteme-repo"));
    assert_eq!(args.run_id, "ltc_preflight_seed");
    assert_eq!(args.expected_apply_plan_tsv_sha256, "abc123");
    assert!(args.allow_source_mutation);
}
