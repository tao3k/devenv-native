use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(feature = "julia")]
use std::io::Cursor;
#[cfg(feature = "julia")]
use std::{
    collections::BTreeMap,
    env,
    net::TcpListener,
    process::{Child, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "julia")]
use arrow::array::Array;
use arrow::array::{Int64Array, StringArray};
#[cfg(feature = "julia")]
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;
use sha2::{Digest, Sha256};
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::EpistemeValidationHashCacheReport;
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::build_episteme_wendaograph_quality_request_batches;
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::configured_episteme_corpus_root_env;
use xiuxian_wendao::episteme::{
    EpistemeEvidenceByteSizeStatus, EpistemeEvidenceReadRequest,
    EpistemeEvidenceReadValidationMode, EpistemeEvidenceSelectionPlanRequest,
    EpistemeEvidenceSelectionValidationMode, EpistemeEvidenceSha256Status,
    EpistemeEvidenceSourceAvailability, EpistemeReadModelMaterialization, EpistemeReadModelRequest,
    EpistemeRegistryEntry, EpistemeRunPlanRequest, EpistemeStructureTocRequest,
    EpistemeStructureTocValidationMode, LoadedEpistemeSourceKind, load_episteme_registry_entries,
    load_episteme_runtime_config, materialize_episteme_read_model_seed,
    materialize_episteme_read_model_seed_with_validation_hash_cache,
    materialize_episteme_registry_reference_graph_read_model_seed, plan_episteme_extraction_run,
    read_episteme_evidence, read_episteme_evidence_selection_file_ids,
    validate_episteme_read_model_relation_endpoints, validate_episteme_registry_reference_graph,
    validate_episteme_source_contract, validate_episteme_source_contract_with_hash_cache,
    write_episteme_evidence_selection_plan, write_episteme_extraction_run_plan,
    write_episteme_structure_toc,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_core::{capabilities::PluginCapabilityBinding, transport::PluginTransportKind};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::{
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    WendaoGraphOntologyReadModelQualityRequestBatches,
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
};

#[cfg(feature = "julia")]
const RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_ENV: &str =
    "RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_TEST";
#[cfg(feature = "julia")]
const EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_REPEAT_ENV: &str =
    "WENDAO_EPISTEME_SOURCE_CONTRACT_QUALITY_REPEATS";
#[cfg(feature = "julia")]
const EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_PREWARM_ENV: &str =
    "WENDAO_EPISTEME_SOURCE_CONTRACT_QUALITY_PREWARM_ROUNDS";
#[cfg(feature = "julia")]
const EPISTEME_SOURCE_CONTRACT_VALIDATION_HASH_CACHE_PATH_ENV: &str =
    "WENDAO_EPISTEME_SOURCE_CONTRACT_VALIDATION_HASH_CACHE_PATH";
#[cfg(feature = "julia")]
const EPISTEME_SOURCE_CONTRACT_ROOT_ENV: &str = "WENDAO_EPISTEME_SOURCE_CONTRACT_ROOT";
#[cfg(feature = "julia")]
const EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV: &str =
    "WENDAO_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL";

#[test]
fn episteme_source_contract_validates_and_plans_seed_run() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;
    assert!(report.passed, "{:?}", report.errors);
    assert_eq!(report.files_tsv_rows, 2);
    assert_eq!(report.extraction_queue_rows, 2);
    assert_eq!(report.mapping_ledger_sections, 1);
    assert_eq!(report.mapping_ledger_reasoning_property_records, 1);
    assert!(!report.raw_to_rdf_promotion_allowed);

    let receipt = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "source_contract_seed",
        )
        .with_route("document_text_evidence")
        .with_limit(1),
    )?;
    assert_eq!(receipt.selected_count, 1);
    assert!(!receipt.extraction_executed);
    assert_eq!(receipt.tasks[0].queue_id, "episteme.extract.a");
    assert_eq!(
        receipt.tasks[0].planned_output_path,
        "outputs/episteme.extract.a.json"
    );
    assert_eq!(receipt.route_counts.get("document_text_evidence"), Some(&1));

    Ok(())
}

#[test]
fn episteme_runtime_config_resolves_relative_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fs::write(
        fixture.episteme_root.join("episteme.toml"),
        r#"schema_version = 1

[runtime]
corpus_root = "../corpus-root"
structure_run_root = "runs/structure"
evidence_selection_run_root = "runs/evidence-selection"
extraction_run_root = "runs/extraction"
"#,
    )?;

    let Some(config) = load_episteme_runtime_config(&fixture.episteme_root)? else {
        return Err("expected episteme runtime config".into());
    };
    assert_eq!(config.corpus, Some(fixture.corpus_root.clone()));
    assert_eq!(
        config.structure_runs,
        Some(fixture.episteme_root.join("runs/structure"))
    );
    assert_eq!(
        config.evidence_selection_runs,
        Some(fixture.episteme_root.join("runs/evidence-selection"))
    );
    assert_eq!(
        config.extraction_runs,
        Some(fixture.episteme_root.join("runs/extraction"))
    );

    Ok(())
}

#[test]
fn episteme_source_contract_writes_deterministic_run_plan() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let request = EpistemeRunPlanRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "source_contract_seed",
    )
    .with_route("document_text_evidence")
    .with_limit(1);
    let run_root = fixture.episteme_root.join("runs/extraction");

    let report = write_episteme_extraction_run_plan(&request, &run_root)?;
    assert_eq!(report.selected_count, 1);
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert!(report.outputs_dir.is_dir());
    assert_eq!(report.run_dir, run_root.join("source_contract_seed"));

    let tasks = fs::read_to_string(&report.tasks_path)?;
    assert!(tasks.starts_with(
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\tsource_sha256\tplanned_output_path\toutput_contract\tstatus\n"
    ));
    assert!(tasks.contains(
        "episteme.extract.a\tepisteme.file.a\tdocs/a.docx\tsynthetic_policy_category\tzh-CN\tdocument_text_evidence\t10"
    ));
    assert!(
        tasks.contains("\toutputs/episteme.extract.a.json\tcache_only_no_rdf_promotion\tplanned\n")
    );

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(receipt["run_id"], "source_contract_seed");
    assert_eq!(receipt["selected_count"], 1);
    assert_eq!(receipt["extraction_executed"], false);
    assert_eq!(receipt["raw_to_rdf_promotion_allowed"], false);
    assert_eq!(receipt["tasks"][0]["queue_id"], "episteme.extract.a");

    fs::write(&report.tasks_path, "stale")?;
    let rewritten = write_episteme_extraction_run_plan(&request, &run_root)?;
    assert_ne!(fs::read_to_string(rewritten.tasks_path)?, "stale");

    Ok(())
}

#[test]
fn episteme_structure_toc_writes_org_ledger_without_raw_text()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let report = write_episteme_structure_toc(
        &EpistemeStructureTocRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "structure_seed",
        ),
        fixture.episteme_root.join("runs/structure"),
    )?;

    assert_eq!(report.run_id, "structure_seed");
    assert_eq!(report.file_count, 2);
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert_eq!(
        report.validation_mode,
        EpistemeStructureTocValidationMode::MetadataOnly
    );
    assert!(report.toc_org_path.is_file());
    assert!(report.receipt_path.is_file());

    let toc = fs::read_to_string(&report.toc_org_path)?;
    assert!(toc.contains(":WENDAO_KIND: episteme_structure_toc"));
    assert!(toc.contains(":ONTOLOGY_KIND: source_structure_toc"));
    assert!(toc.contains("episteme.file.a"));
    assert!(toc.contains("docs/a.docx"));
    assert!(toc.contains("document_text_evidence"));
    assert!(
        !toc.contains("fixture content"),
        "TOC ledger must not embed raw source corpus text"
    );

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(receipt["runId"], "structure_seed");
    assert_eq!(receipt["fileCount"], 2);
    assert_eq!(
        receipt["routeCounts"]["document_text_evidence"],
        serde_json::json!(1)
    );
    assert_eq!(receipt["rawToRdfPromotionAllowed"], false);
    assert_eq!(receipt["validationMode"], "metadata-only");

    Ok(())
}

