//! `search::manifest::fingerprint` owns Wendao search manifest fingerprint behavior.

use serde::{Deserialize, Serialize};

/// Stable file-level fingerprint payload for incremental manifest updates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct SearchFileFingerprint {
    /// Repo-relative path for the source file.
    pub relative_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional local partition identifier used to route incremental updates.
    pub partition_id: Option<String>,
    /// File size captured during manifest generation.
    pub size_bytes: u64,
    /// Modification time expressed as unix milliseconds.
    pub modified_unix_ms: u64,
    /// Extractor version that produced the manifest row.
    pub extractor_version: u32,
    /// Search-plane schema version associated with the row payload.
    pub schema_version: u32,
    /// Optional content hash used when metadata is insufficient.
    pub blake3: Option<String>,
}

impl SearchFileFingerprint {
    /// Returns whether the stored scan metadata still matches the current file snapshot.
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn matches_scan_metadata(
        &self,
        partition_id: Option<&str>,
        size_bytes: u64,
        modified_unix_ms: u64,
        extractor_version: u32,
        schema_version: u32,
    ) -> bool {
        self.partition_id.as_deref() == partition_id
            && self.size_bytes == size_bytes
            && self.modified_unix_ms == modified_unix_ms
            && self.extractor_version == extractor_version
            && self.schema_version == schema_version
    }

    /// Returns whether two fingerprints represent the same incremental search payload.
    #[must_use]
    pub fn equivalent_for_incremental(&self, other: &Self) -> bool {
        if self.relative_path != other.relative_path
            || self.partition_id != other.partition_id
            || self.extractor_version != other.extractor_version
            || self.schema_version != other.schema_version
        {
            return false;
        }

        match (&self.blake3, &other.blake3) {
            (Some(left), Some(right)) => left == right,
            _ => {
                self.size_bytes == other.size_bytes
                    && self.modified_unix_ms == other.modified_unix_ms
                    && self.blake3 == other.blake3
            }
        }
    }
}
