use crate::qianji_bpmn::{QianjiBpmnDuckDbDataStore, QianjiBpmnDuckDbDataStoreConfig};
use serde_json::Value;
use std::sync::Arc;
use tempfile::TempDir;
use xiuxian_qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnEdgeSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec,
    BpmnPackage, BpmnProcessSpec, ProcessKey, create_instance,
};

pub(super) fn must_ok<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

pub(super) fn must_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{context}"),
    }
}

pub(super) fn open_file_store(temp_dir: &TempDir, file_name: &str) -> QianjiBpmnDuckDbDataStore {
    open_store_path(temp_dir.path().join(file_name))
}

pub(super) fn open_store_path(path: impl Into<std::path::PathBuf>) -> QianjiBpmnDuckDbDataStore {
    must_ok(
        QianjiBpmnDuckDbDataStore::open(QianjiBpmnDuckDbDataStoreConfig::file(path)),
        "DuckDB workflow data store should open",
    )
}

pub(super) fn sample_checkpoint(
    instance_id: &str,
    sequence: u64,
    variables: Value,
) -> BpmnCheckpointEnvelope {
    let package = sample_package();
    sample_checkpoint_with_package(&package, instance_id, sequence, variables)
}

pub(super) fn sample_package() -> Arc<BpmnPackage> {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_duckdb", "approve", "digest_duckdb"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "end", BpmnNodeKind::EndEvent),
        ],
        vec![BpmnEdgeSpec::new(0, 1, None::<&str>)],
        Vec::new(),
    );
    Arc::new(BpmnPackage::new("pkg_duckdb", vec![process]))
}

pub(super) fn sample_checkpoint_with_package(
    package: &Arc<BpmnPackage>,
    instance_id: &str,
    sequence: u64,
    variables: Value,
) -> BpmnCheckpointEnvelope {
    let state = must_ok(
        create_instance(
            package.as_ref(),
            "approve",
            BpmnInstanceInit::new(instance_id, variables, 1_760_000_000_004),
        ),
        "known process should create an instance",
    );
    let mut state = state;
    state.sequence = sequence;
    state.updated_at_ms = 1_760_000_000_004 + sequence;
    BpmnCheckpointEnvelope::from_state(state)
}