#[test]
fn episteme_structure_toc_full_hash_rejects_hash_drift() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let source_path = fixture.corpus_root.join("docs/a.docx");
    let original = fs::read_to_string(&source_path)?;
    fs::write(&source_path, "x".repeat(original.len()))?;

    let metadata_report = write_episteme_structure_toc(
        &EpistemeStructureTocRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "structure_metadata_seed",
        ),
        fixture.episteme_root.join("runs/structure"),
    )?;
    assert_eq!(
        metadata_report.validation_mode,
        EpistemeStructureTocValidationMode::MetadataOnly
    );

    let Err(error) = write_episteme_structure_toc(
        &EpistemeStructureTocRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "structure_full_hash_seed",
        )
        .with_validation_mode(EpistemeStructureTocValidationMode::FullHash),
        fixture.episteme_root.join("runs/structure"),
    ) else {
        return Err("full-hash mode must reject same-size content drift".into());
    };
    assert!(error.to_string().contains("sha256 drift"));

    Ok(())
}

#[test]
fn episteme_evidence_read_returns_bounded_text_preview() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let report = read_episteme_evidence(
        &EpistemeEvidenceReadRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "episteme.file.a",
        )
        .with_max_preview_bytes(12),
    )?;

    assert_eq!(report.source.file_id, "episteme.file.a");
    assert_eq!(report.source.relative_path, "docs/a.txt");
    assert_eq!(report.source.extraction_route, "document_text_evidence");
    assert_eq!(report.preview_kind, "plain-text");
    assert_eq!(
        report.source_availability,
        EpistemeEvidenceSourceAvailability::Available
    );
    assert_eq!(
        report.byte_size_status,
        EpistemeEvidenceByteSizeStatus::Matches
    );
    assert_eq!(
        report.sha256_status,
        EpistemeEvidenceSha256Status::NotChecked
    );
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    let Some(preview) = report.text_preview else {
        return Err("expected text preview".into());
    };
    assert_eq!(preview.text, "fixture cont");
    assert_eq!(preview.byte_count, 12);
    assert!(preview.truncated);

    Ok(())
}

#[test]
fn episteme_evidence_read_binary_source_returns_reference_without_preview()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let report = read_episteme_evidence(&EpistemeEvidenceReadRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "episteme.file.b",
    ))?;

    assert_eq!(report.source.file_id, "episteme.file.b");
    assert_eq!(report.source.relative_path, "images/b.jpg");
    assert_eq!(report.preview_kind, "unsupported-binary");
    assert!(report.text_preview.is_none());
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);

    Ok(())
}

#[test]
fn episteme_evidence_read_full_hash_rejects_hash_drift() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let source_path = fixture.corpus_root.join("docs/a.txt");
    let original = fs::read_to_string(&source_path)?;
    fs::write(&source_path, "x".repeat(original.len()))?;

    let Err(error) = read_episteme_evidence(
        &EpistemeEvidenceReadRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "episteme.file.a",
        )
        .with_validation_mode(EpistemeEvidenceReadValidationMode::FullHash),
    ) else {
        return Err("full-hash evidence read must reject same-size content drift".into());
    };
    assert!(error.to_string().contains("sha256 drift"));

    Ok(())
}

#[test]
fn episteme_evidence_read_rejects_unknown_file_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let Err(error) = read_episteme_evidence(&EpistemeEvidenceReadRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "episteme.file.missing",
    )) else {
        return Err("unknown file id must fail".into());
    };
    assert!(
        error
            .to_string()
            .contains("unknown source-contract file_id")
    );

    Ok(())
}

#[test]
fn episteme_evidence_selection_writes_org_tsv_and_receipt() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let report = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_seed",
            vec!["episteme.file.b".to_string(), "episteme.file.a".to_string()],
        )
        .with_selection_reason("agent selected table and policy evidence"),
        fixture.episteme_root.join("runs/evidence-selection"),
    )?;

    assert_eq!(report.run_id, "selection_seed");
    assert_eq!(report.selected_count, 2);
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert_eq!(
        report.validation_mode,
        EpistemeEvidenceSelectionValidationMode::MetadataOnly
    );
    assert!(report.selection_org_path.is_file());
    assert!(report.selection_tsv_path.is_file());
    assert!(report.receipt_path.is_file());

    let org = fs::read_to_string(&report.selection_org_path)?;
    assert!(org.contains(":WENDAO_KIND: episteme_evidence_selection"));
    assert!(org.contains(":ONTOLOGY_KIND: source_evidence_selection"));
    assert!(org.contains("episteme.file.b"));
    assert!(org.contains("agent selected table and policy evidence"));
    assert!(org.contains("extractor:image_ocr_evidence"));
    assert!(
        !org.contains("fixture content"),
        "selection ledger must not embed raw source corpus text"
    );

    let tsv = fs::read_to_string(&report.selection_tsv_path)?;
    assert!(tsv.starts_with(
        "selection_index\tfile_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\tselection_reason\tnext_action\n"
    ));
    assert!(tsv.contains("1\tepisteme.file.b\timages/b.jpg\tjpg\t"));
    assert!(tsv.contains("\timage_ocr_evidence\tagent selected table and policy evidence\textractor:image_ocr_evidence\n"));

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(receipt["runId"], "selection_seed");
    assert_eq!(receipt["selectedCount"], 2);
    assert_eq!(receipt["sourceFileCount"], 2);
    assert_eq!(receipt["rawToRdfPromotionAllowed"], false);
    assert_eq!(receipt["extractionExecuted"], false);
    assert_eq!(receipt["routeCounts"]["image_ocr_evidence"], 1);
    assert_eq!(receipt["selections"][0]["fileId"], "episteme.file.b");
    assert_eq!(receipt["selections"][1]["fileId"], "episteme.file.a");

    Ok(())
}

#[test]
fn episteme_evidence_selection_rejects_duplicate_file_ids() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let Err(error) = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_seed",
            vec!["episteme.file.a".to_string(), "episteme.file.a".to_string()],
        ),
        fixture.episteme_root.join("runs/evidence-selection"),
    ) else {
        return Err("duplicate selected file ids must fail".into());
    };
    assert!(error.to_string().contains("duplicate selected file_id"));

    Ok(())
}

#[test]
fn episteme_evidence_selection_rejects_unknown_file_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let Err(error) = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_seed",
            vec!["episteme.file.missing".to_string()],
        ),
        fixture.episteme_root.join("runs/evidence-selection"),
    ) else {
        return Err("unknown selected file id must fail".into());
    };
    assert!(error.to_string().contains("unknown selected file_id"));

    Ok(())
}

#[test]
fn episteme_evidence_selection_full_hash_rejects_hash_drift()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let source_path = fixture.corpus_root.join("docs/a.txt");
    let original = fs::read_to_string(&source_path)?;
    fs::write(&source_path, "x".repeat(original.len()))?;

    let Err(error) = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_full_hash_seed",
            vec!["episteme.file.a".to_string()],
        )
        .with_validation_mode(EpistemeEvidenceSelectionValidationMode::FullHash),
        fixture.episteme_root.join("runs/evidence-selection"),
    ) else {
        return Err("full-hash selection must reject same-size content drift".into());
    };
    assert!(error.to_string().contains("sha256 drift"));

    Ok(())
}

#[test]
fn episteme_selection_driven_extraction_plan_uses_selected_file_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let selection = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_seed",
            vec!["episteme.file.b".to_string(), "episteme.file.a".to_string()],
        ),
        fixture.episteme_root.join("runs/evidence-selection"),
    )?;
    let selected_file_ids =
        read_episteme_evidence_selection_file_ids(&selection.selection_tsv_path)?;

    let report = write_episteme_extraction_run_plan(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selected_extraction_seed",
        )
        .with_selected_file_ids(selected_file_ids),
        fixture.episteme_root.join("runs/extraction"),
    )?;

    assert_eq!(report.selected_count, 2);
    let tasks = fs::read_to_string(&report.tasks_path)?;
    assert!(tasks.contains("episteme.extract.b\tepisteme.file.b\timages/b.jpg"));
    assert!(tasks.contains("episteme.extract.a\tepisteme.file.a\tdocs/a.txt"));

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(
        receipt["selected_file_ids"],
        serde_json::json!(["episteme.file.b", "episteme.file.a"])
    );
    assert_eq!(receipt["tasks"][0]["file_id"], "episteme.file.b");
    assert_eq!(receipt["tasks"][1]["file_id"], "episteme.file.a");
    assert_eq!(receipt["raw_to_rdf_promotion_allowed"], false);
    assert_eq!(receipt["extraction_executed"], false);
    assert_eq!(receipt["validation_mode"], "contract_shape_only");

    Ok(())
}

