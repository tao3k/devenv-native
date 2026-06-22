use std::{fs, fs::File, path::Path, sync::Arc};

use arrow::{
    array::{Array, ArrayRef, StringArray},
    datatypes::DataType,
    record_batch::RecordBatch,
};
use parquet::arrow::{ArrowWriter, arrow_reader::ParquetRecordBatchReaderBuilder};
use serde_json::json;
use sha2::{Digest, Sha256};
use xiuxian_wendao_episteme::{
    EpistemeOntologyCandidateGenerationReport, EpistemeOntologyCandidateGenerationRequest,
    EpistemeOntologyCandidateReadModelSummaryRequest, generate_episteme_ontology_candidates,
    summarize_episteme_ontology_candidate_read_model,
};

#[test]
fn ontology_candidate_generation_writes_review_gated_rows() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    write_episteme_fixture(temp.path())?;
    let evidence_text = "示例政策文件定义服务项目。";
    write_cache_output(
        temp.path(),
        "seed",
        "synthetic.extract.policy",
        "synthetic.file.policy",
        "docs/示例政策文件.docx",
        evidence_text,
        false,
    )?;

    let request = EpistemeOntologyCandidateGenerationRequest::new(
        temp.path(),
        "ontology_seed",
        temp.path().join("runs/extraction"),
    )
    .with_extraction_run_ids(["seed".to_string()]);
    let report = generate_episteme_ontology_candidates(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_candidate_generation_report(&report);
    assert_candidate_tsv_outputs(&report, evidence_text)?;
    assert_candidate_parquet_outputs(&report, evidence_text)?;
    assert_candidate_read_model_summary(&report)?;
    Ok(())
}

fn assert_candidate_generation_report(report: &EpistemeOntologyCandidateGenerationReport) {
    assert_eq!(report.source_file_count, 1);
    assert_eq!(report.mapping_term_count, 2);
    assert_eq!(report.extraction_evidence_count, 1);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert!(!report.ontology_truth);
}

fn assert_candidate_tsv_outputs(
    report: &EpistemeOntologyCandidateGenerationReport,
    evidence_text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let objects = fs::read_to_string(&report.candidate_objects_tsv)?;
    assert!(objects.contains("ontology_candidate.source_artifact"));
    assert!(objects.contains("policy.document"));
    assert!(objects.contains("政策文件"));
    assert!(!objects.contains(evidence_text));

    let relations = fs::read_to_string(&report.candidate_relations_tsv)?;
    assert!(relations.contains("ontology_candidate.source_artifact.suggested_object_type"));
    assert!(relations.contains("ontology_candidate.extraction_evidence.supports_source_artifact"));

    let evidence = fs::read_to_string(&report.candidate_evidence_tsv)?;
    assert!(evidence.contains("synthetic.extract.policy"));
    assert!(evidence.contains(&sha256_text(evidence_text)));
    assert!(!evidence.contains(evidence_text));

    let ledger = fs::read_to_string(&report.review_ledger_org)?;
    assert!(ledger.contains(":PROMOTION_STATE: candidate"));
    assert!(ledger.contains("不把原始文本或抽取文本直接提升为 RDF truth"));
    Ok(())
}

fn assert_candidate_parquet_outputs(
    report: &EpistemeOntologyCandidateGenerationReport,
    evidence_text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let object_batches = read_parquet_batches(&report.candidate_objects_parquet)?;
    assert_eq!(row_count(&object_batches), report.candidate_object_count);
    assert_eq!(
        object_batches[0]
            .schema()
            .field_with_name("text_char_count")?
            .data_type(),
        &DataType::Int64
    );
    assert!(
        string_values(&object_batches, "source_file_id")?
            .contains(&"synthetic.file.policy".to_string())
    );

    let relation_batches = read_parquet_batches(&report.candidate_relations_parquet)?;
    assert_eq!(
        row_count(&relation_batches),
        report.candidate_relation_count
    );
    assert!(
        string_values(&relation_batches, "relation_kind")?.contains(
            &"ontology_candidate.extraction_evidence.supports_source_artifact".to_string()
        )
    );

    let evidence_batches = read_parquet_batches(&report.candidate_evidence_parquet)?;
    assert_eq!(
        row_count(&evidence_batches),
        report.candidate_evidence_count
    );
    assert_eq!(
        evidence_batches[0]
            .schema()
            .field_with_name("text_char_count")?
            .data_type(),
        &DataType::Int64
    );
    let evidence_strings = string_values(&evidence_batches, "evidence_sha256")?;
    assert!(evidence_strings.contains(&sha256_text(evidence_text)));
    for column in ["source_path", "cache_output_path", "evidence_sha256"] {
        assert!(
            !string_values(&evidence_batches, column)?
                .iter()
                .any(|value| value.contains(evidence_text)),
            "read-model column `{column}` must not contain raw extracted text"
        );
    }
    Ok(())
}

