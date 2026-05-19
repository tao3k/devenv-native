//! Org source linting adapter.

use std::path::{Path, PathBuf};

use orgize::ast::{PriorityProfile, PriorityValue};
use orgize::lint::{LintOptions, LintReport, lint_org_with_options};

use super::OrgizeToolError;
use super::io::{collect_org_paths, read_to_string};

/// Output format for Orgize lint reports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrgizeLintOutputFormat {
    /// Compact diagnostics for agents.
    Compact,
    /// Human-readable text diagnostics.
    Text,
    /// JSON diagnostics.
    Json,
}

/// Options for Org source linting.
#[derive(Clone, Debug)]
pub struct OrgizeLintRequest {
    /// Files or directories to lint.
    pub paths: Vec<PathBuf>,
    /// Rendered lint output format.
    pub output_format: OrgizeLintOutputFormat,
    /// Optional highest priority bound.
    pub priority_highest: Option<String>,
    /// Optional lowest priority bound.
    pub priority_lowest: Option<String>,
    /// Optional default priority value.
    pub priority_default: Option<String>,
}

/// Result of Org source linting.
#[derive(Clone, Debug)]
pub struct OrgizeLintRunReport {
    /// Per-file lint reports.
    pub files: Vec<OrgizeLintFileReport>,
}

impl OrgizeLintRunReport {
    /// Returns true when no lint finding was emitted.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.files.iter().all(|file| file.report.is_clean())
    }

    /// Renders this lint report using the requested format.
    #[must_use]
    pub fn render(&self, output_format: OrgizeLintOutputFormat) -> String {
        match output_format {
            OrgizeLintOutputFormat::Compact => self.render_compact(),
            OrgizeLintOutputFormat::Text => self.render_text(),
            OrgizeLintOutputFormat::Json => self.render_json(),
        }
    }

    fn render_compact(&self) -> String {
        let rendered = self
            .files
            .iter()
            .filter(|file| !file.report.is_clean())
            .map(|file| file.report.to_compact_text(&file.path, &file.source))
            .collect::<Vec<_>>();

        if rendered.is_empty() {
            "[ok] orgize lint\n".to_string()
        } else {
            rendered.join("\n")
        }
    }

    fn render_text(&self) -> String {
        self.files
            .iter()
            .map(|file| file.report.to_text(&file.path))
            .collect::<String>()
    }

    fn render_json(&self) -> String {
        let files = self
            .files
            .iter()
            .map(|file| file.report.to_json_file(&file.path))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"files\":[{files}]}}\n")
    }
}

/// One Orgize lint report with source context.
#[derive(Clone, Debug)]
pub struct OrgizeLintFileReport {
    /// Display path used by rendered diagnostics.
    pub path: String,
    /// Original source text.
    pub source: String,
    /// Orgize lint report.
    pub report: LintReport,
}

/// Lints Org files with the upstream Orgize linter.
///
/// # Errors
///
/// Returns an error when a target cannot be inspected/read or when priority
/// profile flags are invalid.
pub fn lint_org_files(request: &OrgizeLintRequest) -> Result<OrgizeLintRunReport, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let priority_profile = priority_profile_from_flags(
        request.priority_highest.as_deref(),
        request.priority_lowest.as_deref(),
        request.priority_default.as_deref(),
    )?;
    let base_lint_options = LintOptions {
        priority_profile,
        ..LintOptions::default()
    };

    let mut reports = Vec::new();
    for path in files {
        let source = read_to_string(&path)?;
        let lint_options = LintOptions {
            include_base_dir: path.parent().map(Path::to_path_buf),
            attachment_base_dir: path.parent().map(Path::to_path_buf),
            file_base_dir: path.parent().map(Path::to_path_buf),
            ..base_lint_options.clone()
        };
        let report = lint_org_with_options(&source, &lint_options);
        reports.push(OrgizeLintFileReport {
            path: path.display().to_string(),
            source,
            report,
        });
    }

    Ok(OrgizeLintRunReport { files: reports })
}

fn parse_priority(value: &str) -> Result<PriorityValue, OrgizeToolError> {
    PriorityValue::parse(value).ok_or_else(|| OrgizeToolError::InvalidPriority {
        value: value.to_string(),
    })
}

fn priority_profile_from_flags(
    highest: Option<&str>,
    lowest: Option<&str>,
    default: Option<&str>,
) -> Result<PriorityProfile, OrgizeToolError> {
    if highest.is_none() && lowest.is_none() && default.is_none() {
        return Ok(PriorityProfile::org_default());
    }
    let profile = PriorityProfile::org_default();
    let highest = highest
        .map(parse_priority)
        .transpose()?
        .unwrap_or_else(|| profile.highest().clone());
    let lowest = lowest
        .map(parse_priority)
        .transpose()?
        .unwrap_or_else(|| profile.lowest().clone());
    let default = default
        .map(parse_priority)
        .transpose()?
        .unwrap_or_else(|| profile.default_priority().clone());
    PriorityProfile::new(highest, lowest, default).ok_or(OrgizeToolError::InvalidPriorityProfile)
}