#[test]
fn episteme_selection_driven_extraction_plan_rejects_unplannable_selected_id()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let Err(error) = write_episteme_extraction_run_plan(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selected_extraction_seed",
        )
        .with_route("document_text_evidence")
        .with_selected_file_ids(vec!["episteme.file.b".to_string()]),
        fixture.episteme_root.join("runs/extraction"),
    ) else {
        return Err("route-filtered selection must not silently drop selected ids".into());
    };
    assert!(
        error
            .to_string()
            .contains("selected file_id has no plannable pending queue row")
    );

    Ok(())
}

#[test]
fn episteme_selection_driven_extraction_plan_rejects_selection_over_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let Err(error) = write_episteme_extraction_run_plan(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selected_extraction_seed",
        )
        .with_limit(1)
        .with_selected_file_ids(vec![
            "episteme.file.a".to_string(),
            "episteme.file.b".to_string(),
        ]),
        fixture.episteme_root.join("runs/extraction"),
    ) else {
        return Err("selection larger than run-plan limit must fail".into());
    };
    assert!(error.to_string().contains("run-plan limit is 1"));

    Ok(())
}

#[test]
fn episteme_registry_loads_local_path_entry() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;

    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::local(
            "source_contract",
            fixture.episteme_root.clone(),
        )],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 1);
    assert_eq!(receipt.entries[0].id, "source_contract");
    assert_eq!(
        receipt.entries[0].source_kind,
        LoadedEpistemeSourceKind::Local
    );
    assert_eq!(receipt.entries[0].episteme_root, fixture.episteme_root);
    assert_eq!(receipt.entries[0].subdir, ".");
    assert!(receipt.entries[0].resolved_revision.is_none());
    Ok(())
}

#[test]
fn episteme_registry_filters_disabled_entries() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry {
            id: "disabled_entry".to_string(),
            path: None,
            url: None,
            enabled: false,
            subdir: PathBuf::from("."),
        }],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 0);
    assert!(receipt.entries.is_empty());
    Ok(())
}

#[test]
fn episteme_registry_rejects_mixed_path_and_url() {
    let result = load_episteme_registry_entries(
        &[EpistemeRegistryEntry {
            id: "mixed".to_string(),
            path: Some(PathBuf::from(".")),
            url: Some("https://github.com/example/example-episteme.git".to_string()),
            enabled: true,
            subdir: PathBuf::from("."),
        }],
        Path::new("."),
    );
    let Err(error) = result else {
        panic!("mixed path/url entry should fail");
    };

    assert!(error.to_string().contains("exactly one of `path` or `url`"));
}

#[test]
fn episteme_registry_rejects_unsafe_subdir() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;

    let result = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::local("unsafe", fixture.episteme_root).with_subdir("../escape")],
        Path::new("."),
    );
    let Err(error) = result else {
        panic!("unsafe subdir should fail");
    };

    assert!(error.to_string().contains("unsafe subdir"));
    Ok(())
}

#[test]
fn episteme_registry_materializes_git_url_entry() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;
    init_git_repository(fixture.episteme_root.as_path())?;
    let url = fixture.episteme_root.display().to_string();

    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::git("remote_source", url.clone())],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 1);
    assert_eq!(receipt.entries[0].id, "remote_source");
    assert_eq!(
        receipt.entries[0].source_kind,
        LoadedEpistemeSourceKind::Git
    );
    assert_eq!(receipt.entries[0].url.as_deref(), Some(url.as_str()));
    assert!(
        receipt.entries[0]
            .episteme_root
            .join("ontology/manifest.toml")
            .is_file()
    );
    assert!(receipt.entries[0].resolved_revision.is_some());

    cleanup_managed_git_entry("remote_source", url.as_str())?;
    Ok(())
}

#[test]
fn episteme_registry_reference_graph_accepts_satisfied_extension_target()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let common_root = temp.path().join("common-episteme");
    let extension_root = temp.path().join("extension-episteme");
    write_registry_manifest(
        common_root.as_path(),
        r#"schema_version = 1
name = "common-episteme"

[[domains]]
id = "episteme://common/domain"
"#,
    )?;
    write_registry_manifest(
        extension_root.as_path(),
        r#"schema_version = 1
name = "extension-episteme"

[extends]
manifest = "episteme://common/domain"

[[domains]]
id = "private://extension/domain"
"#,
    )?;

    let receipt = load_episteme_registry_entries(
        &[
            EpistemeRegistryEntry::local("common", common_root),
            EpistemeRegistryEntry::local("extension", extension_root),
        ],
        Path::new("."),
    )?;
    let graph = validate_episteme_registry_reference_graph(&receipt)?;

    assert_eq!(graph.entry_count, 2);
    assert_eq!(graph.domain_count, 2);
    assert_eq!(graph.reference_links.len(), 1);
    assert_eq!(graph.reference_links[0].source_registry, "extension");
    assert_eq!(
        graph.reference_links[0].target_domain,
        "episteme://common/domain"
    );
    assert_eq!(graph.reference_links[0].target_registry, "common");
    Ok(())
}

#[test]
fn episteme_registry_reference_graph_rejects_missing_extension_target()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let extension_root = temp.path().join("extension-episteme");
    write_registry_manifest(
        extension_root.as_path(),
        r#"schema_version = 1
name = "extension-episteme"

[extends]
manifest = "episteme://missing/domain"

[[domains]]
id = "private://extension/domain"
"#,
    )?;

    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::local("extension", extension_root)],
        Path::new("."),
    )?;
    let Err(error) = validate_episteme_registry_reference_graph(&receipt) else {
        panic!("missing extension target should fail");
    };

    assert!(error.to_string().contains("episteme://missing/domain"));
    assert!(error.to_string().contains("no loaded registry owns it"));
    Ok(())
}

#[test]
fn episteme_registry_reference_graph_rejects_duplicate_domain_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let first_root = temp.path().join("first-episteme");
    let second_root = temp.path().join("second-episteme");
    let manifest = r#"schema_version = 1
name = "duplicate-domain-episteme"

[[domains]]
id = "episteme://duplicate/domain"
"#;
    write_registry_manifest(first_root.as_path(), manifest)?;
    write_registry_manifest(second_root.as_path(), manifest)?;

    let receipt = load_episteme_registry_entries(
        &[
            EpistemeRegistryEntry::local("first", first_root),
            EpistemeRegistryEntry::local("second", second_root),
        ],
        Path::new("."),
    )?;
    let Err(error) = validate_episteme_registry_reference_graph(&receipt) else {
        panic!("duplicate domain ids should fail");
    };

    assert!(error.to_string().contains("episteme://duplicate/domain"));
    assert!(error.to_string().contains("first"));
    assert!(error.to_string().contains("second"));
    Ok(())
}

#[test]
fn episteme_registry_reference_graph_materializes_read_model_seed()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let common_root = temp.path().join("common-episteme");
    let extension_root = temp.path().join("extension-episteme");
    write_registry_manifest(
        common_root.as_path(),
        r#"schema_version = 1
name = "common-episteme"

[[domains]]
id = "episteme://common/domain"
"#,
    )?;
    write_registry_manifest(
        extension_root.as_path(),
        r#"schema_version = 1
name = "extension-episteme"

[extends]
manifest = "episteme://common/domain"

[[domains]]
id = "private://extension/domain"
"#,
    )?;

    let receipt = load_episteme_registry_entries(
        &[
            EpistemeRegistryEntry::local("common", common_root),
            EpistemeRegistryEntry::local("extension", extension_root),
        ],
        Path::new("."),
    )?;
    let graph = validate_episteme_registry_reference_graph(&receipt)?;
    let materialization = materialize_episteme_registry_reference_graph_read_model_seed(&graph)?;
    validate_episteme_read_model_relation_endpoints(&materialization)?;

    assert!(materialization.source_revision.starts_with("sha256:"));
    assert_eq!(materialization.row_counts(), [4, 3, 1]);

    let objects = table(&materialization, "semantic_objects");
    let object_ids = string_column(objects, "id");
    assert_eq!(object_ids.value(0), "episteme_registry.entry:common");
    assert_eq!(
        string_column(objects, "kind").value(1),
        "episteme_registry.domain"
    );

    let relations = table(&materialization, "semantic_relations");
    let relation_kinds = (0..relations.num_rows())
        .map(|index| string_column(relations, "kind").value(index).to_string())
        .collect::<BTreeSet<_>>();
    assert!(relation_kinds.contains("episteme_registry.loaded_entry.owns_domain"));
    assert!(relation_kinds.contains("episteme_registry.loaded_entry.extends_domain"));

    let projection = table(&materialization, "semantic_projection_state");
    assert_eq!(
        string_column(projection, "projection").value(0),
        "episteme_registry.reference_graph_read_model_seed.v1"
    );
    assert_eq!(i64_column(projection, "source_object_count").value(0), 4);

    Ok(())
}

