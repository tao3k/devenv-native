use super::{
    EpistemeApplyOntologySourcePatchArgs, EpistemeCommand, EpistemeSourceContractCommand,
    EpistemeWriteOntologySourcePatchApplyPlanArgs,
    EpistemeWriteOntologySourcePatchApplyPreviewArgs, EpistemeWriteOntologySourcePatchDraftArgs,
    EpistemeWriteOntologySourcePatchPreflightArgs,
    EpistemeWriteOntologySourcePatchRdfReadModelArgs,
    EpistemeWriteOntologySourcePatchReviewPacketArgs,
    EpistemeWriteOntologySourcePatchSemanticPreviewArgs,
};

#[test]
fn episteme_source_contract_command_debug_names_source_patch_preflight_variant() {
    assert_source_patch_debug_name(
        EpistemeSourceContractCommand::WriteOntologySourcePatchPreflight(
            EpistemeWriteOntologySourcePatchPreflightArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ltc_preflight_seed".to_string(),
            },
        ),
        "WriteOntologySourcePatchPreflight",
    );
}

#[test]
fn episteme_source_contract_command_debug_names_source_patch_draft_variant() {
    assert_source_patch_debug_name(
        EpistemeSourceContractCommand::WriteOntologySourcePatchDraft(
            EpistemeWriteOntologySourcePatchDraftArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ltc_preflight_seed".to_string(),
            },
        ),
        "WriteOntologySourcePatchDraft",
    );
}

#[test]
fn episteme_source_contract_command_debug_names_source_patch_apply_plan_variant() {
    assert_source_patch_debug_name(
        EpistemeSourceContractCommand::WriteOntologySourcePatchApplyPlan(
            EpistemeWriteOntologySourcePatchApplyPlanArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ltc_preflight_seed".to_string(),
            },
        ),
        "WriteOntologySourcePatchApplyPlan",
    );
}

#[test]
fn episteme_source_contract_command_debug_names_source_patch_review_packet_variant() {
    assert_source_patch_debug_name(
        EpistemeSourceContractCommand::WriteOntologySourcePatchReviewPacket(
            EpistemeWriteOntologySourcePatchReviewPacketArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ltc_preflight_seed".to_string(),
            },
        ),
        "WriteOntologySourcePatchReviewPacket",
    );
}

#[test]
fn episteme_source_contract_command_debug_names_source_patch_apply_preview_variant() {
    assert_source_patch_debug_name(
        EpistemeSourceContractCommand::WriteOntologySourcePatchApplyPreview(
            EpistemeWriteOntologySourcePatchApplyPreviewArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ltc_preflight_seed".to_string(),
                expected_apply_plan_tsv_sha256: "abc123".to_string(),
            },
        ),
        "WriteOntologySourcePatchApplyPreview",
    );
}

#[test]
fn episteme_source_contract_command_debug_names_source_patch_semantic_preview_variant() {
    assert_source_patch_debug_name(
        EpistemeSourceContractCommand::WriteOntologySourcePatchSemanticPreview(
            EpistemeWriteOntologySourcePatchSemanticPreviewArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ltc_preflight_seed".to_string(),
            },
        ),
        "WriteOntologySourcePatchSemanticPreview",
    );
}

#[test]
fn episteme_source_contract_command_debug_names_source_patch_rdf_read_model_variant() {
    assert_source_patch_debug_name(
        EpistemeSourceContractCommand::WriteOntologySourcePatchRdfReadModel(
            EpistemeWriteOntologySourcePatchRdfReadModelArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ltc_preflight_seed".to_string(),
            },
        ),
        "WriteOntologySourcePatchRdfReadModel",
    );
}

#[test]
fn episteme_source_contract_command_debug_names_apply_ontology_source_patch_variant() {
    assert_source_patch_debug_name(
        EpistemeSourceContractCommand::ApplyOntologySourcePatch(
            EpistemeApplyOntologySourcePatchArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ltc_preflight_seed".to_string(),
                expected_apply_plan_tsv_sha256: "abc123".to_string(),
                allow_source_mutation: false,
            },
        ),
        "ApplyOntologySourcePatch",
    );
}

fn assert_source_patch_debug_name(command: EpistemeSourceContractCommand, variant_name: &str) {
    let command = EpistemeCommand::SourceContract { command };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains(variant_name));
}
