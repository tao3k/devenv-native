use super::{
    EpistemeCommand, EpistemeGenerateOntologyCandidatesArgs,
    EpistemeImportQianjiReviewCandidatesArgs, EpistemeInspectOntologyCandidatesArgs,
    EpistemeReviewOntologyCandidatesArgs, EpistemeSourceContractCommand,
    EpistemeWriteOntologyRdfDraftArgs,
};

#[test]
fn episteme_source_contract_generate_ontology_candidates_args_capture_runs() {
    let args = EpistemeGenerateOntologyCandidatesArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        extraction_run_root: Some("episteme-repo/runs/extraction".into()),
        run_id: "ontology_seed".to_string(),
        extraction_run_ids: vec!["docling_seed".to_string(), "image_seed".to_string()],
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
    assert_eq!(
        args.extraction_run_root,
        Some(std::path::PathBuf::from("episteme-repo/runs/extraction"))
    );
    assert_eq!(args.extraction_run_ids, ["docling_seed", "image_seed"]);
}

#[test]
fn episteme_source_contract_command_debug_names_ontology_candidate_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::GenerateOntologyCandidates(
            EpistemeGenerateOntologyCandidatesArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                extraction_run_root: None,
                run_id: "ontology_seed".to_string(),
                extraction_run_ids: Vec::new(),
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("GenerateOntologyCandidates"));
}

#[test]
fn episteme_source_contract_review_ontology_candidates_args_capture_run() {
    let args = EpistemeReviewOntologyCandidatesArgs {
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
fn episteme_source_contract_command_debug_names_ontology_review_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::ReviewOntologyCandidates(
            EpistemeReviewOntologyCandidatesArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ontology_seed".to_string(),
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("ReviewOntologyCandidates"));
}

#[test]
fn episteme_source_contract_inspect_ontology_candidates_args_capture_run() {
    let args = EpistemeInspectOntologyCandidatesArgs {
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
fn episteme_source_contract_command_debug_names_ontology_inspection_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::InspectOntologyCandidates(
            EpistemeInspectOntologyCandidatesArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ontology_seed".to_string(),
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("InspectOntologyCandidates"));
}

#[test]
fn episteme_source_contract_import_qianji_review_candidates_args_capture_artifacts() {
    let args = EpistemeImportQianjiReviewCandidatesArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        run_id: "ontology_seed".to_string(),
        qianji_review_artifacts: vec![
            "episteme-repo/runs/ontology-generation/qianji/review.json".into(),
        ],
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(args.run_id, "ontology_seed");
    assert_eq!(
        args.qianji_review_artifacts,
        [std::path::PathBuf::from(
            "episteme-repo/runs/ontology-generation/qianji/review.json"
        )]
    );
}

#[test]
fn episteme_source_contract_command_debug_names_qianji_review_candidate_import_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::ImportQianjiReviewCandidates(
            EpistemeImportQianjiReviewCandidatesArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ontology_seed".to_string(),
                qianji_review_artifacts: vec!["review.json".into()],
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("ImportQianjiReviewCandidates"));
}

#[test]
fn episteme_source_contract_write_ontology_rdf_draft_args_capture_run() {
    let args = EpistemeWriteOntologyRdfDraftArgs {
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
fn episteme_source_contract_command_debug_names_ontology_rdf_draft_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteOntologyRdfDraft(
            EpistemeWriteOntologyRdfDraftArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                run_root: None,
                run_id: "ontology_seed".to_string(),
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteOntologyRdfDraft"));
}
