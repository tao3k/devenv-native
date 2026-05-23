use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const STRUCTURAL_IDF_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_structural_idf.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StructuralIdfInput {
    pub schema_version: String,
    pub documents: Vec<StructuralIdfDocumentInput>,
    pub anchors: Vec<StructuralIdfAnchorInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StructuralIdfDocumentInput {
    pub document_id: String,
    pub file_id: String,
    pub domain_id: String,
    pub source_contract_id: String,
    pub relative_path: String,
    pub sha256: String,
    pub category: String,
    pub language: String,
    pub extraction_route: String,
    pub ontology_truth: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StructuralIdfAnchorInput {
    pub anchor_id: String,
    pub anchor_kind: String,
    pub document_id: String,
    pub file_id: String,
    pub source_content_hash: String,
    pub ontology_truth: bool,
}

pub(super) fn read_structural_idf_input(path: &Path) -> Result<StructuralIdfInput> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let input: StructuralIdfInput = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse `{}`", path.display()))?;
    if input.schema_version != STRUCTURAL_IDF_SCHEMA_VERSION {
        bail!(
            "structural IDF input has unsupported schemaVersion `{}`",
            input.schema_version
        );
    }
    if input.documents.is_empty() {
        bail!("structural IDF input has no documents");
    }
    Ok(input)
}
