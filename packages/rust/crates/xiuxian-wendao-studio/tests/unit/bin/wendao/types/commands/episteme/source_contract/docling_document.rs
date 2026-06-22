use super::{EpistemeCommand, EpistemeRunDoclingDocumentCacheArgs, EpistemeSourceContractCommand};

#[test]
fn episteme_source_contract_run_docling_document_cache_args_capture_commands() {
    let args = EpistemeRunDoclingDocumentCacheArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        corpus_root: Some("corpus-root".into()),
        run_root: Some("episteme-repo/runs/extraction".into()),
        run_id: "docling_document_seed".to_string(),
        category: Some("policy".to_string()),
        limit: 4,
        selection_run_id: Some("selection_seed".to_string()),
        selection_root: Some("episteme-repo/runs/evidence-selection".into()),
        analyzer_command: "wendao-docling-document-jsonl".to_string(),
        docling_profile: "full".to_string(),
        document_results_jsonl: Some(
            "episteme-repo/runs/extraction/docling_document_seed/document_results.jsonl".into(),
        ),
        use_existing_results: true,
        dry_run: true,
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(args.run_id, "docling_document_seed");
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.limit, 4);
    assert_eq!(args.selection_run_id.as_deref(), Some("selection_seed"));
    assert_eq!(args.analyzer_command, "wendao-docling-document-jsonl");
    assert_eq!(args.docling_profile, "full");
    assert!(args.use_existing_results);
    assert!(args.dry_run);
}

#[test]
fn episteme_source_contract_command_debug_names_docling_document_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::RunDoclingDocumentCache(
            EpistemeRunDoclingDocumentCacheArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                corpus_root: None,
                run_root: None,
                run_id: "docling_document_seed".to_string(),
                category: None,
                limit: 12,
                selection_run_id: None,
                selection_root: None,
                analyzer_command: "wendao-docling-document-jsonl".to_string(),
                docling_profile: "full".to_string(),
                document_results_jsonl: None,
                use_existing_results: false,
                dry_run: false,
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("RunDoclingDocumentCache"));
}