fn assert_candidate_read_model_summary(
    report: &EpistemeOntologyCandidateGenerationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let read_model_summary = summarize_episteme_ontology_candidate_read_model(
        &EpistemeOntologyCandidateReadModelSummaryRequest::from_generation_report(report),
    )?;
    assert!(read_model_summary.read_model_gate_passed);
    assert_eq!(
        read_model_summary.candidate_object_count,
        report.candidate_object_count
    );
    assert_eq!(
        read_model_summary.candidate_relation_count,
        report.candidate_relation_count
    );
    assert_eq!(
        read_model_summary.candidate_evidence_count,
        report.candidate_evidence_count
    );
    assert_eq!(read_model_summary.missing_relation_endpoint_count, 0);
    assert_eq!(read_model_summary.review_status_violation_count, 0);
    assert_eq!(read_model_summary.promotion_status_violation_count, 0);
    assert_eq!(read_model_summary.ontology_truth_violation_count, 0);
    assert_eq!(read_model_summary.raw_to_rdf_promotion_violation_count, 0);
    Ok(())
}

#[test]
fn ontology_candidate_generation_rejects_promotable_cache_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_episteme_fixture(temp.path())?;
    write_cache_output(
        temp.path(),
        "seed",
        "synthetic.extract.policy",
        "synthetic.file.policy",
        "docs/示例政策文件.docx",
        "unsafe promoted text",
        true,
    )?;

    let request = EpistemeOntologyCandidateGenerationRequest::new(
        temp.path(),
        "ontology_seed",
        temp.path().join("runs/extraction"),
    )
    .with_extraction_run_ids(["seed".to_string()]);
    let Err(error) = generate_episteme_ontology_candidates(
        &request,
        temp.path().join("runs/ontology-generation"),
    ) else {
        return Err("promotable cache row must be rejected".into());
    };

    assert!(error.to_string().contains("must not be ontology truth"));
    Ok(())
}

#[test]
fn ontology_candidate_read_model_summary_rejects_missing_relation_endpoint()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_episteme_fixture(temp.path())?;
    write_cache_output(
        temp.path(),
        "seed",
        "synthetic.extract.policy",
        "synthetic.file.policy",
        "docs/示例政策文件.docx",
        "示例政策文件定义服务项目。",
        false,
    )?;

    let request = EpistemeOntologyCandidateGenerationRequest::new(
        temp.path(),
        "ontology_seed",
        temp.path().join("runs/extraction"),
    )
    .with_extraction_run_ids(["seed".to_string()]);
    let report = generate_episteme_ontology_candidates(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;
    rewrite_first_relation_target(&report.candidate_relations_parquet, "candidate:missing")?;

    let read_model_summary = summarize_episteme_ontology_candidate_read_model(
        &EpistemeOntologyCandidateReadModelSummaryRequest::from_generation_report(&report),
    )?;

    assert!(!read_model_summary.read_model_gate_passed);
    assert_eq!(read_model_summary.missing_relation_endpoint_count, 1);
    assert_eq!(
        read_model_summary.missing_relation_endpoints[0].endpoint_role,
        "target"
    );
    assert_eq!(
        read_model_summary.missing_relation_endpoints[0].endpoint_candidate_id,
        "candidate:missing"
    );
    Ok(())
}

fn write_episteme_fixture(root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("ontology/10_Extension/corpus"))?;
    fs::create_dir_all(root.join("ontology/10_Extension/mappings"))?;
    fs::write(root.join("ontology/manifest.toml"), ontology_manifest())?;
    fs::write(
        root.join("ontology/10_Extension/corpus/source_manifest.toml"),
        source_manifest(),
    )?;
    fs::write(
        root.join("ontology/10_Extension/corpus/files.tsv"),
        format!(
            "file_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\nsynthetic.file.policy\tdocs/示例政策文件.docx\tdocx\t12\t{}\t示例语料\tzh-CN\tdocument_text_evidence\n",
            sha256_text("source")
        ),
    )?;
    fs::write(
        root.join("ontology/10_Extension/corpus/extraction_queue.tsv"),
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus\nsynthetic.extract.policy\tsynthetic.file.policy\tdocs/示例政策文件.docx\t示例语料\tzh-CN\tdocument_text_evidence\t10\tcache_only_no_rdf_promotion\tpending\n",
    )?;
    fs::write(
        root.join("ontology/10_Extension/mappings/corpus_mapping.org"),
        mapping_ledger(),
    )?;
    Ok(())
}

