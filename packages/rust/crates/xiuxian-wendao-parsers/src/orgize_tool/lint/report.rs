//! Report types and renderers for `orgize` lint results.

use std::path::PathBuf;

use orgize::lint::LintReport;

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
    /// Apply safe source fixes before rendering diagnostics.
    pub fix: bool,
}

/// Result of Org source linting.
#[derive(Clone, Debug)]
pub struct OrgizeLintRunReport {
    /// Per-file lint reports.
    pub files: Vec<OrgizeLintFileReport>,
    /// Safe fixes applied before linting.
    pub fixed: Vec<OrgizeLintFixReport>,
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
        let mut rendered = self
            .files
            .iter()
            .filter(|file| !file.report.is_clean())
            .map(|file| file.report.to_compact_text(&file.path, &file.source))
            .collect::<Vec<_>>();

        if self.fixed_count() > 0 {
            rendered.insert(
                0,
                format!("[fixed] orgize lint: {}\n", self.fixed_summary()),
            );
        }

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
        let fixed = self
            .fixed
            .iter()
            .map(OrgizeLintFixReport::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"fixed\":[{fixed}],\"files\":[{files}]}}\n")
    }

    fn fixed_count(&self) -> usize {
        self.fixed
            .iter()
            .map(OrgizeLintFixReport::fixed_count)
            .sum()
    }

    fn fixed_summary(&self) -> String {
        let added_ids: usize = self.fixed.iter().map(|report| report.added_ids).sum();
        let removed_redundant_properties: usize = self
            .fixed
            .iter()
            .map(|report| report.removed_redundant_properties)
            .sum();
        let fixed_metadata_lines: usize = self
            .fixed
            .iter()
            .map(|report| report.fixed_metadata_lines)
            .sum();
        let updated_lifecycle_keywords: usize = self
            .fixed
            .iter()
            .map(|report| report.updated_lifecycle_keywords)
            .sum();
        let fixed_closed_timestamps: usize = self
            .fixed
            .iter()
            .map(|report| report.fixed_closed_timestamps)
            .sum();
        let mut parts = Vec::new();
        if added_ids > 0 {
            parts.push(format!("added {added_ids} missing ID properties"));
        }
        if removed_redundant_properties > 0 {
            parts.push(format!(
                "removed {removed_redundant_properties} redundant properties"
            ));
        }
        if fixed_metadata_lines > 0 {
            parts.push(format!(
                "fixed {fixed_metadata_lines} agent Org metadata lines"
            ));
        }
        if updated_lifecycle_keywords > 0 {
            parts.push(format!(
                "updated {updated_lifecycle_keywords} lifecycle keywords"
            ));
        }
        if fixed_closed_timestamps > 0 {
            parts.push(format!("fixed {fixed_closed_timestamps} CLOSED timestamps"));
        }
        parts.join(", ")
    }
}

/// One safe Org lint fix report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgizeLintFixReport {
    /// Display path.
    pub path: String,
    /// Count of inserted ID properties.
    pub added_ids: usize,
    /// Count of removed redundant agent task properties.
    pub removed_redundant_properties: usize,
    /// Count of inserted or replaced agent Org file metadata lines.
    pub fixed_metadata_lines: usize,
    /// Count of lifecycle keyword updates.
    pub updated_lifecycle_keywords: usize,
    /// Count of converted CLOSED timestamps.
    pub fixed_closed_timestamps: usize,
}

impl OrgizeLintFixReport {
    fn fixed_count(&self) -> usize {
        self.added_ids
            + self.removed_redundant_properties
            + self.fixed_metadata_lines
            + self.updated_lifecycle_keywords
            + self.fixed_closed_timestamps
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"path\":{},\"addedIds\":{},\"removedRedundantProperties\":{},\"fixedMetadataLines\":{},\"updatedLifecycleKeywords\":{},\"fixedClosedTimestamps\":{}}}",
            serde_json::to_string(&self.path).unwrap_or_else(|_| "\"\"".to_string()),
            self.added_ids,
            self.removed_redundant_properties,
            self.fixed_metadata_lines,
            self.updated_lifecycle_keywords,
            self.fixed_closed_timestamps
        )
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
