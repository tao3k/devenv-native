use super::{
    EpistemeCommand, EpistemeSourceContractCommand, EpistemeWriteOntologyPromotionApplyPlanArgs,
    EpistemeWriteOntologyPromotionReviewArgs,
};

#[test]
fn episteme_source_contract_write_ontology_promotion_review_args_capture_run() {
    let args = EpistemeWriteOntologyPromotionReviewArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        run_id: "ontology_seed".to_string(),
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(args.run_id, "ontology_seed");
    assert_eq!(
        args.run_root,
        Some(std::path::PathBuf::from(
            "episteme-repo/runs/ontology-generation"
        ))
    );
}

#[test]
fn episteme_source_contract_command_debug_names_promotion_review_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteOntologyPromotionReview(
            EpistemeWriteOntologyPromotionReviewArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ontology_seed".to_string(),
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteOntologyPromotionReview"));
}

#[test]
fn episteme_source_contract_write_ontology_promotion_apply_plan_args_capture_run() {
    let args = EpistemeWriteOntologyPromotionApplyPlanArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        run_id: "ontology_seed".to_string(),
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(args.run_id, "ontology_seed");
    assert_eq!(
        args.run_root,
        Some(std::path::PathBuf::from(
            "episteme-repo/runs/ontology-generation"
        ))
    );
}

#[test]
fn episteme_source_contract_command_debug_names_promotion_apply_plan_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteOntologyPromotionApplyPlan(
            EpistemeWriteOntologyPromotionApplyPlanArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ontology_seed".to_string(),
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteOntologyPromotionApplyPlan"));
}
