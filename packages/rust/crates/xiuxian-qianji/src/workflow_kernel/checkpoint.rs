//! In-process workflow memory checkpoints.

use std::{
    any::{Any, type_name},
    collections::HashMap,
    fmt::{self, Display},
    sync::Arc,
};

use super::{WorkflowCheckpointId, WorkflowEdgeKind, WorkflowStageFacts, WorkflowStageId};

/// Storage class for a workflow checkpoint reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowCheckpointStorageKind {
    /// Same-process memory handle.
    Memory,
}

/// Serializable checkpoint reference attached to workflow traces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowCheckpointRef {
    /// Stable checkpoint identifier.
    pub checkpoint_id: String,
    /// Stage that produced the checkpointed edge.
    pub stage_id: String,
    /// Storage class for this checkpoint.
    pub storage_kind: WorkflowCheckpointStorageKind,
    /// Optional logical payload kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_kind: Option<WorkflowEdgeKind>,
    /// Optional row or item count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_count: Option<usize>,
    /// Optional producer-supplied content fingerprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_fingerprint: Option<String>,
    /// Rust type name of the retained in-process payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_type_name: Option<String>,
}

impl WorkflowCheckpointRef {
    /// Creates an in-process memory checkpoint reference from stage facts.
    #[must_use]
    pub fn memory(
        checkpoint_id: impl Into<String>,
        stage_id: impl Into<String>,
        facts: WorkflowStageFacts,
    ) -> Self {
        Self {
            checkpoint_id: checkpoint_id.into(),
            stage_id: stage_id.into(),
            storage_kind: WorkflowCheckpointStorageKind::Memory,
            edge_kind: facts.edge_kind,
            item_count: facts.item_count,
            content_fingerprint: None,
            payload_type_name: None,
        }
    }

    /// Adds a producer-supplied content fingerprint.
    #[must_use]
    pub fn with_content_fingerprint(mut self, content_fingerprint: impl Into<String>) -> Self {
        self.content_fingerprint = Some(content_fingerprint.into());
        self
    }

    /// Adds a retained Rust payload type name.
    #[must_use]
    pub fn with_payload_type_name(mut self, payload_type_name: impl Into<String>) -> Self {
        self.payload_type_name = Some(payload_type_name.into());
        self
    }
}

/// In-process memory checkpoint store.
#[derive(Clone, Default)]
pub struct WorkflowMemoryCheckpointStore {
    entries: HashMap<String, WorkflowMemoryCheckpointEntry>,
}

impl WorkflowMemoryCheckpointStore {
    /// Inserts a same-process typed payload handle.
    ///
    /// # Errors
    ///
    /// Returns an error when another checkpoint already uses the same id.
    pub fn insert<T>(
        &mut self,
        reference: WorkflowCheckpointRef,
        payload: Arc<T>,
    ) -> Result<WorkflowCheckpointRef, WorkflowCheckpointError>
    where
        T: Any + Send + Sync + 'static,
    {
        let reference = reference.with_payload_type_name(type_name::<T>());
        let checkpoint_id = reference.checkpoint_id.clone();
        if self.entries.contains_key(checkpoint_id.as_str()) {
            return Err(WorkflowCheckpointError::DuplicateCheckpoint { checkpoint_id });
        }
        self.entries.insert(
            checkpoint_id,
            WorkflowMemoryCheckpointEntry {
                reference: reference.clone(),
                payload,
            },
        );
        Ok(reference)
    }

    /// Returns true when the store contains the supplied checkpoint id.
    #[must_use]
    pub fn contains(&self, checkpoint_id: &WorkflowCheckpointId) -> bool {
        self.entries.contains_key(checkpoint_id.as_str())
    }

    /// Returns checkpoint references in stable id order.
    #[must_use]
    pub fn references(&self) -> Vec<WorkflowCheckpointRef> {
        let mut references = self
            .entries
            .values()
            .map(|entry| entry.reference.clone())
            .collect::<Vec<_>>();
        references.sort_by(|left, right| left.checkpoint_id.cmp(&right.checkpoint_id));
        references
    }

    /// Returns the typed payload for one checkpoint.
    ///
    /// # Errors
    ///
    /// Returns an error when the checkpoint is missing or the requested type
    /// does not match the retained payload type.
    pub fn get<T>(
        &self,
        checkpoint_id: &WorkflowCheckpointId,
    ) -> Result<Arc<T>, WorkflowCheckpointError>
    where
        T: Any + Send + Sync + 'static,
    {
        let entry = self.entries.get(checkpoint_id.as_str()).ok_or_else(|| {
            WorkflowCheckpointError::MissingCheckpoint {
                checkpoint_id: checkpoint_id.as_str().to_owned(),
            }
        })?;
        Arc::clone(&entry.payload).downcast::<T>().map_err(|_| {
            WorkflowCheckpointError::PayloadTypeMismatch {
                checkpoint_id: checkpoint_id.as_str().to_owned(),
                expected_type_name: type_name::<T>().to_owned(),
                actual_type_name: entry.reference.payload_type_name.clone(),
            }
        })
    }
}

impl fmt::Debug for WorkflowMemoryCheckpointStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkflowMemoryCheckpointStore")
            .field("references", &self.references())
            .finish()
    }
}

impl PartialEq for WorkflowMemoryCheckpointStore {
    fn eq(&self, other: &Self) -> bool {
        self.references() == other.references()
    }
}

impl Eq for WorkflowMemoryCheckpointStore {}

#[derive(Clone)]
struct WorkflowMemoryCheckpointEntry {
    reference: WorkflowCheckpointRef,
    payload: Arc<dyn Any + Send + Sync>,
}

/// Error returned by memory checkpoint operations.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum WorkflowCheckpointError {
    /// A checkpoint id was reused.
    #[error("workflow memory checkpoint `{checkpoint_id}` already exists")]
    DuplicateCheckpoint {
        /// Duplicate checkpoint id.
        checkpoint_id: String,
    },
    /// A checkpoint id was not found.
    #[error("workflow memory checkpoint `{checkpoint_id}` does not exist")]
    MissingCheckpoint {
        /// Missing checkpoint id.
        checkpoint_id: String,
    },
    /// A checkpoint payload was requested with the wrong Rust type.
    #[error(
        "workflow memory checkpoint `{checkpoint_id}` payload type mismatch: expected `{expected_type_name}`, actual `{actual_type_name:?}`"
    )]
    PayloadTypeMismatch {
        /// Checkpoint id.
        checkpoint_id: String,
        /// Requested Rust type name.
        expected_type_name: String,
        /// Retained payload Rust type name.
        actual_type_name: Option<String>,
    },
    /// A checkpoint was requested for a stage that has not succeeded.
    #[error("{0}")]
    StageNotSucceeded(WorkflowStageCheckpointMiss),
}

/// Stage/checkpoint pair that could not be materialized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowStageCheckpointMiss {
    /// Stage id.
    pub stage_id: WorkflowStageId,
    /// Checkpoint id.
    pub checkpoint_id: WorkflowCheckpointId,
}

impl Display for WorkflowStageCheckpointMiss {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workflow stage `{}` has no successful trace for checkpoint `{}`",
            self.stage_id, self.checkpoint_id
        )
    }
}