#[test]
fn episteme_source_contract_materializes_read_model_seed() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let materialization = materialize_episteme_read_model_seed(&EpistemeReadModelRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
    ))?;
    validate_episteme_read_model_relation_endpoints(&materialization)?;

    assert!(materialization.source_revision.starts_with("sha256:"));
    assert_eq!(materialization.row_counts(), [4, 2, 1]);
    assert_eq!(materialization.tables[0].table_name(), "semantic_objects");
    assert_eq!(materialization.tables[1].table_name(), "semantic_relations");
    assert_eq!(
        materialization.tables[2].table_name(),
        "semantic_projection_state"
    );

    let objects = table(&materialization, "semantic_objects");
    assert_eq!(string_column(objects, "id").value(0), "episteme.file.a");
    assert_eq!(
        string_column(objects, "kind").value(0),
        "episteme_source_contract.source_file"
    );
    assert_eq!(
        string_column(objects, "source_path").value(0),
        "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT/docs/a.docx"
    );
    assert_eq!(i64_column(objects, "owner_count").value(0), 1);
    assert_eq!(i64_column(objects, "relation_count").value(0), 1);
    assert!(
        !string_column(objects, "verification_evidence_json")
            .value(0)
            .contains("fixture content"),
        "read-model seed must not embed raw source corpus text"
    );

    let relations = table(&materialization, "semantic_relations");
    assert_eq!(
        string_column(relations, "source").value(0),
        "episteme.extract.a"
    );
    assert_eq!(
        string_column(relations, "kind").value(0),
        "episteme_source_contract.extraction_task.has_source_file"
    );
    assert_eq!(
        string_column(relations, "target").value(0),
        "episteme.file.a"
    );

    let projection = table(&materialization, "semantic_projection_state");
    assert_eq!(
        string_column(projection, "projection").value(0),
        "episteme_source_contract.source_contract_read_model_seed.v1"
    );
    assert_eq!(i64_column(projection, "source_object_count").value(0), 4);

    Ok(())
}

#[test]
fn episteme_source_contract_validation_hash_cache_reuses_unchanged_files()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;
    let cache_path = fixture.episteme_root.join("cache/hash-cache.json");

    let (first_report, first_cache) = validate_episteme_source_contract_with_hash_cache(
        &fixture.episteme_root,
        &fixture.corpus_root,
        &cache_path,
    )?;
    assert!(first_report.passed, "{:?}", first_report.errors);
    assert_eq!(first_cache.hash_cache_hits, 0);
    assert_eq!(first_cache.hash_cache_misses, 2);
    assert_eq!(first_cache.entries_written, 2);

    let (second_report, second_cache) = validate_episteme_source_contract_with_hash_cache(
        &fixture.episteme_root,
        &fixture.corpus_root,
        &cache_path,
    )?;
    assert!(second_report.passed, "{:?}", second_report.errors);
    assert_eq!(second_cache.entries_loaded, 2);
    assert_eq!(second_cache.hash_cache_hits, 2);
    assert_eq!(second_cache.hash_cache_misses, 0);
    assert_eq!(second_cache.entries_written, 2);

    let (materialization, materialization_cache) =
        materialize_episteme_read_model_seed_with_validation_hash_cache(
            &EpistemeReadModelRequest::new(&fixture.episteme_root, &fixture.corpus_root),
            &cache_path,
        )?;
    assert_eq!(materialization.row_counts(), [4, 2, 1]);
    assert_eq!(materialization_cache.hash_cache_hits, 2);
    assert_eq!(materialization_cache.hash_cache_misses, 0);

    Ok(())
}

#[test]
fn episteme_source_contract_validation_rejects_invalid_mapping_ledger_properties()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fs::write(
        fixture.mapping_ledger_path(),
        SYNTHETIC_MAPPING_LEDGER.replace(
            "16b4038b-2c91-4f70-b38a-e0152629752d",
            "episteme.mapping.invalid",
        ),
    )?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;

    assert!(!report.passed);
    assert_eq!(report.mapping_ledger_sections, 0);
    assert_eq!(report.mapping_ledger_reasoning_property_records, 0);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("mapping ledger") && error.contains("UUID"))
    );

    let Err(error) = plan_episteme_extraction_run(&EpistemeRunPlanRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "source_contract_seed",
    )) else {
        return Err("invalid mapping ledger should prevent run planning".into());
    };
    assert!(error.to_string().contains("mapping ledger"));

    Ok(())
}

#[test]
fn episteme_source_contract_validation_hash_cache_rejects_drift()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    let cache_path = fixture.episteme_root.join("cache/hash-cache.json");

    let (first_report, first_cache) = validate_episteme_source_contract_with_hash_cache(
        &fixture.episteme_root,
        &fixture.corpus_root,
        &cache_path,
    )?;
    assert!(first_report.passed, "{:?}", first_report.errors);
    assert_eq!(first_cache.hash_cache_misses, 1);

    fs::write(fixture.corpus_root.join("docs/a.docx"), "changed")?;
    let (second_report, second_cache) = validate_episteme_source_contract_with_hash_cache(
        &fixture.episteme_root,
        &fixture.corpus_root,
        &cache_path,
    )?;
    assert!(!second_report.passed);
    assert!(
        second_report
            .errors
            .iter()
            .any(|error| error.contains("sha256 drift"))
    );
    assert_eq!(second_cache.hash_cache_hits, 0);
    assert_eq!(second_cache.hash_cache_misses, 1);

    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_read_model_seed_builds_wendaograph_quality_request()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let materialization = materialize_episteme_read_model_seed(&EpistemeReadModelRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
    ))?;
    let quality_batches = build_episteme_wendaograph_quality_request_batches(&materialization)?;

    assert_eq!(quality_batches.row_counts(), [4, 2, 1]);

    let request = build_wendaograph_ontology_read_model_quality_arrow_request(&quality_batches)?;
    let bundle = build_wendaograph_ontology_read_model_quality_flight_request_batch(&request)?;
    assert_eq!(bundle.num_rows(), 1);
    assert!(
        request
            .payload_byte_sizes()
            .into_iter()
            .all(|size| size > 0)
    );

    let objects = decode_single_arrow_batch(request.semantic_objects_payload.as_slice())?;
    assert_eq!(string_column(&objects, "id").value(0), "episteme.file.a");
    assert_eq!(
        string_column(&objects, "read_model_projection_staleness").value(0),
        "fresh"
    );

    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_base_url_normalization_trims_trailing_slash() {
    let Ok(base_url) = normalize_live_quality_base_url("  http://127.0.0.1:41082/  ") else {
        panic!("valid base URL should normalize");
    };

    assert_eq!(base_url, "http://127.0.0.1:41082");
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_base_url_normalization_rejects_blank() {
    let Err(error) = normalize_live_quality_base_url("   ") else {
        panic!("blank URL should fail");
    };

    assert!(error.contains(EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV));
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_base_url_normalization_rejects_unsupported_scheme() {
    let Err(error) = normalize_live_quality_base_url("grpc://127.0.0.1:41082") else {
        panic!("unsupported scheme should fail");
    };

    assert!(error.contains("http:// or https://"));
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_round_count_parser_uses_default() {
    let Ok(count) = parse_live_quality_round_count(None, "TEST_ROUNDS", 0, 0, 3) else {
        panic!("missing env should use default");
    };

    assert_eq!(count, 0);
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_round_count_parser_accepts_valid_count() {
    let Ok(count) = parse_live_quality_round_count(Some("2"), "TEST_ROUNDS", 0, 0, 3) else {
        panic!("valid round count should parse");
    };

    assert_eq!(count, 2);
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_round_count_parser_rejects_invalid_count() {
    let Err(error) = parse_live_quality_round_count(Some("abc"), "TEST_ROUNDS", 0, 0, 3) else {
        panic!("invalid round count should fail");
    };

    assert!(error.contains("TEST_ROUNDS"));
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_round_count_parser_rejects_out_of_range_count() {
    let Err(error) = parse_live_quality_round_count(Some("4"), "TEST_ROUNDS", 0, 0, 3) else {
        panic!("out-of-range round count should fail");
    };

    assert!(error.contains("between 0 and 3"));
}

#[cfg(feature = "julia")]
#[tokio::test]
async fn episteme_source_contract_live_wendaograph_quality_diagnostic_uses_compiled_seed()
-> Result<(), Box<dyn std::error::Error>> {
    if env::var_os(RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_ENV).is_none() {
        eprintln!(
            "skipping episteme source-contract WendaoGraph quality live diagnostic; set {RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_ENV}=1"
        );
        return Ok(());
    }

    let context = live_quality_diagnostic_context()?;
    let materialized = materialize_live_quality_read_model(&context.repo_root)?;
    let (quality_batches, request_packaging_ms) =
        package_live_quality_batches(&materialized.materialization)?;
    let service = start_live_quality_service(&context).await?;
    let prewarm_summaries =
        run_live_quality_prewarm_roundtrips(&service.binding, &quality_batches).await?;
    let roundtrip_summaries =
        run_live_quality_roundtrips(&service.binding, &quality_batches).await?;

    write_live_quality_evidence(&LiveQualityEvidenceInput {
        repo_root: &context.repo_root,
        source_revision: &materialized.materialization.source_revision,
        request_row_counts: quality_batches.row_counts(),
        phase_timings: LiveQualityPhaseTimings {
            materialization: materialized.elapsed_ms,
            request_packaging: request_packaging_ms,
            service_ready: service.ready_ms,
        },
        service: &service,
        prewarm_summaries: &prewarm_summaries,
        roundtrip_summaries: &roundtrip_summaries,
        validation_hash_cache_report: materialized.validation_hash_cache_report.as_ref(),
    })?;

    Ok(())
}

#[test]
fn episteme_source_contract_validation_reports_hash_drift() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fs::write(fixture.corpus_root.join("docs/a.docx"), "changed")?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;
    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("sha256 drift"))
    );

    Ok(())
}

#[test]
fn episteme_extraction_plan_uses_shape_validation_without_hashing()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let source_path = fixture.corpus_root.join("docs/a.txt");
    let original = fs::read_to_string(&source_path)?;
    fs::write(&source_path, "x".repeat(original.len()))?;

    let receipt = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "shape_only_plan_seed",
        )
        .with_limit(1),
    )?;

    assert_eq!(receipt.selected_count, 1);
    assert_eq!(receipt.validation_mode, "contract_shape_only");

    let full_validation =
        validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;
    assert!(!full_validation.passed);
    assert!(
        full_validation
            .errors
            .iter()
            .any(|error| error.contains("sha256 drift"))
    );

    Ok(())
}

