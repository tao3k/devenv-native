use super::BpmnOrchestrationError;
use qianji_bpmn_engine::BpmnCheckpointEnvelope;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use xiuxian_db_store::qianji_bpmn::{
    QianjiBpmnDataStoreError, QianjiBpmnDuckDbDataStore, QianjiBpmnDuckDbDataStoreConfig,
};

pub(super) fn hydrate_duckdb_latest_cache(
    cache: &Mutex<HashMap<String, BpmnCheckpointEnvelope>>,
    hydrated: &Mutex<bool>,
    store: &QianjiBpmnDuckDbDataStore,
) -> Result<(), QianjiBpmnDataStoreError> {
    let mut hydrated_guard =
        hydrated
            .lock()
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "lock_duckdb_workflow_state_latest_cache_hydrated",
                message: error.to_string(),
            })?;
    if *hydrated_guard {
        return Ok(());
    }
    store.compact_workflow_state_latest_snapshots()?;
    let checkpoints = store.load_compacted_workflow_state_snapshots()?;
    let mut cache_guard = cache
        .lock()
        .map_err(|error| QianjiBpmnDataStoreError::Storage {
            operation: "lock_duckdb_workflow_state_latest_cache",
            message: error.to_string(),
        })?;
    for checkpoint in checkpoints {
        cache_guard.insert(
            checkpoint.state.instance_id.as_ref().to_string(),
            checkpoint,
        );
    }
    *hydrated_guard = true;
    Ok(())
}

pub(super) fn cached_duckdb_checkpoint_for_store(
    cache: &Mutex<HashMap<String, BpmnCheckpointEnvelope>>,
    instance_id: &str,
) -> Result<Option<BpmnCheckpointEnvelope>, QianjiBpmnDataStoreError> {
    let guard = cache
        .lock()
        .map_err(|error| QianjiBpmnDataStoreError::Storage {
            operation: "lock_duckdb_workflow_state_latest_cache",
            message: error.to_string(),
        })?;
    Ok(guard.get(instance_id).cloned())
}

pub(super) fn cached_duckdb_checkpoint(
    cache: &Mutex<HashMap<String, BpmnCheckpointEnvelope>>,
    instance_id: &str,
) -> Result<Option<BpmnCheckpointEnvelope>, BpmnOrchestrationError> {
    let guard = cache.lock().map_err(|error| {
        BpmnOrchestrationError::DuckDbWorkflowState(QianjiBpmnDataStoreError::Storage {
            operation: "lock_duckdb_workflow_state_latest_cache",
            message: error.to_string(),
        })
    })?;
    Ok(guard.get(instance_id).cloned())
}

pub(super) fn cache_duckdb_checkpoint(
    cache: &Mutex<HashMap<String, BpmnCheckpointEnvelope>>,
    checkpoint: &BpmnCheckpointEnvelope,
) -> Result<(), BpmnOrchestrationError> {
    let mut guard = cache.lock().map_err(|error| {
        BpmnOrchestrationError::DuckDbWorkflowState(QianjiBpmnDataStoreError::Storage {
            operation: "lock_duckdb_workflow_state_latest_cache",
            message: error.to_string(),
        })
    })?;
    guard.insert(
        checkpoint.state.instance_id.as_ref().to_string(),
        checkpoint.clone(),
    );
    Ok(())
}

pub(super) fn cache_duckdb_checkpoints(
    cache: &Mutex<HashMap<String, BpmnCheckpointEnvelope>>,
    hydrated: &Mutex<bool>,
    checkpoints: &[BpmnCheckpointEnvelope],
) -> Result<(), BpmnOrchestrationError> {
    let mut guard = cache.lock().map_err(|error| {
        BpmnOrchestrationError::DuckDbWorkflowState(QianjiBpmnDataStoreError::Storage {
            operation: "lock_duckdb_workflow_state_latest_cache",
            message: error.to_string(),
        })
    })?;
    for checkpoint in checkpoints {
        guard.insert(
            checkpoint.state.instance_id.as_ref().to_string(),
            checkpoint.clone(),
        );
    }
    drop(guard);
    let mut hydrated_guard = hydrated.lock().map_err(|error| {
        BpmnOrchestrationError::DuckDbWorkflowState(QianjiBpmnDataStoreError::Storage {
            operation: "lock_duckdb_workflow_state_latest_cache_hydrated",
            message: error.to_string(),
        })
    })?;
    *hydrated_guard = true;
    Ok(())
}

pub(super) fn remove_cached_duckdb_checkpoint(
    cache: &Mutex<HashMap<String, BpmnCheckpointEnvelope>>,
    instance_id: &str,
) -> Result<(), BpmnOrchestrationError> {
    let mut guard = cache.lock().map_err(|error| {
        BpmnOrchestrationError::DuckDbWorkflowState(QianjiBpmnDataStoreError::Storage {
            operation: "lock_duckdb_workflow_state_latest_cache",
            message: error.to_string(),
        })
    })?;
    guard.remove(instance_id);
    Ok(())
}

fn open_duckdb_workflow_state_store(
    path: &Path,
) -> Result<QianjiBpmnDuckDbDataStore, BpmnOrchestrationError> {
    QianjiBpmnDuckDbDataStore::open(QianjiBpmnDuckDbDataStoreConfig::file(path.to_path_buf()))
        .map_err(Into::into)
}

pub(super) fn with_duckdb_workflow_state_store<T>(
    path: &Path,
    cache: &Mutex<Option<QianjiBpmnDuckDbDataStore>>,
    operation: impl FnOnce(&QianjiBpmnDuckDbDataStore) -> Result<T, QianjiBpmnDataStoreError>,
) -> Result<T, BpmnOrchestrationError> {
    let mut guard = cache.lock().map_err(|error| {
        BpmnOrchestrationError::DuckDbWorkflowState(QianjiBpmnDataStoreError::Storage {
            operation: "lock_duckdb_workflow_state_store",
            message: error.to_string(),
        })
    })?;
    if guard.is_none() {
        *guard = Some(open_duckdb_workflow_state_store(path)?);
    }
    let Some(store) = guard.as_ref() else {
        return Err(BpmnOrchestrationError::DuckDbWorkflowState(
            QianjiBpmnDataStoreError::Storage {
                operation: "open_duckdb_workflow_state_store",
                message: "DuckDB workflow-state cache remained empty after open".to_string(),
            },
        ));
    };
    operation(store).map_err(Into::into)
}
