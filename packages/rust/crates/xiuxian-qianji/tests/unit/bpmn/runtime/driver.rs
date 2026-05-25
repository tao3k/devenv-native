use super::{
    Arc, BpmnAdvanceOutcome, BpmnInstanceInit, BpmnOrchestrationError, BpmnPackage,
    DmnEvaluationResult, ProcessKey, QianjiBpmnExecutionDriver, QianjiBpmnExecutionRequest,
    QianjiBpmnHostBridge, QianjiBpmnSession, TempDir, err_of, json, load_bpmn_package_from_files,
    ok_of, write_business_rule_bundle,
};

#[tokio::test(flavor = "current_thread")]
async fn execution_driver_runs_fresh_session_without_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bundle = write_business_rule_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bundle.bpmn_path, std::slice::from_ref(&bundle.dmn_path)),
        "bundle should load from disk",
    );
    let driver = QianjiBpmnExecutionDriver::new(Arc::clone(&package), None);
    let host = QianjiBpmnHostBridge::builder()
        .on_business_rule_task(|request| async move {
            Ok(xiuxian_qianji_bpmn_engine::BusinessRuleTaskOutcome {
                evaluation: DmnEvaluationResult::new(
                    request.evaluation.decision.decision_id.as_ref(),
                    json!({ "approved": true, "path": "auto_approved" }),
                    vec![Arc::<str>::from("rule_host")],
                ),
            })
        })
        .clock(|| 100)
        .build();

    let execution = ok_of(
        driver
            .run_until_stable(
                &QianjiBpmnExecutionRequest::new(
                    "review",
                    "wf_driver_fresh",
                    Some(json!({ "risk": "low" })),
                    17,
                ),
                &host,
            )
            .await,
        "fresh driver execution should complete",
    );

    assert_eq!(execution.outcome, BpmnAdvanceOutcome::Completed);
    assert!(!execution.resumed_from_checkpoint);
    assert!(!execution.checkpoint_saved);
    assert!(!execution.checkpoint_deleted);
    assert_eq!(
        execution.session.instance().variables,
        json!({
            "risk": "low",
            "approved": true,
            "path": "auto_approved",
        })
    );
}

#[test]
fn execution_driver_requires_fresh_context_when_checkpoint_is_missing() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_review",
        vec![xiuxian_qianji_bpmn_engine::BpmnProcessSpec::new(
            ProcessKey::new("pkg_review", "review", "digest_review"),
            vec![
                xiuxian_qianji_bpmn_engine::BpmnNodeSpec::new(
                    0,
                    "start",
                    xiuxian_qianji_bpmn_engine::BpmnNodeKind::StartEvent,
                ),
                xiuxian_qianji_bpmn_engine::BpmnNodeSpec::new(
                    1,
                    "end",
                    xiuxian_qianji_bpmn_engine::BpmnNodeKind::EndEvent,
                ),
            ],
            vec![xiuxian_qianji_bpmn_engine::BpmnEdgeSpec::new(
                0,
                1,
                None::<&str>,
            )],
            Vec::new(),
        )],
    ));
    let driver = QianjiBpmnExecutionDriver::new(Arc::clone(&package), None);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|error| panic!("runtime should build: {error}"));

    let error = err_of(runtime.block_on(async {
        driver
            .run_until_stable(
                &QianjiBpmnExecutionRequest::new("review", "wf_missing_context", None, 5),
                &QianjiBpmnHostBridge::default(),
            )
            .await
    }));

    match error {
        BpmnOrchestrationError::FreshContextRequired {
            process_id,
            instance_id,
        } => {
            assert_eq!(process_id, "review");
            assert_eq!(instance_id, "wf_missing_context");
        }
        other => panic!("expected fresh-context error, got {other:?}"),
    }
}

#[test]
fn resume_from_checkpoint_rejects_process_identity_drift() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_review",
        vec![xiuxian_qianji_bpmn_engine::BpmnProcessSpec::new(
            ProcessKey::new("pkg_review", "review", "digest_review"),
            vec![
                xiuxian_qianji_bpmn_engine::BpmnNodeSpec::new(
                    0,
                    "start",
                    xiuxian_qianji_bpmn_engine::BpmnNodeKind::StartEvent,
                ),
                xiuxian_qianji_bpmn_engine::BpmnNodeSpec::new(
                    1,
                    "end",
                    xiuxian_qianji_bpmn_engine::BpmnNodeKind::EndEvent,
                ),
            ],
            vec![xiuxian_qianji_bpmn_engine::BpmnEdgeSpec::new(
                0,
                1,
                None::<&str>,
            )],
            Vec::new(),
        )],
    ));
    let mut session = ok_of(
        QianjiBpmnSession::new(
            Arc::clone(&package),
            "review",
            BpmnInstanceInit::new("wf_resume", json!({}), 7),
        ),
        "session should be created",
    );
    session.instance_mut().process.spec_digest_hex = Arc::<str>::from("drifted_digest");
    let checkpoint = session.checkpoint();

    let error = err_of(QianjiBpmnSession::from_checkpoint(
        Arc::clone(&package),
        checkpoint,
    ));
    match error {
        BpmnOrchestrationError::CheckpointProcessIdentityDrift {
            process_id,
            checkpoint_spec_digest,
            loaded_spec_digest,
            ..
        } => {
            assert_eq!(process_id, "review");
            assert_eq!(checkpoint_spec_digest, "drifted_digest");
            assert_eq!(loaded_spec_digest, "digest_review");
        }
        other => panic!("expected checkpoint identity drift, got {other:?}"),
    }
}