fn write_cache_output(
    root: &std::path::Path,
    run_id: &str,
    queue_id: &str,
    file_id: &str,
    relative_path: &str,
    extracted_text: &str,
    ontology_truth: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = root.join("runs/extraction").join(run_id).join("outputs");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join(format!("{queue_id}.json")),
        serde_json::to_string_pretty(&json!({
            "schema_version": "xiuxian_wendao.episteme_evidence_text_cache.v1",
            "status": "succeeded",
            "queue_id": queue_id,
            "file_id": file_id,
            "relative_path": relative_path,
            "extension": "docx",
            "category": "示例语料",
            "language": "zh-CN",
            "extraction_route": "document_text_evidence",
            "source_sha256": sha256_text("source"),
            "text_sha256": sha256_text(extracted_text),
            "text_char_count": extracted_text.chars().count(),
            "extracted_text": extracted_text,
            "raw_to_rdf_promotion_allowed": ontology_truth,
            "ontology_truth": ontology_truth
        }))?,
    )?;
    Ok(())
}

fn ontology_manifest() -> &'static str {
    r#"schema_version = 1
name = "synthetic-extension-episteme"
primary_language = "zh-CN"
artifact_mode = "extension_source_contract"
mutation_allowed = false

[boundaries]
owner = "synthetic-extension-episteme"
common_domain_owner = "wendao-episteme"
runtime_compilation_owner = "xiuxian-wendao"
raw_corpus_policy = "external_evidence_root_only"
raw_to_rdf_promotion_allowed = false

[extends]
common_manifest = "episteme://synthetic/healthcare"
common_ontology_iri = "https://wendao.ai/ontology/healthcare"

[[domains]]
id = "episteme://synthetic-extension/10_Extension"
name_zh = "Extension Domain"
rdf_files = []
source_manifests = ["10_Extension/corpus/source_manifest.toml"]
mapping_ledgers = ["10_Extension/mappings/corpus_mapping.org"]
"#
}

fn source_manifest() -> &'static str {
    r#"schema_version = 1
source_contract_id = "synthetic_private.corpus.v1"
domain = "episteme://synthetic-extension/10_Extension"
primary_language = "zh-CN"
corpus_root_env = "WENDAO_SYNTHETIC_CORPUS_ROOT"
files = "files.tsv"
extraction_queue = "extraction_queue.tsv"
copy_raw_files = false
raw_to_rdf_promotion_allowed = false

[routes]
document_text_evidence = ["docx"]
"#
}

fn mapping_ledger() -> &'static str {
    r"#+TITLE: Synthetic Mapping

* Synthetic mapping
:PROPERTIES:
:ID: 0c27860c-3ae7-461e-8f82-0a2d5129fe74
:WENDAO_KIND: ontology_mapping
:ONTOLOGY_KIND: corpus_mapping
:LIFECYCLE_STATE: candidate
:DOMAIN: episteme://synthetic-extension/10_Extension
:END:

** Object candidates

| stable_key | label | note |
| policy.document | 政策文件 | policy source |

** Relation candidates

| stable_key | label | note |
| policy.defines_service | 定义服务项目 | candidate relation |
"
}

fn sha256_text(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn read_parquet_batches(path: &Path) -> Result<Vec<RecordBatch>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)?.build()?;
    reader.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn row_count(batches: &[RecordBatch]) -> usize {
    batches.iter().map(RecordBatch::num_rows).sum()
}

fn string_values(
    batches: &[RecordBatch],
    column_name: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut values = Vec::new();
    for batch in batches {
        let column = batch.column_by_name(column_name).ok_or_else(|| {
            format!(
                "missing `{column_name}` column in {:?}",
                batch
                    .schema()
                    .fields()
                    .iter()
                    .map(|field| field.name().as_str())
                    .collect::<Vec<_>>()
            )
        })?;
        let strings = column
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| format!("`{column_name}` is not a string column"))?;
        values.extend((0..strings.len()).map(|index| strings.value(index).to_string()));
    }
    Ok(values)
}

fn rewrite_first_relation_target(
    path: &Path,
    replacement: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut batches = read_parquet_batches(path)?;
    let Some(batch) = batches.pop() else {
        return Err("relation read model must contain a batch".into());
    };
    let column_index = batch.schema().index_of("target_candidate_id")?;
    let mut values = string_values(std::slice::from_ref(&batch), "target_candidate_id")?;
    let Some(first_value) = values.first_mut() else {
        return Err("relation read model must contain a target candidate".into());
    };
    *first_value = replacement.to_string();
    let mut columns = batch.columns().to_vec();
    columns[column_index] = Arc::new(StringArray::from(values)) as ArrayRef;
    let rewritten = RecordBatch::try_new(batch.schema(), columns)?;
    write_parquet_batch(path, &rewritten)
}

fn write_parquet_batch(path: &Path, batch: &RecordBatch) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)?;
    writer.write(batch)?;
    writer.close()?;
    Ok(())
}
