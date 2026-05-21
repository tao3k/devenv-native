//! Org source formatting adapter.

use std::fs;
use std::path::PathBuf;

use orgize::fmt::{FormatOptions, format_org};

use super::OrgizeToolError;
use super::io::{collect_org_paths, read_to_string};

/// Options for Org source formatting.
#[derive(Clone, Debug)]
pub struct OrgizeFormatRequest {
    /// Files or directories to format.
    pub paths: Vec<PathBuf>,
    /// Check formatting without writing changes.
    pub check: bool,
}

/// Result of Org source formatting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgizeFormatReport {
    /// Files that would change or were changed.
    pub changed_paths: Vec<PathBuf>,
}

impl OrgizeFormatReport {
    /// Returns true when at least one file needs formatting.
    #[must_use]
    pub fn changed(&self) -> bool {
        !self.changed_paths.is_empty()
    }
}

/// Formats Org files with the upstream Orgize formatter.
///
/// # Errors
///
/// Returns an error when a target cannot be inspected, read, or written.
pub fn format_org_files(
    request: &OrgizeFormatRequest,
) -> Result<OrgizeFormatReport, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let options = FormatOptions::default();
    let mut changed_paths = Vec::new();

    for path in files {
        let source = read_to_string(&path)?;
        let formatted = format_org(&source, &options);
        if formatted.changed {
            changed_paths.push(path.clone());
            if !request.check {
                fs::write(&path, formatted.output).map_err(|source| OrgizeToolError::Io {
                    path: path.clone(),
                    source,
                })?;
            }
        }
    }

    Ok(OrgizeFormatReport { changed_paths })
}
