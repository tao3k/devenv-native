use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(feature = "julia")]
use std::io::Cursor;
#[cfg(feature = "julia")]
use std::{
    collections::{BTreeMap, BTreeSet},
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
use xiuxian_wendao::episteme::EpistemeReadModelMaterialization;
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::{
    EpistemeReadModelRequest, materialize_episteme_read_model_seed,
    materialize_episteme_read_model_seed_with_validation_hash_cache,
};
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::{
    EpistemeValidationHashCacheReport, build_episteme_wendaograph_quality_request_batches,
    configured_episteme_corpus_root_env,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_core::{capabilities::PluginCapabilityBinding, transport::PluginTransportKind};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::{
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    WendaoGraphOntologyReadModelQualityRequestBatches,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
};

#[cfg(feature = "julia")]
pub(super) const RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_ENV: &str =
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
pub(super) const EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV: &str =
    "WENDAO_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL";

pub(super) struct EpistemeFixture {
    _temp: tempfile::TempDir,
    pub(super) episteme_root: PathBuf,
    pub(super) corpus_root: PathBuf,
    files: Vec<FileFixture>,
}

impl EpistemeFixture {
    pub(super) fn new() -> Result<Self, Box<dyn std::error::Error>> {
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

    pub(super) fn add_source(
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

    pub(super) fn write_contract(&self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub(super) fn add_legacy_office_route(&self) -> Result<(), Box<dyn std::error::Error>> {
        let manifest_path = self
            .episteme_root
            .join("ontology/SourceContract/corpus/source_manifest.toml");
        let manifest = fs::read_to_string(&manifest_path)?;
        fs::write(
            manifest_path,
            manifest.replace(
                "image_ocr_evidence = [\"jpg\"]",
                "image_ocr_evidence = [\"jpg\"]\nlegacy_office_document_evidence = [\"doc\", \"ppt\", \"xls\"]",
            ),
        )?;
        Ok(())
    }

    pub(super) fn write_multi_domain_manifest(
        &self,
        active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

    pub(super) fn mapping_ledger_path(&self) -> PathBuf {
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

pub(super) const SYNTHETIC_MAPPING_LEDGER: &str = r"#+TITLE: Synthetic Source Corpus Mapping Ledger

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

pub(super) fn table<'a>(
    materialization: &'a EpistemeReadModelMaterialization,
    table_name: &str,
) -> &'a RecordBatch {
    materialization
        .tables
        .iter()
        .find(|table| table.table_name() == table_name)
        .unwrap_or_else(|| panic!("missing table {table_name}"))
        .batch()
}

pub(super) fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a StringArray {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("missing string column {name}"))
}

pub(super) fn i64_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a Int64Array {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<Int64Array>())
        .unwrap_or_else(|| panic!("missing i64 column {name}"))
}

#[cfg(feature = "julia")]
pub(super) fn decode_single_arrow_batch(
    payload: &[u8],
) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)?;
    let batches = reader.collect::<Result<Vec<_>, _>>()?;
    let [batch] = batches.as_slice() else {
        return Err(format!("expected one Arrow batch, got {}", batches.len()).into());
    };
    Ok(batch.clone())
}

#[cfg(feature = "julia")]
pub(super) struct LiveQualityDiagnosticContext {
    pub(super) repo_root: PathBuf,
    pub(super) wendaograph_project: PathBuf,
    pub(super) runner: PathBuf,
}

#[cfg(feature = "julia")]
pub(super) fn live_quality_diagnostic_context()
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
pub(super) struct LiveQualityMaterialization {
    pub(super) materialization: EpistemeReadModelMaterialization,
    pub(super) validation_hash_cache_report: Option<EpistemeValidationHashCacheReport>,
    pub(super) elapsed_ms: f64,
}

#[cfg(feature = "julia")]
pub(super) fn materialize_live_quality_read_model(
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
pub(super) fn package_live_quality_batches(
    materialization: &EpistemeReadModelMaterialization,
) -> Result<(WendaoGraphOntologyReadModelQualityRequestBatches, f64), Box<dyn std::error::Error>> {
    let started_at = Instant::now();
    let quality_batches = build_episteme_wendaograph_quality_request_batches(materialization)?;
    let elapsed_ms = elapsed_millis(started_at);
    assert_eq!(quality_batches.row_counts(), [380, 190, 1]);
    Ok((quality_batches, elapsed_ms))
}

#[cfg(feature = "julia")]
pub(super) struct LiveQualityService {
    pub(super) _process_guard: Option<ChildGuard>,
    pub(super) binding: PluginCapabilityBinding,
    mode: &'static str,
    pub(super) base_url: String,
    pub(super) ready_ms: f64,
}

#[cfg(feature = "julia")]
pub(super) async fn start_live_quality_service(
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
pub(super) fn live_quality_external_base_url() -> Result<Option<String>, Box<dyn std::error::Error>>
{
    let Some(raw) = env::var_os(EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV) else {
        return Ok(None);
    };
    normalize_live_quality_base_url(raw.to_string_lossy().as_ref())
        .map(Some)
        .map_err(Into::into)
}

#[cfg(feature = "julia")]
pub(super) fn normalize_live_quality_base_url(raw: &str) -> Result<String, String> {
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
pub(super) fn live_quality_binding(base_url: String) -> Result<PluginCapabilityBinding, String> {
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
pub(super) async fn run_live_quality_roundtrips(
    binding: &PluginCapabilityBinding,
    quality_batches: &WendaoGraphOntologyReadModelQualityRequestBatches,
) -> Result<Vec<LiveQualityRoundtripSummary>, Box<dyn std::error::Error>> {
    let repeat_count = episteme_source_contract_quality_repeat_count()?;
    run_live_quality_roundtrip_count(binding, quality_batches, repeat_count).await
}

#[cfg(feature = "julia")]
pub(super) async fn run_live_quality_prewarm_roundtrips(
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
pub(super) struct ChildGuard {
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
pub(super) fn parse_live_quality_round_count(
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
pub(super) struct LiveQualityRoundtripSummary {
    run_index: usize,
    pub(super) elapsed_ms: f64,
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
pub(super) struct LiveQualityPhaseTimings {
    pub(super) materialization: f64,
    pub(super) request_packaging: f64,
    pub(super) service_ready: f64,
}

#[cfg(feature = "julia")]
pub(super) struct LiveQualityEvidenceInput<'a> {
    pub(super) repo_root: &'a Path,
    pub(super) source_revision: &'a str,
    pub(super) request_row_counts: [usize; 3],
    pub(super) phase_timings: LiveQualityPhaseTimings,
    pub(super) service: &'a LiveQualityService,
    pub(super) prewarm_summaries: &'a [LiveQualityRoundtripSummary],
    pub(super) roundtrip_summaries: &'a [LiveQualityRoundtripSummary],
    pub(super) validation_hash_cache_report: Option<&'a EpistemeValidationHashCacheReport>,
}

#[cfg(feature = "julia")]
pub(super) fn write_live_quality_evidence(
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

pub(super) fn write_registry_manifest(
    episteme_root: &Path,
    manifest: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(episteme_root.join("ontology"))?;
    fs::write(episteme_root.join("ontology/manifest.toml"), manifest)?;
    Ok(())
}

pub(super) fn init_git_repository(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

pub(super) fn cleanup_managed_git_entry(
    id: &str,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
