use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReasoningPacketInputRow {
    pub packet_id: String,
    pub reasoning_task_kind: String,
    pub document_id: String,
    pub document_anchor_id: String,
    pub file_id: String,
    pub domain_id: String,
    pub source_contract_id: String,
    pub relative_path: String,
    pub category: String,
    pub language: String,
    pub extraction_route: String,
    pub source_content_hash: String,
    pub ontology_truth: bool,
}

pub(super) fn read_reasoning_packet_rows(path: &Path) -> Result<Vec<ReasoningPacketInputRow>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let rows: Vec<ReasoningPacketInputRow> = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse `{}`", path.display()))?;
    if rows.is_empty() {
        bail!("reasoning packet JSON `{}` has no rows", path.display());
    }
    for row in &rows {
        validate_packet_row(path, row)?;
    }
    Ok(rows)
}

fn validate_packet_row(path: &Path, row: &ReasoningPacketInputRow) -> Result<()> {
    for (field, value) in [
        ("packet_id", row.packet_id.as_str()),
        ("reasoning_task_kind", row.reasoning_task_kind.as_str()),
        ("document_id", row.document_id.as_str()),
        ("document_anchor_id", row.document_anchor_id.as_str()),
        ("file_id", row.file_id.as_str()),
        ("domain_id", row.domain_id.as_str()),
        ("source_contract_id", row.source_contract_id.as_str()),
        ("relative_path", row.relative_path.as_str()),
        ("source_content_hash", row.source_content_hash.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!(
                "reasoning packet JSON `{}` row `{}` has blank {field}",
                path.display(),
                row.packet_id
            );
        }
    }
    if row.ontology_truth {
        bail!(
            "reasoning packet JSON `{}` row `{}` attempted to mark ontology truth",
            path.display(),
            row.packet_id
        );
    }
    Ok(())
}