#[test]
fn episteme_extraction_plan_shape_validation_rejects_queue_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fs::write(
        fixture
            .episteme_root
            .join("ontology/SourceContract/corpus/extraction_queue.tsv"),
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus\n",
    )?;

    let Err(error) = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "queue_shape_plan_seed",
        )
        .with_limit(1),
    ) else {
        return Err("queue mismatch should fail shape-only planning".into());
    };
    assert!(
        error
            .to_string()
            .contains("extraction_queue.tsv missing file_id")
    );

    Ok(())
}

#[test]
fn episteme_source_contract_read_model_rejects_hash_drift() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fs::write(fixture.corpus_root.join("docs/a.docx"), "changed")?;

    let Err(error) = materialize_episteme_read_model_seed(&EpistemeReadModelRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
    )) else {
        return Err("hash drift should prevent read-model materialization".into());
    };
    assert!(error.to_string().contains("invalid"));

    Ok(())
}

#[test]
fn episteme_source_contract_planner_rejects_unsafe_run_id() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let Err(error) = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(&fixture.episteme_root, &fixture.corpus_root, "../bad")
            .with_route("document_text_evidence")
            .with_limit(1),
    ) else {
        return Err("unsafe run id should fail".into());
    };
    assert!(error.to_string().contains("invalid run id"));

    Ok(())
}

#[test]
fn episteme_source_contract_manifest_active_selector_resolves_multi_domain_repo()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fixture.write_multi_domain_manifest(true)?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;

    assert!(report.passed, "{:?}", report.errors);
    assert_eq!(
        report.corpus_root_env,
        "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT"
    );

    Ok(())
}

#[test]
fn episteme_source_contract_manifest_requires_active_selector_for_multi_domain_repo()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fixture.write_multi_domain_manifest(false)?;

    let Err(error) =
        validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)
    else {
        return Err("multi-domain episteme without active selector should fail".into());
    };

    assert!(error.to_string().contains("active_source_contract"));

    Ok(())
}

struct EpistemeFixture {
    _temp: tempfile::TempDir,
    episteme_root: PathBuf,
    corpus_root: PathBuf,
    files: Vec<FileFixture>,
}

impl EpistemeFixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let episteme_root = temp.path().join("source-contract");
        let corpus_root = temp.path().join("corpus-root");
        fs::create_dir_all(episteme_root.join("ontology/SourceContract/corpus"))?;
        fs::create_dir_all(episteme_root.join("ontology/SourceContract/mappings"))?;
        fs::create_dir_all(&corpus_root)?;
        Ok(Self {
            _temp: temp,
            episteme_root,
            corpus_root,
            files: Vec::new(),
        })
    }

    fn add_source(
        &mut self,
        relative_path: &str,
        file_id: &str,
        queue_id: &str,
        category: &str,
        route: &str,
        priority: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_path = self.corpus_root.join(relative_path);
        if let Some(parent) = source_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&source_path, format!("fixture content for {relative_path}"))?;
        let metadata = fs::metadata(&source_path)?;
        self.files.push(FileFixture {
            file_id: file_id.to_string(),
            queue_id: queue_id.to_string(),
            relative_path: relative_path.to_string(),
            extension: Path::new(relative_path)
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            byte_size: metadata.len(),
            sha256: sha256_file(&source_path)?,
            category: category.to_string(),
            route: route.to_string(),
            priority,
        });
        Ok(())
    }

    fn write_contract(&self) -> Result<(), Box<dyn std::error::Error>> {
        let corpus_dir = self.episteme_root.join("ontology/SourceContract/corpus");
        fs::write(
            self.episteme_root.join("ontology/manifest.toml"),
            r#"schema_version = 1
name = "synthetic-source-contract"
primary_language = "zh-CN"
artifact_mode = "source_contract"
mutation_allowed = false

[[domains]]
id = "episteme://synthetic/source-contract"
source_manifests = ["SourceContract/corpus/source_manifest.toml"]
mapping_ledgers = ["SourceContract/mappings/corpus_mapping.org"]
"#,
        )?;
        fs::write(self.mapping_ledger_path(), SYNTHETIC_MAPPING_LEDGER)?;
        fs::write(
            corpus_dir.join("source_manifest.toml"),
            r#"schema_version = 1
source_contract_id = "episteme_source_contract.corpus.v1"
domain = "episteme://synthetic/source-contract"
primary_language = "zh-CN"
corpus_root_env = "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT"
files = "files.tsv"
extraction_queue = "extraction_queue.tsv"
copy_raw_files = false
raw_to_rdf_promotion_allowed = false

ignored_names = [".DS_Store"]

[routes]
document_text_evidence = ["docx", "txt"]
image_ocr_evidence = ["jpg"]
"#,
        )?;

        let mut files_tsv = fs::File::create(corpus_dir.join("files.tsv"))?;
        writeln!(
            files_tsv,
            "file_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route"
        )?;
        for file in &self.files {
            writeln!(
                files_tsv,
                "{}\t{}\t{}\t{}\t{}\t{}\tzh-CN\t{}",
                file.file_id,
                file.relative_path,
                file.extension,
                file.byte_size,
                file.sha256,
                file.category,
                file.route
            )?;
        }

        let mut queue_tsv = fs::File::create(corpus_dir.join("extraction_queue.tsv"))?;
        writeln!(
            queue_tsv,
            "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus"
        )?;
        for file in &self.files {
            writeln!(
                queue_tsv,
                "{}\t{}\t{}\t{}\tzh-CN\t{}\t{}\tcache_only_no_rdf_promotion\tpending",
                file.queue_id,
                file.file_id,
                file.relative_path,
                file.category,
                file.route,
                file.priority
            )?;
        }
        Ok(())
    }

    fn write_multi_domain_manifest(&self, active: bool) -> Result<(), Box<dyn std::error::Error>> {
        let active_block = if active {
            r#"
[active_source_contract]
domain_id = "episteme://synthetic/source-contract"
source_manifest = "SourceContract/corpus/source_manifest.toml"
mapping_ledger = "SourceContract/mappings/corpus_mapping.org"
"#
        } else {
            ""
        };
        fs::write(
            self.episteme_root.join("ontology/manifest.toml"),
            format!(
                r#"schema_version = 1
name = "synthetic-source-contract"
primary_language = "zh-CN"
artifact_mode = "source_contract"
mutation_allowed = false
{active_block}
[[domains]]
id = "episteme://synthetic/source-contract"
source_manifests = ["SourceContract/corpus/source_manifest.toml"]
mapping_ledgers = ["SourceContract/mappings/corpus_mapping.org"]

[[domains]]
id = "episteme://synthetic/secondary"
source_manifests = ["Secondary/corpus/source_manifest.toml"]
mapping_ledgers = ["Secondary/mappings/corpus_mapping.org"]
"#
            ),
        )?;
        Ok(())
    }

    fn mapping_ledger_path(&self) -> PathBuf {
        self.episteme_root
            .join("ontology/SourceContract/mappings/corpus_mapping.org")
    }
}

