use xiuxian_wendao_parsers::{
    EpistemeSourceContractParseError, parse_episteme_extraction_queue_tsv,
    parse_episteme_files_tsv, parse_episteme_source_manifest_toml,
    validate_episteme_mapping_ledger_org,
};

#[test]
fn episteme_source_contract_manifest_and_tables_parse() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = parse_episteme_source_manifest_toml(MANIFEST)?;
    assert_eq!(manifest.primary_language, "zh-CN");
    assert_eq!(
        manifest.corpus_root_env,
        "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT"
    );
    assert_eq!(
        manifest.routes.get("document_text_evidence"),
        Some(&vec!["docx".to_string()])
    );

    let files = parse_episteme_files_tsv(FILES_TSV)?;
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_id, "episteme.file.a");
    assert_eq!(files[0].byte_size, 123);

    let queue = parse_episteme_extraction_queue_tsv(EXTRACTION_QUEUE_TSV)?;
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].queue_id, "episteme.extract.a");
    assert_eq!(queue[0].priority, 10);

    Ok(())
}

#[test]
fn episteme_source_contract_mapping_ledger_compiles_with_schema_governed_org_properties() {
    let validation = match validate_episteme_mapping_ledger_org(
        PRIVATE_MAPPING_ORG,
        "ontology/SourceContract/mappings/corpus_mapping.org",
    ) {
        Ok(validation) => validation,
        Err(error) => panic!("episteme source-contract mapping ledger should compile: {error}"),
    };

    assert_eq!(validation.section_count, 1);
    assert_eq!(validation.reasoning_property_record_count, 1);
}

#[test]
fn episteme_source_contract_mapping_ledger_rejects_non_uuid_reasoning_property_id() {
    let broken = PRIVATE_MAPPING_ORG.replace(
        "16b4038b-2c91-4f70-b38a-e0152629752d",
        "episteme.mapping.invalid",
    );
    let Err(error) = validate_episteme_mapping_ledger_org(
        broken.as_str(),
        "ontology/SourceContract/mappings/corpus_mapping.org",
    ) else {
        panic!("non-UUID reasoning property id should fail");
    };

    assert!(matches!(
        error,
        EpistemeSourceContractParseError::OrgReasoningProperties { .. }
    ));
}

#[test]
fn episteme_source_contract_files_tsv_rejects_bad_header() {
    let Err(error) = parse_episteme_files_tsv("bad\theader\n") else {
        panic!("bad header should fail");
    };
    assert!(matches!(
        error,
        EpistemeSourceContractParseError::TsvHeader { .. }
    ));
}

#[test]
fn episteme_source_contract_queue_tsv_rejects_row_width_mismatch() {
    let Err(error) = parse_episteme_extraction_queue_tsv(
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus\nonly-one-field\n",
    ) else {
        panic!("row width mismatch should fail");
    };
    assert!(matches!(
        error,
        EpistemeSourceContractParseError::TsvRowWidth { row: 2, .. }
    ));
}

#[test]
fn episteme_source_contract_tsv_rejects_invalid_numeric_fields() {
    let Err(error) = parse_episteme_files_tsv(
        "file_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\nepisteme.file.a\tdocs/a.docx\tdocx\tNaN\tabc\tpolicy\tzh-CN\tdocument_text_evidence\n",
    ) else {
        panic!("invalid byte size should fail");
    };
    assert!(matches!(
        error,
        EpistemeSourceContractParseError::InvalidNumber {
            row: 2,
            field: "byte_size",
            ..
        }
    ));
}

const MANIFEST: &str = r#"schema_version = 1
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
document_text_evidence = ["docx"]
"#;

const FILES_TSV: &str = "file_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\nepisteme.file.a\tdocs/a.docx\tdocx\t123\tabc\tpolicy\tzh-CN\tdocument_text_evidence\n";

const EXTRACTION_QUEUE_TSV: &str = "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus\nepisteme.extract.a\tepisteme.file.a\tdocs/a.docx\tpolicy\tzh-CN\tdocument_text_evidence\t10\tcache_only_no_rdf_promotion\tpending\n";

const PRIVATE_MAPPING_ORG: &str = r"#+TITLE: Synthetic Source Corpus Mapping Ledger

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

** Object candidates

| stable_key | label | note |
| episteme.synthetic_document | Synthetic document | Synthetic object candidate |

** Evidence policy

| decision | state | reason |
| raw files are evidence only | accepted | synthetic raw rows are not ontology truth |
";
