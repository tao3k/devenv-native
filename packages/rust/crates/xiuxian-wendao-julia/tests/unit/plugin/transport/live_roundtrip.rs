#[tokio::test]
#[serial_test::serial(julia_arrow_live)]
async fn process_julia_flight_batches_validates_remote_response() {
    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut service = spawn_real_wendaoarrow_service(port);
    let client = test_transport_client(base_url.clone(), "/rerank");

    wait_for_service_ready(&base_url)
        .await
        .unwrap_or_else(|error| panic!("wait for real WendaoArrow Flight service: {error}"));

    let result = process_julia_flight_batches(&client, &[request_batch()]).await;
    assert!(result.is_ok(), "expected valid Flight response: {result:?}");

    service.kill();
}

#[tokio::test]
#[serial_test::serial(julia_arrow_live)]
async fn process_julia_flight_batches_rejects_invalid_remote_response() {
    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut service = spawn_real_wendaoarrow_bad_response_service(port);
    let client = test_transport_client(base_url.clone(), "/rerank");

    wait_for_service_ready(&base_url)
        .await
        .unwrap_or_else(|error| {
            panic!("wait for real WendaoArrow bad-response Flight service: {error}")
        });

    let Err(error) = process_julia_flight_batches(&client, &[request_batch()]).await else {
        panic!("invalid remote response must fail");
    };
    assert!(
        error.to_string().contains("analyzer_score")
            || error.to_string().contains("Arrow Flight request failed"),
        "unexpected process error: {error}"
    );

    service.kill();
}

#[tokio::test]
#[serial_test::serial(julia_arrow_live)]
async fn process_julia_flight_batches_for_repository_builds_transport_from_repo_config() {
    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut service = spawn_real_wendaoarrow_service(port);
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "flight_transport": {
                    "base_url": base_url.clone(),
                    "route": "/rerank",
                    "health_route": "/healthz",
                    "timeout_secs": 30
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    wait_for_service_ready_with_attempts(&base_url, 600)
        .await
        .unwrap_or_else(|error| {
            panic!("wait for repo-configured WendaoArrow Flight service: {error}")
        });
    let result = process_julia_flight_batches_for_repository(&repository, &[request_batch()]).await;
    assert!(
        result.is_ok(),
        "expected repository-configured Flight transport: {result:?}"
    );

    service.kill();
}

#[tokio::test]
#[serial_test::serial(julia_arrow_live)]
async fn process_julia_flight_batches_for_repository_rejects_missing_transport() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
        ..RegisteredRepository::default()
    };

    let Err(error) =
        process_julia_flight_batches_for_repository(&repository, &[request_batch()]).await
    else {
        panic!("missing inline transport must fail");
    };
    assert!(
        error
            .to_string()
            .contains("does not declare an enabled Julia Flight transport client"),
        "unexpected repository transport error: {error}"
    );
}

#[tokio::test]
#[serial_test::serial(julia_arrow_live)]
async fn process_julia_flight_batches_against_real_wendaoarrow_service() {
    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut service = spawn_real_wendaoarrow_service(port);
    let client = test_transport_client(base_url.clone(), "/rerank");

    wait_for_service_ready(&base_url)
        .await
        .unwrap_or_else(|error| panic!("wait for real WendaoArrow Flight service: {error}"));

    let response_batches = process_julia_flight_batches(&client, &[request_batch()])
        .await
        .unwrap_or_else(|error| {
            panic!("real WendaoArrow Flight roundtrip should succeed: {error}")
        });

    assert_eq!(response_batches.len(), 1);
    let batch = &response_batches[0];
    let doc_id = string_column(batch, JULIA_ARROW_DOC_ID_COLUMN);
    let analyzer_score = float64_column(batch, JULIA_ARROW_ANALYZER_SCORE_COLUMN);
    let final_score = float64_column(batch, JULIA_ARROW_FINAL_SCORE_COLUMN);

    assert_eq!(doc_id.value(0), "doc-a");
    assert_eq!(doc_id.value(1), "doc-b");
    assert!((analyzer_score.value(0) - 0.4).abs() < f64::EPSILON);
    assert!((analyzer_score.value(1) - 0.7).abs() < f64::EPSILON);
    assert!((final_score.value(0) - 0.4).abs() < f64::EPSILON);
    assert!((final_score.value(1) - 0.7).abs() < f64::EPSILON);

    service.kill();
}

#[tokio::test]
#[serial_test::serial(julia_arrow_live)]
async fn real_wendaoarrow_metadata_example_roundtrip_decodes_trace_id_column() {
    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut service = spawn_real_wendaoarrow_metadata_service(port);
    let client = test_transport_client(base_url.clone(), "/rerank");

    wait_for_service_ready(&base_url)
        .await
        .unwrap_or_else(|error| {
            panic!("wait for real WendaoArrow metadata Flight service: {error}")
        });

    let batches =
        process_julia_flight_batches(&client, &[request_batch_with_trace_id("trace-123")])
            .await
            .unwrap_or_else(|error| panic!("metadata Flight request: {error}"));
    assert_eq!(batches.len(), 1);

    let batch = &batches[0];
    let trace_id = string_column(batch, JULIA_ARROW_TRACE_ID_COLUMN);
    assert_eq!(trace_id.value(0), "trace-123");
    assert_eq!(trace_id.value(1), "trace-123");

    service.kill();
}

#[tokio::test]
#[serial_test::serial(julia_arrow_live)]
async fn real_wendaoanalyzer_linear_blend_roundtrip_emits_expected_scores() {
    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut service = spawn_real_wendaoanalyzer_linear_blend_service(port);
    let client = test_transport_client(base_url.clone(), "/rerank");

    wait_for_service_ready_with_attempts(&base_url, 600)
        .await
        .unwrap_or_else(|error| panic!("wait for real WendaoAnalyzer Flight service: {error}"));

    let batches = process_julia_flight_batches(&client, &[request_batch()])
        .await
        .unwrap_or_else(|error| panic!("linear blend Flight request: {error}"));
    assert_eq!(batches.len(), 1);

    let batch = &batches[0];
    let doc_id = string_column(batch, JULIA_ARROW_DOC_ID_COLUMN);
    let analyzer_score = float64_column(batch, JULIA_ARROW_ANALYZER_SCORE_COLUMN);
    let final_score = float64_column(batch, JULIA_ARROW_FINAL_SCORE_COLUMN);
    let ranking_reason = string_column(batch, "ranking_reason");

    assert_eq!(doc_id.value(0), "doc-a");
    assert_eq!(doc_id.value(1), "doc-b");
    assert!((analyzer_score.value(0) - 0.928_476_63).abs() < 1e-8);
    assert!((analyzer_score.value(1) - 0.979_936_66).abs() < 1e-8);
    assert!((final_score.value(0) - 0.743_509_810_566_902_2).abs() < 1e-12);
    assert!((final_score.value(1) - 0.881_958_828_568_458_5).abs() < 1e-12);
    assert_eq!(
        ranking_reason.value(0),
        "final_score=0.35*vector_score+0.65*cosine_similarity"
    );
    assert_eq!(
        ranking_reason.value(1),
        "final_score=0.35*vector_score+0.65*cosine_similarity"
    );

    service.kill();
}