struct FileFixture {
    file_id: String,
    queue_id: String,
    relative_path: String,
    extension: String,
    byte_size: u64,
    sha256: String,
    category: String,
    route: String,
    priority: u32,
}

const SYNTHETIC_MAPPING_LEDGER: &str = r"#+TITLE: Synthetic Source Corpus Mapping Ledger

* Synthetic source corpus mapping
:PROPERTIES:
:ID: 16b4038b-2c91-4f70-b38a-e0152629752d
:WENDAO_KIND: ontology_mapping
:ONTOLOGY_KIND: corpus_mapping
:DOMAIN: episteme://synthetic/source-contract
:MAPPING_ID: episteme_source_contract.corpus.v1
:PROMOTION_STATE: candidate
:LIFECYCLE_STATE: candidate
:PRIMARY_LANGUAGE: zh-CN
:END:

This synthetic fixture verifies the source corpus mapping contract shape
without embedding customer source content in Rust tests.

** Corpus coverage

| source_group | evidence_role | extraction_route |
| synthetic_policy_group | synthetic policy evidence | document_text_evidence |

** Evidence policy

| decision | state | reason |
| raw files are evidence only | accepted | synthetic raw rows are not ontology truth |
";

fn table<'a>(
    materialization: &'a xiuxian_wendao::episteme::EpistemeReadModelMaterialization,
    table_name: &str,
) -> &'a RecordBatch {
    materialization
        .tables
        .iter()
        .find(|table| table.table_name() == table_name)
        .unwrap_or_else(|| panic!("missing table {table_name}"))
        .batch()
}

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a StringArray {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("missing string column {name}"))
}

fn i64_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a Int64Array {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<Int64Array>())
        .unwrap_or_else(|| panic!("missing i64 column {name}"))
}

#[cfg(feature = "julia")]
fn decode_single_arrow_batch(payload: &[u8]) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)?;
    let batches = reader.collect::<Result<Vec<_>, _>>()?;
    let [batch] = batches.as_slice() else {
        return Err(format!("expected one Arrow batch, got {}", batches.len()).into());
    };
    Ok(batch.clone())
}

#[cfg(feature = "julia")]
struct LiveQualityDiagnosticContext {
    repo_root: PathBuf,
    wendaograph_project: PathBuf,
    runner: PathBuf,
}

#[cfg(feature = "julia")]
fn live_quality_diagnostic_context()
-> Result<LiveQualityDiagnosticContext, Box<dyn std::error::Error>> {
    let repo_root = repo_root()?;
    let wendaograph_project = wendaograph_project_root(&repo_root)?;
    let runner = wendaograph_project
        .join("scripts")
        .join("run_ontology_read_model_quality_service.jl");
    if !runner.is_file() {
        return Err(format!(
            "missing WendaoGraph ontology quality runner `{}`",
            runner.display()
        )
        .into());
    }
    Ok(LiveQualityDiagnosticContext {
        repo_root,
        wendaograph_project,
        runner,
    })
}

#[cfg(feature = "julia")]
struct LiveQualityMaterialization {
    materialization: EpistemeReadModelMaterialization,
    validation_hash_cache_report: Option<EpistemeValidationHashCacheReport>,
    elapsed_ms: f64,
}

#[cfg(feature = "julia")]
fn materialize_live_quality_read_model(
    repo_root: &Path,
) -> Result<LiveQualityMaterialization, Box<dyn std::error::Error>> {
    let Some(episteme_root) = env::var_os(EPISTEME_SOURCE_CONTRACT_ROOT_ENV) else {
        return Err(format!(
            "set {EPISTEME_SOURCE_CONTRACT_ROOT_ENV} for live episteme source-contract quality"
        )
        .into());
    };
    let episteme_root = resolve_repo_relative_path(repo_root, &PathBuf::from(episteme_root));
    let corpus_root_env = configured_episteme_corpus_root_env(&episteme_root)?;
    let Some(corpus_root) = env::var_os(corpus_root_env.as_str()) else {
        return Err(
            format!("set {corpus_root_env} for live episteme source-contract quality").into(),
        );
    };
    let corpus_root = PathBuf::from(corpus_root);
    let read_model_request = EpistemeReadModelRequest::new(episteme_root, corpus_root);
    let validation_hash_cache_path = episteme_source_contract_validation_hash_cache_path(repo_root);
    let started_at = Instant::now();
    let (materialization, validation_hash_cache_report) =
        if let Some(cache_path) = validation_hash_cache_path.as_ref() {
            let (materialization, cache_report) =
                materialize_episteme_read_model_seed_with_validation_hash_cache(
                    &read_model_request,
                    cache_path,
                )?;
            (materialization, Some(cache_report))
        } else {
            (
                materialize_episteme_read_model_seed(&read_model_request)?,
                None,
            )
        };

    Ok(LiveQualityMaterialization {
        materialization,
        validation_hash_cache_report,
        elapsed_ms: elapsed_millis(started_at),
    })
}

#[cfg(feature = "julia")]
fn package_live_quality_batches(
    materialization: &EpistemeReadModelMaterialization,
) -> Result<(WendaoGraphOntologyReadModelQualityRequestBatches, f64), Box<dyn std::error::Error>> {
    let started_at = Instant::now();
    let quality_batches = build_episteme_wendaograph_quality_request_batches(materialization)?;
    let elapsed_ms = elapsed_millis(started_at);
    assert_eq!(quality_batches.row_counts(), [380, 190, 1]);
    Ok((quality_batches, elapsed_ms))
}

#[cfg(feature = "julia")]
struct LiveQualityService {
    _process_guard: Option<ChildGuard>,
    binding: PluginCapabilityBinding,
    mode: &'static str,
    base_url: String,
    ready_ms: f64,
}

#[cfg(feature = "julia")]
async fn start_live_quality_service(
    context: &LiveQualityDiagnosticContext,
) -> Result<LiveQualityService, Box<dyn std::error::Error>> {
    if let Some(base_url) = live_quality_external_base_url()? {
        let started_at = Instant::now();
        let binding = live_quality_binding(base_url.clone())?;
        return Ok(LiveQualityService {
            _process_guard: None,
            binding,
            mode: "external",
            base_url,
            ready_ms: elapsed_millis(started_at),
        });
    }

    let port = reserve_loopback_port()?;
    let base_url = format!("http://127.0.0.1:{port}");
    let started_at = Instant::now();
    let guard = ChildGuard::spawn(
        Command::new("julia")
            .arg(format!(
                "--project={}",
                context.wendaograph_project.display()
            ))
            .arg(&context.runner)
            .arg("--host=127.0.0.1")
            .arg(format!("--port={port}"))
            .arg("--max-active-requests=4")
            .arg("--request-capacity=4")
            .arg("--response-capacity=4")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit()),
    )?;

    wait_for_tcp_ready(port).await?;
    let binding = live_quality_binding(base_url.clone())?;
    Ok(LiveQualityService {
        _process_guard: Some(guard),
        binding,
        mode: "spawned",
        base_url,
        ready_ms: elapsed_millis(started_at),
    })
}

