//! `zhenfa_router::native::audit::fix::report` owns Wendao audit fix report behavior.

use serde::{Deserialize, Serialize};

/// Result of applying a single fix to a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFixResult {
    /// Path to the file.
    pub path: String,
    /// Result of the fix operation (as string for serialization).
    pub result: String,
    /// The line number of the fix.
    pub line_number: usize,
    /// Confidence score.
    pub confidence: f32,
}

/// Summary report of a batch fix operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixReport {
    /// Number of fixes successfully applied.
    pub successes: usize,
    /// Number of fixes that failed.
    pub failures: usize,
    /// Number of files modified.
    pub files_modified: usize,
    /// Number of files skipped (due to failures).
    pub files_skipped: usize,
    /// Detailed results for each fix.
    pub results: Vec<FileFixResult>,
    /// Error messages for failed fixes.
    pub errors: Vec<String>,
}

impl FixReport {
    /// Check if all fixes were successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.failures == 0
    }

    /// Get a summary string for display.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.is_success() {
            format!(
                "✓ Applied {} fixes to {} files",
                self.successes, self.files_modified
            )
        } else {
            format!(
                "✗ {} fixes failed, {} succeeded ({} files modified, {} skipped)",
                self.failures, self.successes, self.files_modified, self.files_skipped
            )
        }
    }
}
