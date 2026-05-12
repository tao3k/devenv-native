//! Shared synthetic repo-content fixture construction for search benchmarks.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::repo_index::RepoCodeDocument;
use crate::search::repo_content_chunk::publish_repo_content_chunks;
use crate::search::{
    SearchCorpusKind, SearchFileFingerprint, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneFileFingerprintScope, SearchPlaneService,
};

static REPO_PUBLICATION_BENCH_COUNTER: AtomicU64 = AtomicU64::new(1);
const BENCH_LINE_COUNT: usize = 12;

pub(super) struct RepoContentBenchmarkPaths {
    pub(super) root: PathBuf,
    pub(super) project_root: PathBuf,
    pub(super) storage_root: PathBuf,
}

pub(super) fn benchmark_suffix() -> String {
    let process_id = std::process::id();
    let counter = REPO_PUBLICATION_BENCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{process_id}-{counter}")
}

pub(super) fn assert_minimum_benchmark_documents(base_document_count: usize, benchmark_name: &str) {
    assert!(
        base_document_count >= 8,
        "{benchmark_name} requires at least 8 documents"
    );
}

pub(super) fn repo_content_benchmark_paths(
    root_name: &str,
    storage_dir_name: &str,
) -> RepoContentBenchmarkPaths {
    let root = std::env::temp_dir().join(root_name);
    let project_root = root.join("project");
    let storage_root = root.join(storage_dir_name);
    let _ = std::fs::remove_dir_all(&root);
    create_dir_all(project_root.as_path());
    RepoContentBenchmarkPaths {
        root,
        project_root,
        storage_root,
    }
}

pub(super) fn repo_content_benchmark_service(
    paths: &RepoContentBenchmarkPaths,
    manifest_keyspace: &SearchManifestKeyspace,
) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        paths.project_root.clone(),
        paths.storage_root.clone(),
        manifest_keyspace.clone(),
        SearchMaintenancePolicy::default(),
    )
}

pub(super) fn publish_base_repo_content_fixture(
    service: &SearchPlaneService,
    repo_id: &str,
    base_document_count: usize,
) {
    let base_documents = base_repo_content_documents(base_document_count);
    build_runtime().block_on(async {
        publish_repo_content_chunks(service, repo_id, &base_documents, Some("rev-1"))
            .await
            .unwrap_or_else(|error| panic!("publish base repo-content fixture: {error}"));
    });
}

pub(super) fn repo_content_fingerprints(
    service: &SearchPlaneService,
    repo_id: &str,
) -> BTreeMap<String, SearchFileFingerprint> {
    build_runtime().block_on(async {
        service
            .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoContentChunk,
                repo_id,
            ))
            .await
    })
}

pub(super) fn build_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|error| panic!("build repo-content parquet benchmark runtime: {error}"))
}

pub(super) fn repo_content_document(index: usize, token_seed: usize) -> RepoCodeDocument {
    RepoCodeDocument {
        path: repo_content_path(index),
        language: Some("julia".to_string()),
        contents: Arc::<str>::from(repo_content_body(token_seed)),
        size_bytes: u64::try_from(BENCH_LINE_COUNT * 32).unwrap_or(u64::MAX),
        modified_unix_ms: u64::try_from(token_seed).unwrap_or(u64::MAX),
    }
}

pub(super) fn repo_content_path(index: usize) -> String {
    format!("src/module_{index:05}.jl")
}

pub(super) fn unique_query_token(token_seed: usize) -> String {
    format!("value_{token_seed}_0")
}

pub(super) fn expected_row_count(base_document_count: usize) -> u64 {
    let rows = base_document_count.saturating_mul(BENCH_LINE_COUNT);
    u64::try_from(rows).unwrap_or(u64::MAX)
}

pub(super) fn persisted_metadata_backend(valkey_configured: bool) -> &'static str {
    if valkey_configured {
        "valkey_or_local_json"
    } else {
        "local_json_only"
    }
}

pub(super) fn repo_query_engine_kind() -> &'static str {
    #[cfg(feature = "duckdb")]
    {
        "duckdb"
    }
    #[cfg(not(feature = "duckdb"))]
    {
        "datafusion"
    }
}

pub(super) fn create_dir_all(path: &Path) {
    std::fs::create_dir_all(path)
        .unwrap_or_else(|error| panic!("create directory {}: {error}", path.display()));
}

pub(super) fn copy_dir_recursive(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        copy_dir_entry(source.as_ref(), target, entry)?;
    }
    Ok(())
}

fn copy_dir_entry(_source: &Path, target: &Path, entry: std::fs::DirEntry) -> std::io::Result<()> {
    let source_path = entry.path();
    let target_path = target.join(entry.file_name());
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        copy_dir_recursive(source_path.as_path(), target_path.as_path())
    } else if file_type.is_file() {
        std::fs::copy(source_path.as_path(), target_path.as_path()).map(|_| ())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!(
                "unsupported repo-content parquet benchmark entry {}",
                source_path.display()
            ),
        ))
    }
}

fn base_repo_content_documents(base_document_count: usize) -> Vec<RepoCodeDocument> {
    (0..base_document_count)
        .map(|index| repo_content_document(index, index))
        .collect()
}

fn repo_content_body(token_seed: usize) -> String {
    (0..BENCH_LINE_COUNT)
        .map(|line| format!("value_{}_{} = {}", token_seed, line, token_seed + line))
        .collect::<Vec<_>>()
        .join("\n")
}
