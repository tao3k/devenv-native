use std::fs;

use axum::http::StatusCode;

use super::support::EpistemeGatewayFixture;
use super::{EpistemeRunPlanAdmissionRequest, plan_episteme_extraction_run_from_payload};

#[test]
fn episteme_source_contract_gateway_run_plan_accepts_relative_paths()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeGatewayFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let report = plan_episteme_extraction_run_from_payload(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &EpistemeRunPlanAdmissionRequest {
            episteme_root: Some("source-contract".to_string()),
            episteme_registry_id: None,
            corpus_root: Some("corpus-root".to_string()),
            run_root: None,
            selection_run_id: None,
            selection_root: None,
            run_id: "gateway_seed".to_string(),
            route: Some("document_text_evidence".to_string()),
            category: None,
            limit: Some(1),
        },
    )
    .unwrap_or_else(|error| panic!("Gateway run-plan admission should succeed: {error:?}"));

    assert_eq!(report.selected_count, 1);
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert_eq!(
        report.run_dir,
        fixture
            .episteme_root
            .join("runs/extraction")
            .join("gateway_seed")
    );
    assert!(report.tasks_path.is_file());
    assert!(report.receipt_path.is_file());

    Ok(())
}

#[test]
fn episteme_source_contract_gateway_uses_episteme_toml_and_selection_run()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeGatewayFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "docs/b.docx",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_policy_category",
        "document_text_evidence",
        20,
    )?;
    fixture.write_contract()?;
    fixture.write_runtime_config()?;
    fixture.write_selection_run("selected_seed", &["episteme.file.b"])?;

    let report = plan_episteme_extraction_run_from_payload(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &EpistemeRunPlanAdmissionRequest {
            episteme_root: Some("source-contract".to_string()),
            episteme_registry_id: None,
            corpus_root: None,
            run_root: None,
            selection_run_id: Some("selected_seed".to_string()),
            selection_root: None,
            run_id: "gateway_selected_seed".to_string(),
            route: None,
            category: None,
            limit: Some(12),
        },
    )
    .unwrap_or_else(|error| panic!("Gateway selected run-plan should succeed: {error:?}"));

    assert_eq!(report.selected_count, 1);
    assert_eq!(report.validation_mode, "contract_shape_only");
    assert_eq!(
        report.run_dir,
        fixture
            .episteme_root
            .join("configured-runs/extraction")
            .join("gateway_selected_seed")
    );
    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(receipt.as_str())?;
    assert_eq!(
        receipt["selected_file_ids"],
        serde_json::json!(["episteme.file.b"])
    );
    assert_eq!(receipt["tasks"][0]["file_id"], "episteme.file.b");
    assert_eq!(receipt["validation_mode"], "contract_shape_only");

    Ok(())
}

#[test]
fn episteme_source_contract_gateway_run_plan_rejects_missing_run_id()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeGatewayFixture::new()?;

    let result = plan_episteme_extraction_run_from_payload(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &EpistemeRunPlanAdmissionRequest {
            episteme_root: Some("source-contract".to_string()),
            episteme_registry_id: None,
            corpus_root: Some("corpus-root".to_string()),
            run_root: None,
            selection_run_id: None,
            selection_root: None,
            run_id: "  ".to_string(),
            route: None,
            category: None,
            limit: None,
        },
    );
    let Err(error) = result else {
        panic!("missing run id should be rejected");
    };

    assert_eq!(error.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_RUN_ID");

    Ok(())
}