#[cfg(feature = "julia")]
fn live_quality_external_base_url() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(raw) = env::var_os(EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV) else {
        return Ok(None);
    };
    normalize_live_quality_base_url(raw.to_string_lossy().as_ref())
        .map(Some)
        .map_err(Into::into)
}

#[cfg(feature = "julia")]
fn normalize_live_quality_base_url(raw: &str) -> Result<String, String> {
    let base_url = raw.trim().trim_end_matches('/');
    if base_url.is_empty() {
        return Err(format!(
            "{EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV} must not be blank"
        ));
    }
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err(format!(
            "{EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV} must start with http:// or https://"
        ));
    }
    Ok(base_url.to_owned())
}

#[cfg(feature = "julia")]
fn live_quality_binding(base_url: String) -> Result<PluginCapabilityBinding, String> {
    build_wendaograph_ontology_read_model_quality_flight_binding(
        WendaoGraphOntologyReadModelQualityFlightBindingOptions {
            base_url,
            health_route: None,
            timeout_secs: Some(30),
            max_in_flight_requests: Some(1),
        },
    )
}

#[cfg(feature = "julia")]
async fn run_live_quality_roundtrips(
    binding: &PluginCapabilityBinding,
    quality_batches: &WendaoGraphOntologyReadModelQualityRequestBatches,
) -> Result<Vec<LiveQualityRoundtripSummary>, Box<dyn std::error::Error>> {
    let repeat_count = episteme_source_contract_quality_repeat_count()?;
    run_live_quality_roundtrip_count(binding, quality_batches, repeat_count).await
}

#[cfg(feature = "julia")]
async fn run_live_quality_prewarm_roundtrips(
    binding: &PluginCapabilityBinding,
    quality_batches: &WendaoGraphOntologyReadModelQualityRequestBatches,
) -> Result<Vec<LiveQualityRoundtripSummary>, Box<dyn std::error::Error>> {
    let prewarm_count = episteme_source_contract_quality_prewarm_count()?;
    run_live_quality_roundtrip_count(binding, quality_batches, prewarm_count).await
}

#[cfg(feature = "julia")]
async fn run_live_quality_roundtrip_count(
    binding: &PluginCapabilityBinding,
    quality_batches: &WendaoGraphOntologyReadModelQualityRequestBatches,
    repeat_count: usize,
) -> Result<Vec<LiveQualityRoundtripSummary>, Box<dyn std::error::Error>> {
    let mut roundtrip_summaries = Vec::with_capacity(repeat_count);
    for run_index in 1..=repeat_count {
        let started_at = Instant::now();
        let Some(roundtrip) = roundtrip_wendaograph_ontology_read_model_quality_with_binding(
            binding,
            quality_batches,
        )
        .await
        .map_err(|error| format!("{error:?}"))?
        else {
            return Err(
                "live episteme source-contract ontology quality Flight binding did not negotiate"
                    .into(),
            );
        };
        assert_eq!(
            roundtrip.selection.selected_transport,
            PluginTransportKind::ArrowFlight
        );
        assert_response_batches_pass(&roundtrip.response_batches);
        roundtrip_summaries.push(LiveQualityRoundtripSummary::from_batches(
            run_index,
            elapsed_millis(started_at),
            &roundtrip.response_batches,
        ));
    }
    Ok(roundtrip_summaries)
}

#[cfg(feature = "julia")]
struct ChildGuard {
    child: Child,
}

#[cfg(feature = "julia")]
impl ChildGuard {
    fn spawn(command: &mut Command) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            child: command.spawn()?,
        })
    }
}

#[cfg(feature = "julia")]
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(feature = "julia")]
fn repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_dir.ancestors() {
        if ancestor.join(".data/WendaoGraph.jl").is_dir() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Err(format!("could not find repo root from `{}`", manifest_dir.display()).into())
}

#[cfg(feature = "julia")]
fn wendaograph_project_root(repo_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let raw = env::var_os("WENDAOGRAPH_PACKAGE_DIR")
        .map_or_else(|| repo_root.join(".data/WendaoGraph.jl"), PathBuf::from);
    let project = if raw.is_absolute() {
        raw
    } else {
        repo_root.join(raw)
    };
    if project.join("Project.toml").is_file() {
        Ok(project)
    } else {
        Err(format!(
            "missing WendaoGraph Project.toml under `{}`",
            project.display()
        )
        .into())
    }
}

#[cfg(feature = "julia")]
fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

#[cfg(feature = "julia")]
fn reserve_loopback_port() -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

#[cfg(feature = "julia")]
async fn wait_for_tcp_ready(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let address = format!("127.0.0.1:{port}");
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut last_error = String::new();
    while Instant::now() < deadline {
        match tokio::net::TcpStream::connect(address.as_str()).await {
            Ok(_) => return Ok(()),
            Err(error) => {
                last_error = error.to_string();
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(format!("WendaoGraph service did not become ready: {last_error}").into())
}

#[cfg(feature = "julia")]
fn episteme_source_contract_quality_repeat_count() -> Result<usize, Box<dyn std::error::Error>> {
    let raw = env::var(EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_REPEAT_ENV).ok();
    parse_live_quality_round_count(
        raw.as_deref(),
        EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_REPEAT_ENV,
        1,
        1,
        10,
    )
    .map_err(Into::into)
}

#[cfg(feature = "julia")]
fn episteme_source_contract_quality_prewarm_count() -> Result<usize, Box<dyn std::error::Error>> {
    let raw = env::var(EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_PREWARM_ENV).ok();
    parse_live_quality_round_count(
        raw.as_deref(),
        EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_PREWARM_ENV,
        0,
        0,
        10,
    )
    .map_err(Into::into)
}

#[cfg(feature = "julia")]
fn parse_live_quality_round_count(
    raw: Option<&str>,
    env_name: &str,
    default_count: usize,
    min_count: usize,
    max_count: usize,
) -> Result<usize, String> {
    let Some(raw) = raw else {
        return Ok(default_count);
    };
    let value = raw.trim();
    let count: usize = value
        .parse()
        .map_err(|error| format!("{env_name} must be an integer: {error}"))?;
    if !(min_count..=max_count).contains(&count) {
        return Err(format!(
            "{env_name} must be between {min_count} and {max_count}"
        ));
    }
    Ok(count)
}

#[cfg(feature = "julia")]
fn episteme_source_contract_validation_hash_cache_path(repo_root: &Path) -> Option<PathBuf> {
    env::var_os(EPISTEME_SOURCE_CONTRACT_VALIDATION_HASH_CACHE_PATH_ENV).map(|raw| {
        let path = PathBuf::from(raw);
        if path.is_absolute() {
            path
        } else {
            repo_root.join(path)
        }
    })
}

#[cfg(feature = "julia")]
fn elapsed_millis(started_at: Instant) -> f64 {
    started_at.elapsed().as_secs_f64() * 1000.0
}

#[cfg(feature = "julia")]
fn assert_response_batches_pass(batches: &[RecordBatch]) {
    assert!(!batches.is_empty());
    let mut saw_component_count = false;
    let mut saw_pass = false;
    for batch in batches {
        assert!(batch.num_rows() > 0);
        let check_ids = string_column(batch, "check_id");
        let statuses = string_column(batch, "status");
        for index in 0..batch.num_rows() {
            saw_component_count |= check_ids.value(index) == "object_graph_component_count";
            saw_pass |= statuses.value(index) == "pass";
            assert_ne!(statuses.value(index), "fail");
            assert_ne!(statuses.value(index), "error");
        }
    }
    assert!(
        saw_component_count,
        "response batches must include object graph component quality check"
    );
    assert!(
        saw_pass,
        "response batches must include at least one pass row"
    );
}

#[cfg(feature = "julia")]
struct LiveQualityRoundtripSummary {
    run_index: usize,
    elapsed_ms: f64,
    response_batch_count: usize,
    response_rows: usize,
    status_counts: BTreeMap<String, usize>,
    pass_rows: usize,
    failed_rows: usize,
    check_ids: Vec<String>,
}

#[cfg(feature = "julia")]
impl LiveQualityRoundtripSummary {
    fn from_batches(run_index: usize, elapsed_ms: f64, batches: &[RecordBatch]) -> Self {
        let status_counts = status_counts(batches);
        let pass_rows = status_counts.get("pass").copied().unwrap_or_default();
        let failed_rows = status_counts.get("fail").copied().unwrap_or_default()
            + status_counts.get("error").copied().unwrap_or_default();
        Self {
            run_index,
            elapsed_ms,
            response_batch_count: batches.len(),
            response_rows: response_row_count(batches),
            status_counts,
            pass_rows,
            failed_rows,
            check_ids: unique_string_values(batches, "check_id"),
        }
    }

    fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "runIndex": self.run_index,
            "elapsedMs": self.elapsed_ms,
            "responseBatchCount": self.response_batch_count,
            "responseRows": self.response_rows,
            "statusCounts": self.status_counts,
            "passRows": self.pass_rows,
            "failedRows": self.failed_rows,
            "checkIds": self.check_ids
        })
    }
}

