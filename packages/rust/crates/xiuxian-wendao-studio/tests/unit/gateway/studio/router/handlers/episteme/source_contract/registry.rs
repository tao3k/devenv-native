use std::fs;

use axum::http::StatusCode;

use super::support::EpistemeGatewayFixture;
use super::{
    EpistemeOntologyRegistryQualityProofModeRequest,
    EpistemeOntologyRegistryReadModelGatewayRequest, EpistemeRunPlanAdmissionRequest,
    admit_episteme_ontology_registry_read_model_from_payload,
    admit_episteme_ontology_registry_read_model_from_payload_with_quality_proof,
    plan_episteme_extraction_run_from_payload,
};

#[test]
fn episteme_source_contract_gateway_run_plan_accepts_registry_id()
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
    fixture.write_registry_config("synthetic", "source-contract")?;

    let report = plan_episteme_extraction_run_from_payload(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &EpistemeRunPlanAdmissionRequest {
            episteme_root: None,
            episteme_registry_id: Some("synthetic".to_string()),
            corpus_root: Some("corpus-root".to_string()),
            run_root: None,
            selection_run_id: None,
            selection_root: None,
            run_id: "gateway_registry_seed".to_string(),
            route: Some("document_text_evidence".to_string()),
            category: None,
            limit: Some(1),
        },
    )
    .unwrap_or_else(|error| panic!("Gateway registry run-plan should succeed: {error:?}"));

    assert_eq!(report.selected_count, 1);
    assert_eq!(
        report.run_dir,
        fixture
            .episteme_root
            .join("runs/extraction")
            .join("gateway_registry_seed")
    );
    Ok(())
}

#[test]
fn episteme_source_contract_gateway_registry_id_validates_reference_graph()
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
    fixture.write_contract_extending("episteme://common/domain")?;
    fixture.write_common_domain("common-episteme", "episteme://common/domain")?;
    fs::write(
        fixture.config_root.join("wendao.toml"),
        r#"[episteme.registries.common]
path = "common-episteme"

[episteme.registries.synthetic]
path = "source-contract"
"#,
    )?;

    let report = plan_episteme_extraction_run_from_payload(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &EpistemeRunPlanAdmissionRequest {
            episteme_root: None,
            episteme_registry_id: Some("synthetic".to_string()),
            corpus_root: Some("corpus-root".to_string()),
            run_root: None,
            selection_run_id: None,
            selection_root: None,
            run_id: "gateway_registry_reference_seed".to_string(),
            route: Some("document_text_evidence".to_string()),
            category: None,
            limit: Some(1),
        },
    )
    .unwrap_or_else(|error| panic!("Gateway registry graph should admit run-plan: {error:?}"));

    assert_eq!(report.selected_count, 1);
    Ok(())
}

#[test]
fn episteme_source_contract_gateway_registry_id_rejects_missing_reference()
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
    fixture.write_contract_extending("episteme://missing/domain")?;
    fixture.write_registry_config("synthetic", "source-contract")?;

    let result = plan_episteme_extraction_run_from_payload(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &EpistemeRunPlanAdmissionRequest {
            episteme_root: None,
            episteme_registry_id: Some("synthetic".to_string()),
            corpus_root: Some("corpus-root".to_string()),
            run_root: None,
            selection_run_id: None,
            selection_root: None,
            run_id: "gateway_registry_reference_seed".to_string(),
            route: Some("document_text_evidence".to_string()),
            category: None,
            limit: Some(1),
        },
    );
    let Err(error) = result else {
        panic!("missing registry reference should be rejected");
    };

    assert_eq!(error.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "EPISTEME_REGISTRY_LOAD_REJECTED");
    Ok(())
}

#[test]
fn episteme_ontology_registry_gateway_materializes_read_model_from_registry_id()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeGatewayFixture::new()?;
    fixture.write_contract()?;
    fixture.write_ontology_registry_snapshot()?;
    fixture.write_registry_config("synthetic", "source-contract")?;

    let report = admit_episteme_ontology_registry_read_model_from_payload(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &EpistemeOntologyRegistryReadModelGatewayRequest {
            episteme_root: None,
            episteme_registry_id: Some("synthetic".to_string()),
            quality_proof_mode: None,
        },
    )
    .unwrap_or_else(|error| {
        panic!("Gateway registry snapshot read-model admission should succeed: {error:?}")
    });

    assert_eq!(report.status, "admitted");
    assert!(report.source_revision.starts_with("sha256:"));
    assert_eq!(report.row_counts.objects, 4);
    assert_eq!(report.row_counts.relations, 3);
    assert_eq!(report.row_counts.projection_state, 1);
    assert_eq!(report.tables.len(), 3);
    assert_eq!(report.quality_proof, None);

    Ok(())
}

#[tokio::test]
async fn episteme_ontology_registry_gateway_quality_proof_is_explicit()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeGatewayFixture::new()?;
    fixture.write_contract()?;
    fixture.write_ontology_registry_snapshot()?;

    let request = EpistemeOntologyRegistryReadModelGatewayRequest {
        episteme_root: Some("source-contract".to_string()),
        episteme_registry_id: None,
        quality_proof_mode: Some(EpistemeOntologyRegistryQualityProofModeRequest::IfConfigured),
    };
    let report = admit_episteme_ontology_registry_read_model_from_payload_with_quality_proof(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &request,
    )
    .await
    .map_err(|error| format!("{error:?}"))?;

    let Some(proof) = report.quality_proof.as_ref() else {
        return Err("explicit proof mode should produce a proof summary".into());
    };
    assert_eq!(proof.mode, "if-configured");
    assert_eq!(proof.request_row_counts.objects, 4);
    assert_eq!(proof.request_row_counts.relations, 3);
    assert_eq!(proof.request_row_counts.projection_state, 1);
    #[cfg(feature = "julia")]
    {
        assert_eq!(proof.status, "not-configured");
        assert_eq!(proof.payload_byte_sizes.len(), 3);
        assert!(
            proof.payload_byte_sizes.iter().all(|size| *size > 0),
            "quality proof packaging should emit non-empty Arrow IPC payloads"
        );
    }
    #[cfg(not(feature = "julia"))]
    {
        assert_eq!(proof.status, "not-enabled");
        assert!(proof.payload_byte_sizes.is_empty());
    }

    Ok(())
}
