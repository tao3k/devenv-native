use super::{
    REST_DOCS_PACK_ID, TempDir, build_contract_feedback_config, build_rest_docs_collection_context,
    must_ok, rest_docs_command, run_deterministic_rest_docs_contract_feedback,
    run_scaffold_rest_docs_contract_feedback, write_openapi_fixture,
};

#[tokio::test]
async fn deterministic_rest_docs_contract_feedback_outputs_expected_summary() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let openapi_path = write_openapi_fixture(&temp_dir);
    let workspace_root = temp_dir.path().to_path_buf();
    let command = rest_docs_command(&openapi_path, &workspace_root);

    let context = build_rest_docs_collection_context(&openapi_path, Some(workspace_root.clone()));
    let config = build_contract_feedback_config(&command);
    let advisory_roles = config
        .advisory_policy_for_pack(REST_DOCS_PACK_ID)
        .requested_roles;
    assert!(advisory_roles.is_empty());

    let output = must_ok(
        run_deterministic_rest_docs_contract_feedback(
            &command,
            &openapi_path,
            workspace_root.as_path(),
            context,
            &config,
            advisory_roles,
        )
        .await,
        "deterministic rest-docs contract feedback should succeed",
    );

    assert_eq!(
        output.report["suite_id"],
        "qianji-rest-docs-contract-feedback"
    );
    assert_eq!(output.report["stats"]["total"], 2);
    assert_eq!(output.report["stats"]["deterministic"], 2);
    assert_eq!(output.report["stats"]["advisory"], 0);
    assert_eq!(output.knowledge_entry_ids.len(), 2);
    assert!(output.persisted_entry_ids.is_empty());
    assert!(output.storage.is_none());
}

#[tokio::test]
async fn scaffold_rest_docs_contract_feedback_emits_role_advisory_findings() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let openapi_path = write_openapi_fixture(&temp_dir);
    let workspace_root = temp_dir.path().to_path_buf();
    let mut command = rest_docs_command(&openapi_path, &workspace_root);
    command.roles = vec!["strict_teacher".to_string(), "artisan-engineer".to_string()];

    let context = build_rest_docs_collection_context(&openapi_path, Some(workspace_root.clone()));
    let config = build_contract_feedback_config(&command);
    let advisory_roles = config
        .advisory_policy_for_pack(REST_DOCS_PACK_ID)
        .requested_roles;
    assert_eq!(
        advisory_roles,
        vec!["strict_teacher".to_string(), "artisan-engineer".to_string()]
    );

    let output = must_ok(
        run_scaffold_rest_docs_contract_feedback(
            &command,
            &openapi_path,
            workspace_root.as_path(),
            context,
            &config,
            advisory_roles,
        )
        .await,
        "scaffold rest-docs contract feedback should succeed",
    );

    assert_eq!(
        output.advisory_roles,
        vec!["strict_teacher".to_string(), "artisan-engineer".to_string()]
    );
    assert_eq!(output.report["stats"]["deterministic"], 2);
    assert_eq!(output.report["stats"]["advisory"], 2);
    assert_eq!(output.report["stats"]["total"], 4);
    assert_eq!(output.knowledge_entry_ids.len(), 4);
    assert!(output.persisted_entry_ids.is_empty());
    assert!(output.storage.is_none());
}