#[cfg(feature = "julia")]
struct LiveQualityPhaseTimings {
    materialization: f64,
    request_packaging: f64,
    service_ready: f64,
}

#[cfg(feature = "julia")]
struct LiveQualityEvidenceInput<'a> {
    repo_root: &'a Path,
    source_revision: &'a str,
    request_row_counts: [usize; 3],
    phase_timings: LiveQualityPhaseTimings,
    service: &'a LiveQualityService,
    prewarm_summaries: &'a [LiveQualityRoundtripSummary],
    roundtrip_summaries: &'a [LiveQualityRoundtripSummary],
    validation_hash_cache_report: Option<&'a EpistemeValidationHashCacheReport>,
}

#[cfg(feature = "julia")]
fn write_live_quality_evidence(
    input: &LiveQualityEvidenceInput<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(last_summary) = input.roundtrip_summaries.last() else {
        return Err(
            "episteme source-contract quality evidence requires at least one roundtrip summary"
                .into(),
        );
    };
    let evidence_dir = cache_home(input.repo_root)
        .join("agent/evidence/episteme_source_contract_wendaograph_quality");
    fs::create_dir_all(&evidence_dir)?;
    let roundtrip_elapsed_ms = input
        .roundtrip_summaries
        .iter()
        .map(|summary| summary.elapsed_ms)
        .collect::<Vec<_>>();
    let prewarm_elapsed_ms = input
        .prewarm_summaries
        .iter()
        .map(|summary| summary.elapsed_ms)
        .collect::<Vec<_>>();
    let report = serde_json::json!({
        "schemaVersion": "xiuxian_wendao.episteme_source_contract_wendaograph_quality_live_report.v1",
        "sourceRevision": input.source_revision,
        "requestRowCounts": {
            "semanticObjects": input.request_row_counts[0],
            "semanticRelations": input.request_row_counts[1],
            "semanticProjectionState": input.request_row_counts[2]
        },
        "phaseTimingsMs": {
            "rustMaterialization": input.phase_timings.materialization,
            "requestPackaging": input.phase_timings.request_packaging,
            "serviceReady": input.phase_timings.service_ready,
            "roundtripLast": last_summary.elapsed_ms,
            "roundtripMin": min_f64(&roundtrip_elapsed_ms),
            "roundtripAvg": avg_f64(&roundtrip_elapsed_ms),
            "warmRoundtripAvg": warm_avg_f64(&roundtrip_elapsed_ms)
        },
        "repeatCount": input.roundtrip_summaries.len(),
        "prewarmCount": input.prewarm_summaries.len(),
        "prewarmTimingMs": {
            "roundtripMin": min_f64(&prewarm_elapsed_ms),
            "roundtripAvg": avg_f64(&prewarm_elapsed_ms),
            "roundtripLast": input.prewarm_summaries.last().map(|summary| summary.elapsed_ms)
        },
        "serviceMode": input.service.mode,
        "serviceBaseUrl": input.service.base_url,
        "validationHashCache": input.validation_hash_cache_report,
        "prewarmRuns": input.prewarm_summaries
            .iter()
            .map(LiveQualityRoundtripSummary::as_json)
            .collect::<Vec<_>>(),
        "roundtripRuns": input.roundtrip_summaries
            .iter()
            .map(LiveQualityRoundtripSummary::as_json)
            .collect::<Vec<_>>(),
        "responseBatchCount": last_summary.response_batch_count,
        "responseRows": last_summary.response_rows,
        "statusCounts": last_summary.status_counts,
        "passRows": last_summary.pass_rows,
        "failedRows": last_summary.failed_rows,
        "checkIds": last_summary.check_ids,
        "elapsedMs": last_summary.elapsed_ms,
        "rawCorpusReadByJulia": false,
        "rdfPromotion": false
    });
    let body = format!("{}\n", serde_json::to_string_pretty(&report)?);
    fs::write(evidence_dir.join("latest.json"), &body)?;
    fs::write(
        evidence_dir.join(format!("report-{}.json", unix_timestamp_secs()?)),
        body,
    )?;
    Ok(())
}

#[cfg(feature = "julia")]
fn min_f64(values: &[f64]) -> Option<f64> {
    let (first, rest) = values.split_first()?;
    let mut min = *first;
    for value in rest {
        if *value < min {
            min = *value;
        }
    }
    Some(min)
}

#[cfg(feature = "julia")]
fn avg_f64(values: &[f64]) -> Option<f64> {
    let (first, rest) = values.split_first()?;
    let total = rest.iter().fold(*first, |sum, value| sum + value);
    let len = u32::try_from(values.len()).ok()?;
    Some(total / f64::from(len))
}

#[cfg(feature = "julia")]
fn warm_avg_f64(values: &[f64]) -> Option<f64> {
    if values.len() <= 1 {
        return None;
    }
    avg_f64(&values[1..])
}

#[cfg(feature = "julia")]
fn cache_home(repo_root: &Path) -> PathBuf {
    let raw = env::var_os("PRJ_CACHE_HOME").map_or_else(|| repo_root.join(".cache"), PathBuf::from);
    if raw.is_absolute() {
        raw
    } else {
        repo_root.join(raw)
    }
}

#[cfg(feature = "julia")]
fn unix_timestamp_secs() -> Result<u64, Box<dyn std::error::Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

#[cfg(feature = "julia")]
fn response_row_count(batches: &[RecordBatch]) -> usize {
    batches.iter().map(RecordBatch::num_rows).sum()
}

#[cfg(feature = "julia")]
fn status_counts(batches: &[RecordBatch]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for batch in batches {
        let statuses = string_column(batch, "status");
        for index in 0..statuses.len() {
            *counts.entry(statuses.value(index).to_owned()).or_default() += 1;
        }
    }
    counts
}

#[cfg(feature = "julia")]
fn unique_string_values(batches: &[RecordBatch], column_name: &str) -> Vec<String> {
    let mut values = BTreeSet::new();
    for batch in batches {
        let column = string_column(batch, column_name);
        for index in 0..column.len() {
            values.insert(column.value(index).to_owned());
        }
    }
    values.into_iter().collect()
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_registry_manifest(
    episteme_root: &Path,
    manifest: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(episteme_root.join("ontology"))?;
    fs::write(episteme_root.join("ontology/manifest.toml"), manifest)?;
    Ok(())
}

fn init_git_repository(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_git(root, &["init"])?;
    run_git(root, &["config", "user.name", "episteme-registry-test"])?;
    run_git(
        root,
        &["config", "user.email", "episteme-registry-test@example.com"],
    )?;
    run_git(root, &["add", "."])?;
    run_git(root, &["commit", "-m", "seed episteme fixture"])?;
    Ok(())
}

fn run_git(root: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {args:?} failed with status {status}").into())
    }
}

fn cleanup_managed_git_entry(id: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let spec = xiuxian_git_repo::RepoSpec {
        id: format!("episteme-{id}"),
        local_path: None,
        remote_url: Some(url.to_string()),
        revision: None,
        refresh: xiuxian_git_repo::RepoRefreshPolicy::Fetch,
    };
    for path in [
        xiuxian_git_repo::managed_checkout_root_for(&spec),
        xiuxian_git_repo::managed_mirror_root_for(&spec),
    ] {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}
